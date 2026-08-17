use chrono::{Datelike, Duration, NaiveDateTime};
use uuid::Uuid;

use crate::models::Break;

use super::state::SchedulerState;

/// Ticks land every 15-20s, so a break "on time" might first be observed a
/// little after its scheduled minute. Anything within this window still
/// counts as firing on schedule. Anything beyond it means the app was
/// asleep/closed through the scheduled time, which is always skipped rather
/// than fired late.
const FIRE_TOLERANCE_MINUTES: i64 = 5;

/// Pure decision function: given the current wall-clock time, the
/// configured breaks, and scheduler state, returns the ids of breaks due to
/// fire right now. Mutates `state` in place (idempotency bookkeeping) so a
/// single call per tick is all that's needed.
pub fn compute_due_breaks(
    now: NaiveDateTime,
    breaks: &[Break],
    state: &mut SchedulerState,
    call_active: bool,
) -> Vec<Uuid> {
    let mut due = Vec::new();

    if call_active {
        return due;
    }

    let today = now.date();
    let weekday = now.weekday();

    for b in breaks {
        if !b.enabled || !b.days.contains(&weekday) {
            continue;
        }

        if let Some(&postponed_at) = state.postponed.get(&b.id) {
            if now >= postponed_at {
                state.postponed.remove(&b.id);
                state.last_fired.insert(b.id, today);
                due.push(b.id);
            }
            continue;
        }

        if state.last_fired.get(&b.id) == Some(&today) {
            continue;
        }
        if state.skipped_today.get(&b.id) == Some(&today) {
            continue;
        }

        let scheduled = today.and_time(b.start_time);
        if now < scheduled {
            continue;
        }

        // Mark handled either way so a missed break doesn't keep getting
        // re-evaluated (and potentially fire late) on later ticks today.
        state.last_fired.insert(b.id, today);

        if now <= scheduled + Duration::minutes(FIRE_TOLERANCE_MINUTES) {
            due.push(b.id);
        }
    }

    due
}

