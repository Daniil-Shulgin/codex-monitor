use crate::{
    model::{Status, Task},
    settings::Settings,
};
use std::sync::{mpsc, OnceLock};
struct Alert {
    status: Status,
    volume: u8,
}

static SENDER: OnceLock<mpsc::SyncSender<Alert>> = OnceLock::new();
pub fn transitions<'a>(old: &[Task], new: &'a [Task]) -> Vec<&'a Task> {
    new.iter()
        .filter(|task| {
            task.status != Status::Running
                && old.iter().any(|p| {
                    p.thread_id == task.thread_id
                        && p.turn_id == task.turn_id
                        && p.status != task.status
                })
        })
        .collect()
}
pub fn send(task: &Task, settings: &Settings) {
    if !settings.sound
        || settings.volume == 0
        || (task.status == Status::Done && !settings.notify_done)
        || (task.status == Status::NeedsAction && !settings.notify_action)
    {
        return;
    }
    let sender = SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel::<Alert>(32);
        std::thread::spawn(move || {
            while let Ok(alert) = rx.recv() {
                let done = alert.status == Status::Done;
                let data: &[u8] = if done {
                    include_bytes!("../assets/done.wav")
                } else {
                    include_bytes!("../assets/needs-action.wav")
                };
                if let Ok((_stream, handle)) = rodio::OutputStream::try_default() {
                    if let Ok(sink) = rodio::Sink::try_new(&handle) {
                        if let Ok(source) = rodio::Decoder::new(std::io::Cursor::new(data)) {
                            sink.set_volume(f32::from(alert.volume) / 100.0);
                            sink.append(source);
                            sink.sleep_until_end();
                        }
                    }
                }
            }
        });
        tx
    });
    let _ = sender.try_send(Alert {
        status: task.status,
        volume: settings.volume,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn task(status: Status) -> Task {
        Task {
            thread_id: "a".into(),
            turn_id: "t".into(),
            display_name: "Synthetic".into(),
            status,
            started_at: 1,
            completed_at: None,
            duration_ms: None,
            updated_at: 2,
        }
    }
    #[test]
    fn no_stale_or_duplicate_notifications() {
        let done = [task(Status::Done)];
        assert!(transitions(&[], &done).is_empty());
        assert!(transitions(&done, &done).is_empty());
        assert_eq!(transitions(&[task(Status::Running)], &done).len(), 1);
        let action = [task(Status::NeedsAction)];
        assert_eq!(transitions(&[task(Status::Running)], &action).len(), 1);
        assert!(transitions(&action, &[task(Status::Running)]).is_empty());
    }
    #[test]
    #[ignore = "Requires an interactive Windows desktop and audio device; plays both quiet presets"]
    fn native_audio() {
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();
        sink.set_volume(0.35);
        for data in [
            include_bytes!("../assets/needs-action.wav").as_slice(),
            include_bytes!("../assets/done.wav").as_slice(),
        ] {
            sink.append(rodio::Decoder::new(std::io::Cursor::new(data)).unwrap());
            sink.sleep_until_end();
        }
    }
}
