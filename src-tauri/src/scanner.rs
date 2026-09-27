use std::io::{self, Read, Seek, SeekFrom};

pub const RECORD_CAP: usize = 64 * 1024;
const STRING_CAP: usize = 1024;
/// Bounded JSONL projection: long string values are truncated at a valid JSON/UTF-8
/// boundary before deserialization. This preserves terminal events even when
/// last_agent_message contains megabytes. Huge arrays/objects are drained instead.
#[derive(Default)]
pub struct Scanner {
    pub offset: u64,
    pending: Vec<u8>,
    skipping: bool,
    pub skipped: u64,
    in_string: bool,
    escaped: bool,
    unicode_left: u8,
    unicode_value: u16,
    high_surrogate: bool,
    utf8_left: u8,
    string_bytes: usize,
    safe_end: usize,
    drop_string: bool,
}
impl Scanner {
    fn push(&mut self, byte: u8) {
        if self.pending.len() == RECORD_CAP {
            self.pending.clear();
            self.skipping = true;
            self.skipped += 1;
        } else {
            if self.pending.capacity() == 0 {
                self.pending.reserve_exact(RECORD_CAP);
            }
            self.pending.push(byte);
        }
    }
    pub fn feed(&mut self, bytes: &[u8], mut record: impl FnMut(&[u8])) {
        for &byte in bytes {
            self.offset += 1;
            if byte == b'\n' {
                if !self.skipping && !self.in_string {
                    record(&self.pending);
                }
                self.pending.clear();
                self.skipping = false;
                self.in_string = false;
                self.escaped = false;
                self.unicode_left = 0;
                self.high_surrogate = false;
                self.utf8_left = 0;
                self.drop_string = false;
                continue;
            }
            if self.skipping {
                continue;
            }
            if !self.in_string {
                self.push(byte);
                if byte == b'"' {
                    self.in_string = true;
                    self.string_bytes = 0;
                    self.safe_end = self.pending.len();
                    self.drop_string = false;
                    self.high_surrogate = false;
                }
                continue;
            }
            if byte == b'"' && !self.escaped && self.unicode_left == 0 {
                if self.drop_string {
                    self.pending.truncate(self.safe_end);
                }
                self.push(byte);
                self.in_string = false;
                self.utf8_left = 0;
                continue;
            }
            self.string_bytes += 1;
            if self.string_bytes > STRING_CAP {
                self.drop_string = true;
            }
            if !self.drop_string {
                self.push(byte);
            }
            if self.unicode_left > 0 {
                self.unicode_value = self
                    .unicode_value
                    .wrapping_mul(16)
                    .wrapping_add((byte as char).to_digit(16).unwrap_or(0) as u16);
                self.unicode_left -= 1;
                if self.unicode_left == 0 {
                    self.high_surrogate = (0xd800..=0xdbff).contains(&self.unicode_value);
                }
            } else if self.escaped {
                self.escaped = false;
                if byte == b'u' {
                    self.unicode_left = 4;
                    self.unicode_value = 0;
                }
            } else if byte == b'\\' {
                self.escaped = true;
            }
            if byte >= 0xc0 {
                self.utf8_left = if byte >= 0xf0 {
                    3
                } else if byte >= 0xe0 {
                    2
                } else {
                    1
                };
            } else if byte >= 0x80 {
                self.utf8_left = self.utf8_left.saturating_sub(1);
            }
            if !self.drop_string
                && !self.escaped
                && self.unicode_left == 0
                && self.utf8_left == 0
                && !self.high_surrogate
            {
                self.safe_end = self.pending.len();
            }
        }
    }
    pub fn tail(
        &mut self,
        file: &mut std::fs::File,
        budget: u64,
        mut record: impl FnMut(&[u8]),
    ) -> io::Result<bool> {
        let length = file.metadata()?.len();
        let truncated = length < self.offset;
        if truncated {
            *self = Self::default();
        }
        file.seek(SeekFrom::Start(self.offset))?;
        let mut buffer = [0u8; 16384];
        let mut remaining = budget;
        while remaining > 0 {
            let size = buffer.len().min(remaining as usize);
            let n = file.read(&mut buffer[..size])?;
            if n == 0 {
                break;
            }
            self.feed(&buffer[..n], &mut record);
            remaining -= n as u64;
        }
        Ok(truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partial_last_record() {
        let mut s = Scanner::default();
        let mut n = 0;
        s.feed(b"{\"type\":", |_| n += 1);
        assert_eq!(n, 0);
        s.feed(b"\"ok\"}\n", |_| n += 1);
        assert_eq!(n, 1);
    }
    #[test]
    fn hundred_megabytes_constant_memory_and_recovery() {
        let mut s = Scanner::default();
        let chunk = [b'x'; 16384];
        let mut n = 0;
        for _ in 0..6400 {
            s.feed(&chunk, |_| n += 1);
            assert!(s.pending.capacity() <= RECORD_CAP);
        }
        s.feed(b"\n{\"type\":\"task_complete\"}\n", |_| n += 1);
        assert_eq!(n, 1);
        assert_eq!(s.skipped, 1);
        assert!(s.pending.capacity() <= RECORD_CAP);
    }
    #[test]
    fn giant_terminal_message_preserves_completion_without_materializing_it() {
        let mut scanner = Scanner::default();
        let mut output = Vec::new();
        scanner.feed(br#"{"type":"event_msg","payload":{"type":"task_complete","turn_id":"a","last_agent_message":""#, |_|panic!("partial"));
        for _ in 0..6400 {
            scanner.feed(&[b'x'; 16384], |_| panic!("partial"));
            assert!(scanner.pending.capacity() <= RECORD_CAP);
        }
        scanner.feed(b"\"}}\n", |line| {
            output.push(serde_json::from_slice::<serde_json::Value>(line).unwrap())
        });
        assert_eq!(output[0]["payload"]["type"], "task_complete");
        assert_eq!(
            output[0]["payload"]["last_agent_message"]
                .as_str()
                .unwrap()
                .len(),
            STRING_CAP
        );
    }
    #[test]
    fn projected_strings_preserve_unicode_and_escaped_quote_boundaries() {
        let mut scanner = Scanner::default();
        let mut valid = false;
        scanner.feed(b"{\"message\":\"", |_| {});
        for _ in 0..800 {
            scanner.feed("\u{20ac}\\\"".as_bytes(), |_| {});
        }
        scanner.feed(b"\",\"type\":\"task_complete\"}\n", |line| {
            let v: serde_json::Value = serde_json::from_slice(line).unwrap();
            valid = v["type"] == "task_complete";
        });
        assert!(valid);
    }
    #[test]
    fn truncation_never_leaves_half_a_surrogate_pair() {
        let mut scanner = Scanner::default();
        let mut count = 0;
        scanner.feed(b"{\"message\":\"", |_| {});
        scanner.feed(&vec![b'x'; STRING_CAP - 6], |_| {});
        scanner.feed(b"\\uD83D\\uDE00\",\"type\":\"task_complete\"}\n", |line| {
            let value: serde_json::Value = serde_json::from_slice(line).unwrap();
            assert_eq!(value["type"], "task_complete");
            count += 1;
        });
        assert_eq!(count, 1);
    }
}
