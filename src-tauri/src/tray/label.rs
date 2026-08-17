/// Formats the time remaining until the next break for the tray label:
/// under a minute shows "<1m" (rather than a confusing "0m"), under an hour
/// shows minutes ("45m"), under a day shows hours rounded to the nearest
/// whole hour ("4h"), otherwise shows days rounded to the nearest whole day
/// ("3d") — consistently round-half-up at each tier, with no combined
/// "Xd Yh" form.
pub fn format_remaining(total_minutes: i64) -> String {
    let total_minutes = total_minutes.max(0);
    if total_minutes < 1 {
        "<1m".to_string()
    } else if total_minutes < 60 {
        format!("{total_minutes}m")
    } else if total_minutes < 1440 {
        format!("{}h", round_div(total_minutes, 60))
    } else {
        format!("{}d", round_div(total_minutes, 1440))
    }
}

fn round_div(value: i64, divisor: i64) -> i64 {
    (value + divisor / 2) / divisor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_an_hour_shows_minutes() {
        assert_eq!(format_remaining(45), "45m");
        assert_eq!(format_remaining(1), "1m");
    }

    #[test]
    fn under_a_minute_shows_placeholder_instead_of_zero() {
        assert_eq!(format_remaining(0), "<1m");
        assert_eq!(format_remaining(-5), "<1m");
    }

    #[test]
    fn rounds_to_nearest_hour() {
        assert_eq!(format_remaining(225), "4h"); // 3h45m -> 4h
        assert_eq!(format_remaining(190), "3h"); // 3h10m -> 3h
        assert_eq!(format_remaining(60), "1h");
        assert_eq!(format_remaining(90), "2h"); // exactly half rounds up
    }

    #[test]
    fn rounds_to_nearest_day_for_multi_day_gaps() {
        assert_eq!(format_remaining(1440), "1d");
        assert_eq!(format_remaining(1440 * 3 + 720), "4d"); // 3.5 days rounds up
        assert_eq!(format_remaining(1440 * 3 + 100), "3d");
    }
}
