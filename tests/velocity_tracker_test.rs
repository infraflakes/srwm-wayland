use std::time::{Duration, Instant};

use smithay::utils::Point;
use srwc::canvas::VelocityTracker;

#[test]
fn empty_tracker_returns_zero_velocity() {
    let tracker = VelocityTracker::new();
    let v = tracker.launch_velocity();
    assert_eq!(v.x, 0.0);
    assert_eq!(v.y, 0.0);
}

#[test]
fn single_sample_returns_zero_velocity() {
    let mut tracker = VelocityTracker::new();
    tracker.push(Instant::now(), Point::from((10.0, 5.0)));
    // Need at least 2 samples for velocity
    let v = tracker.launch_velocity();
    assert_eq!(v.x, 0.0);
    assert_eq!(v.y, 0.0);
}

#[test]
fn two_samples_computes_velocity() {
    let mut tracker = VelocityTracker::new();
    let t0 = Instant::now();
    let t1 = t0 + Duration::from_millis(10);
    tracker.push(t0, Point::from((100.0, 0.0)));
    tracker.push(t1, Point::from((100.0, 0.0)));
    let v = tracker.launch_velocity();
    // total displacement = 200 px over 0.01 sec = 20000 px/sec
    assert!((v.x - 20000.0).abs() < 1.0, "vx={}", v.x);
    assert!(v.y.abs() < 1e-6);
}

#[test]
fn velocity_direction_matches_input() {
    let mut tracker = VelocityTracker::new();
    let t0 = Instant::now();
    tracker.push(t0, Point::from((-50.0, 30.0)));
    tracker.push(t0 + Duration::from_millis(10), Point::from((-50.0, 30.0)));
    let v = tracker.launch_velocity();
    assert!(v.x < 0.0, "should be negative x");
    assert!(v.y > 0.0, "should be positive y");
}

#[test]
fn old_samples_are_pruned() {
    let mut tracker = VelocityTracker::new();
    let t0 = Instant::now();
    // Push a sample that will be outside the 80ms window
    tracker.push(t0, Point::from((1000.0, 0.0)));
    // Push samples 100ms later (beyond the 80ms window)
    let t1 = t0 + Duration::from_millis(100);
    tracker.push(t1, Point::from((10.0, 0.0)));
    let t2 = t1 + Duration::from_millis(10);
    tracker.push(t2, Point::from((10.0, 0.0)));
    let v = tracker.launch_velocity();
    // The old 1000px sample should be pruned; velocity based on 20px / 0.01s = 2000
    assert!(v.x < 5000.0, "old sample should be pruned, got vx={}", v.x);
}

#[test]
fn clear_resets_tracker() {
    let mut tracker = VelocityTracker::new();
    let t0 = Instant::now();
    tracker.push(t0, Point::from((100.0, 100.0)));
    tracker.push(t0 + Duration::from_millis(10), Point::from((100.0, 100.0)));
    tracker.clear();
    let v = tracker.launch_velocity();
    assert_eq!(v.x, 0.0);
    assert_eq!(v.y, 0.0);
}

#[test]
fn last_sample_time_returns_most_recent() {
    let mut tracker = VelocityTracker::new();
    assert!(tracker.last_sample_time().is_none());
    let t0 = Instant::now();
    tracker.push(t0, Point::from((1.0, 1.0)));
    let t1 = t0 + Duration::from_millis(5);
    tracker.push(t1, Point::from((1.0, 1.0)));
    assert_eq!(tracker.last_sample_time(), Some(t1));
}

#[test]
fn simultaneous_samples_return_zero() {
    let mut tracker = VelocityTracker::new();
    let t = Instant::now();
    tracker.push(t, Point::from((100.0, 100.0)));
    tracker.push(t, Point::from((100.0, 100.0)));
    // elapsed is ~0, should return zero to avoid division by zero
    let v = tracker.launch_velocity();
    assert_eq!(v.x, 0.0);
    assert_eq!(v.y, 0.0);
}
