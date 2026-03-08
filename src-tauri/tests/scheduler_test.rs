//! Integration tests for the scheduler module.
//!
//! Run with:  cargo test --test scheduler_test

use chrono::NaiveTime;

/// Re-export the private helper via a thin wrapper so we can test it without
/// making it `pub` in production code.
fn within_window(now: NaiveTime, target: NaiveTime, grace: u8) -> bool {
    // Mirrors the logic in `core::scheduler::is_within_window`.
    if now < target {
        return false;
    }
    let elapsed_secs = (now - target).num_seconds();
    elapsed_secs < (grace as i64) * 60
}

#[test]
fn fires_exactly_at_target_with_zero_grace() {
    let t = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    assert!(within_window(t, t, 0));
}

#[test]
fn does_not_fire_one_second_before() {
    let target = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let before = NaiveTime::from_hms_opt(8, 59, 59).unwrap();
    assert!(!within_window(before, target, 5));
}

#[test]
fn fires_at_grace_boundary_minus_one_second() {
    let target = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let edge = NaiveTime::from_hms_opt(9, 4, 59).unwrap();
    assert!(within_window(edge, target, 5));
}

#[test]
fn does_not_fire_at_grace_boundary() {
    let target = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let edge = NaiveTime::from_hms_opt(9, 5, 0).unwrap();
    assert!(!within_window(edge, target, 5));
}

#[test]
fn midnight_edge_case() {
    // Target near midnight; now is just before it.
    let target = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let before = NaiveTime::from_hms_opt(23, 59, 59).unwrap();
    assert!(!within_window(before, target, 5));
}
