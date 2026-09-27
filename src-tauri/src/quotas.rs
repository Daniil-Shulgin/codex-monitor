use crate::model::Quota;
use serde_json::Value;

/// Select Codex's account bucket, then classify windows by duration, never position.
pub fn parse(value: &Value) -> (Quota, Quota) {
    let root = value.get("result").unwrap_or(value);
    let bucket = root
        .pointer("/rateLimitsByLimitId/codex")
        .filter(|v| v.is_object())
        .or_else(|| root.get("rateLimits"))
        .unwrap_or(root);
    let mut five = Quota::default();
    let mut week = Quota::default();
    let windows: Vec<&Value> = if let Some(a) = bucket.as_array() {
        a.iter().collect()
    } else {
        [bucket.get("primary"), bucket.get("secondary")]
            .into_iter()
            .flatten()
            .collect()
    };
    for w in windows {
        let duration = w
            .get("windowDurationMins")
            .or_else(|| w.get("window_minutes"))
            .and_then(Value::as_i64);
        let used = w
            .get("usedPercent")
            .or_else(|| w.get("used_percent"))
            .and_then(Value::as_f64);
        let remaining = w
            .get("remainingPercent")
            .and_then(Value::as_f64)
            .or_else(|| used.map(|v| 100.0 - v));
        let Some(remaining) = remaining.filter(|v| v.is_finite() && (0.0..=100.0).contains(v))
        else {
            continue;
        };
        let reset = w
            .get("resetsAt")
            .or_else(|| w.get("resets_at"))
            .and_then(Value::as_i64)
            .filter(|v| *v > 0);
        let quota = Quota {
            remaining_percent: Some(remaining),
            resets_at: reset,
        };
        match duration {
            Some(300) => five = quota,
            Some(10080) => week = quota,
            _ => {}
        }
    }
    (five, week)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn swapped_windows_and_reset() {
        let (f, w) = parse(
            &json!({"rateLimits":{"primary":{"windowDurationMins":10080,"usedPercent":58,"resetsAt":1800000000},"secondary":{"windowDurationMins":300,"usedPercent":32}}}),
        );
        assert_eq!(f.remaining_percent, Some(68.0));
        assert_eq!(w.remaining_percent, Some(42.0));
        assert_eq!(w.resets_at, Some(1800000000));
    }
    #[test]
    fn missing_weekly_only_unknown_and_malformed() {
        for v in [
            json!({}),
            json!({"primary":{"windowDurationMins":42,"usedPercent":20}}),
            json!({"primary":{"windowDurationMins":300,"usedPercent":"wrong"}}),
            json!({"primary":{"windowDurationMins":300,"usedPercent":120}}),
        ] {
            assert_eq!(parse(&v), (Quota::default(), Quota::default()));
        }
        let (f, w) = parse(&json!({"primary":{"windowDurationMins":10080,"usedPercent":2}}));
        assert_eq!(f, Quota::default());
        assert_eq!(w.remaining_percent, Some(98.0));
    }
    #[test]
    fn named_bucket_beats_unrelated_models() {
        let (f, _) = parse(
            &json!({"rateLimits":{"primary":{"windowDurationMins":300,"usedPercent":99}},"rateLimitsByLimitId":{"codex":{"primary":{"windowDurationMins":300,"usedPercent":10}}}}),
        );
        assert_eq!(f.remaining_percent, Some(90.0));
    }
}