/// Finds the soonest upcoming (day, start_time) match across all enabled
/// breaks, looking up to a week ahead. Used to drive the tray label.
pub fn next_occurrence(now: NaiveDateTime, breaks: &[Break]) -> Option<NaiveDateTime> {
    let mut soonest: Option<NaiveDateTime> = None;

    for offset in 0..7 {
        let day = now.date() + Duration::days(offset);
        let weekday = day.weekday();

        for b in breaks {
            if !b.enabled || !b.days.contains(&weekday) {
                continue;
            }

            let candidate = day.and_time(b.start_time);
            if candidate < now {
                continue;
            }

            if soonest.is_none_or(|s| candidate < s) {
                soonest = Some(candidate);
            }
        }
    }

    soonest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::break_item::BreakType;
    use chrono::{NaiveDate, NaiveTime, Weekday};
    use std::collections::HashSet;

    fn dt(y: i32, m: u32, d: u32, h: u32, min: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(h, min, 0)
            .unwrap()
    }

    fn daily_break(id: Uuid, start: NaiveTime) -> Break {
        Break {
            id,
            break_type: BreakType::Hydration,
            start_time: start,
            duration_minutes: 5,
            days: HashSet::from([
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ]),
            display_mode: None,
            image_filename: None,
            message: "Take a break now".into(),
            enabled: true,
        }
    }

    // 2026-08-17 is a Monday.
    fn monday_9am() -> NaiveDateTime {
        dt(2026, 8, 17, 9, 0)
    }

    #[test]
    fn fires_when_due() {
        let id = Uuid::new_v4();
        let breaks = vec![daily_break(id, NaiveTime::from_hms_opt(9, 0, 0).unwrap())];
        let mut state = SchedulerState::default();

        let due = compute_due_breaks(monday_9am(), &breaks, &mut state, false);
        assert_eq!(due, vec![id]);
    }

    #[test]
    fn does_not_fire_before_scheduled_time() {
        let id = Uuid::new_v4();
        let breaks = vec![daily_break(id, NaiveTime::from_hms_opt(9, 0, 0).unwrap())];
        let mut state = SchedulerState::default();

        let now = dt(2026, 8, 17, 8, 59);
        let due = compute_due_breaks(now, &breaks, &mut state, false);
        assert!(due.is_empty());
    }

    #[test]
    fn is_idempotent_within_the_same_day() {
        let id = Uuid::new_v4();
        let breaks = vec![daily_break(id, NaiveTime::from_hms_opt(9, 0, 0).unwrap())];
        let mut state = SchedulerState::default();

        let first = compute_due_breaks(monday_9am(), &breaks, &mut state, false);
        assert_eq!(first, vec![id]);

        // A later tick the same day must not refire it.
        let later = compute_due_breaks(dt(2026, 8, 17, 9, 3), &breaks, &mut state, false);
        assert!(later.is_empty());
    }

    #[test]
    fn missed_break_is_always_skipped_not_fired_late() {
        let id = Uuid::new_v4();
        let breaks = vec![daily_break(id, NaiveTime::from_hms_opt(9, 0, 0).unwrap())];
        let mut state = SchedulerState::default();

        // App wakes up an hour after the scheduled time (e.g. was asleep).
        let now = dt(2026, 8, 17, 10, 0);
        let due = compute_due_breaks(now, &breaks, &mut state, false);
        assert!(due.is_empty());

        // And it should not fire again later that same day either.
        let later = compute_due_breaks(dt(2026, 8, 17, 12, 0), &breaks, &mut state, false);
        assert!(later.is_empty());
    }

    #[test]
    fn postpone_bypasses_same_day_dedup() {
        let id = Uuid::new_v4();
        let breaks = vec![daily_break(id, NaiveTime::from_hms_opt(9, 0, 0).unwrap())];
        let mut state = SchedulerState::default();

        let first = compute_due_breaks(monday_9am(), &breaks, &mut state, false);
        assert_eq!(first, vec![id]);

        // User postpones 10 minutes.
        state.postponed.insert(id, dt(2026, 8, 17, 9, 10));

        // Not due yet at 9:05.
        let too_early = compute_due_breaks(dt(2026, 8, 17, 9, 5), &breaks, &mut state, false);
        assert!(too_early.is_empty());

        // Fires again at the postponed time, despite last_fired already
        // being set for today.
        let postponed_fire = compute_due_breaks(dt(2026, 8, 17, 9, 10), &breaks, &mut state, false);
        assert_eq!(postponed_fire, vec![id]);
        assert!(!state.postponed.contains_key(&id));
    }

    #[test]
    fn skipped_today_blocks_firing_but_not_persisted_across_days() {
        let id = Uuid::new_v4();
        let breaks = vec![daily_break(id, NaiveTime::from_hms_opt(9, 0, 0).unwrap())];
        let mut state = SchedulerState::default();
        state
            .skipped_today
            .insert(id, NaiveDate::from_ymd_opt(2026, 8, 17).unwrap());

        let due = compute_due_breaks(monday_9am(), &breaks, &mut state, false);
        assert!(due.is_empty());

        // Next day it's unaffected.
        let next_day = compute_due_breaks(dt(2026, 8, 18, 9, 0), &breaks, &mut state, false);
        assert_eq!(next_day, vec![id]);
    }

    #[test]
    fn call_active_blocks_all_firing() {
        let id = Uuid::new_v4();
        let breaks = vec![daily_break(id, NaiveTime::from_hms_opt(9, 0, 0).unwrap())];
        let mut state = SchedulerState::default();

        let due = compute_due_breaks(monday_9am(), &breaks, &mut state, true);
        assert!(due.is_empty());
    }

    #[test]
    fn disabled_break_never_fires() {
        let id = Uuid::new_v4();
        let mut b = daily_break(id, NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        b.enabled = false;
        let mut state = SchedulerState::default();

        let due = compute_due_breaks(monday_9am(), &[b], &mut state, false);
        assert!(due.is_empty());
    }

    #[test]
    fn wrong_day_never_fires() {
        let id = Uuid::new_v4();
        let mut b = daily_break(id, NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        b.days = HashSet::from([Weekday::Fri]); // Monday isn't Friday.
        let mut state = SchedulerState::default();

        let due = compute_due_breaks(monday_9am(), &[b], &mut state, false);
        assert!(due.is_empty());
    }

    #[test]
    fn next_occurrence_same_day() {
        let id = Uuid::new_v4();
        let breaks = vec![daily_break(id, NaiveTime::from_hms_opt(14, 0, 0).unwrap())];
        let next = next_occurrence(monday_9am(), &breaks).unwrap();
        assert_eq!(next, dt(2026, 8, 17, 14, 0));
    }

    #[test]
    fn next_occurrence_rolls_to_next_day_once_todays_time_has_passed() {
        let id = Uuid::new_v4();
        let breaks = vec![daily_break(id, NaiveTime::from_hms_opt(8, 0, 0).unwrap())];
        let next = next_occurrence(monday_9am(), &breaks).unwrap();
        assert_eq!(next, dt(2026, 8, 18, 8, 0));
    }

    #[test]
    fn next_occurrence_handles_multi_day_gap() {
        let id = Uuid::new_v4();
        let mut b = daily_break(id, NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        b.days = HashSet::from([Weekday::Fri]); // Checked from a Monday.
        let next = next_occurrence(monday_9am(), &[b]).unwrap();
        assert_eq!(next, dt(2026, 8, 21, 9, 0)); // Friday of the same week.
    }

    #[test]
    fn next_occurrence_none_when_no_enabled_breaks() {
        assert_eq!(next_occurrence(monday_9am(), &[]), None);
    }
}
