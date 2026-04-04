use std::time::{Duration, Instant};

use smithay::utils::Point;
use srwm::canvas::MomentumState;

#[test]
fn new_state_is_not_coasting() {
    let m = MomentumState::new(0.94);
    assert!(!m.coasting);
    assert_eq!(m.velocity.x, 0.0);
    assert_eq!(m.velocity.y, 0.0);
}

#[test]
fn accumulate_does_not_start_coasting() {
    let mut m = MomentumState::new(0.94);
    m.accumulate(Point::from((10.0, 5.0)), Instant::now());
    assert!(!m.coasting);
}

#[test]
fn tick_returns_none_when_not_coasting() {
    let mut m = MomentumState::new(0.94);
    let result = m.tick(Duration::from_millis(16));
    assert!(result.is_none());
}

#[test]
fn launch_starts_coasting() {
    let mut m = MomentumState::new(0.94);
    let t0 = Instant::now();
    m.accumulate(Point::from((100.0, 0.0)), t0);
    m.accumulate(Point::from((100.0, 0.0)), t0 + Duration::from_millis(10));
    m.launch();
    assert!(m.coasting);
    // Velocity should be non-zero after launch with input
    assert!(m.velocity.x.abs() > 0.0);
}

#[test]
fn tick_returns_delta_while_coasting() {
    let mut m = MomentumState::new(0.94);
    let t0 = Instant::now();
    // Feed enough velocity to be above the stop threshold (15 px/sec)
    m.accumulate(Point::from((500.0, 0.0)), t0);
    m.accumulate(Point::from((500.0, 0.0)), t0 + Duration::from_millis(10));
    m.launch();
    let delta = m.tick(Duration::from_millis(16));
    assert!(delta.is_some(), "should return delta while coasting");
    let d = delta.unwrap();
    assert!(d.x > 0.0, "delta should be in direction of velocity");
}

#[test]
fn velocity_decays_over_time() {
    let mut m = MomentumState::new(0.94);
    let t0 = Instant::now();
    m.accumulate(Point::from((500.0, 0.0)), t0);
    m.accumulate(Point::from((500.0, 0.0)), t0 + Duration::from_millis(10));
    m.launch();
    let v_before = m.velocity.x;
    m.tick(Duration::from_millis(16));
    let v_after = m.velocity.x;
    assert!(
        v_after < v_before,
        "velocity should decay: before={v_before}, after={v_after}"
    );
}

#[test]
fn coasting_eventually_stops() {
    let mut m = MomentumState::new(0.94);
    let t0 = Instant::now();
    m.accumulate(Point::from((100.0, 0.0)), t0);
    m.accumulate(Point::from((100.0, 0.0)), t0 + Duration::from_millis(10));
    m.launch();
    // Tick many frames — should eventually stop
    for _ in 0..1000 {
        if m.tick(Duration::from_millis(16)).is_none() {
            break;
        }
    }
    assert!(!m.coasting, "should stop coasting after enough frames");
}

#[test]
fn stop_resets_everything() {
    let mut m = MomentumState::new(0.94);
    let t0 = Instant::now();
    m.accumulate(Point::from((500.0, 0.0)), t0);
    m.accumulate(Point::from((500.0, 0.0)), t0 + Duration::from_millis(10));
    m.launch();
    assert!(m.coasting);
    m.stop();
    assert!(!m.coasting);
    assert_eq!(m.velocity.x, 0.0);
    assert_eq!(m.velocity.y, 0.0);
    assert!(m.tick(Duration::from_millis(16)).is_none());
}

#[test]
fn fast_fling_coasts_longer_than_slow() {
    // Fast fling
    let mut fast = MomentumState::new(0.94);
    let t0 = Instant::now();
    fast.accumulate(Point::from((2000.0, 0.0)), t0);
    fast.accumulate(Point::from((2000.0, 0.0)), t0 + Duration::from_millis(10));
    fast.launch();
    let mut fast_frames = 0;
    for _ in 0..5000 {
        if fast.tick(Duration::from_millis(16)).is_none() {
            break;
        }
        fast_frames += 1;
    }

    // Slow fling
    let mut slow = MomentumState::new(0.94);
    slow.accumulate(Point::from((20.0, 0.0)), t0);
    slow.accumulate(Point::from((20.0, 0.0)), t0 + Duration::from_millis(10));
    slow.launch();
    let mut slow_frames = 0;
    for _ in 0..5000 {
        if slow.tick(Duration::from_millis(16)).is_none() {
            break;
        }
        slow_frames += 1;
    }

    assert!(
        fast_frames > slow_frames,
        "fast fling ({fast_frames} frames) should coast longer than slow ({slow_frames} frames)"
    );
}

#[test]
fn accumulate_resets_coasting() {
    let mut m = MomentumState::new(0.94);
    let t0 = Instant::now();
    m.accumulate(Point::from((500.0, 0.0)), t0);
    m.accumulate(Point::from((500.0, 0.0)), t0 + Duration::from_millis(10));
    m.launch();
    assert!(m.coasting);
    // New input should reset coasting
    m.accumulate(Point::from((10.0, 0.0)), t0 + Duration::from_millis(20));
    assert!(!m.coasting);
}
