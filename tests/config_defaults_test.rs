use srwc::config::{
    AccelProfile, Config, CycleModifier, GestureThresholds, InputConfig, ModKey,
    MouseDeviceSettings, NavigationConfig, TrackpadSettings, ZoomConfig,
};

#[test]
fn nav_default_trackpad_speed() {
    let nav = NavigationConfig::default();
    assert!((nav.trackpad_speed - 1.5).abs() < f64::EPSILON);
}

#[test]
fn nav_default_mouse_speed() {
    let nav = NavigationConfig::default();
    assert!((nav.mouse_speed - 1.0).abs() < f64::EPSILON);
}

#[test]
fn nav_default_friction() {
    let nav = NavigationConfig::default();
    assert!((nav.friction - 0.94).abs() < f64::EPSILON);
}

#[test]
fn nav_default_nudge_step() {
    assert_eq!(NavigationConfig::default().nudge_step, 20);
}

#[test]
fn nav_default_pan_step() {
    assert!((NavigationConfig::default().pan_step - 100.0).abs() < f64::EPSILON);
}

#[test]
fn nav_default_edge_zone() {
    assert!((NavigationConfig::default().edge_zone - 100.0).abs() < f64::EPSILON);
}

#[test]
fn nav_default_animation_speed() {
    assert!((NavigationConfig::default().animation_speed - 0.3).abs() < f64::EPSILON);
}

#[test]
fn nav_default_anchors_has_origin() {
    let nav = NavigationConfig::default();
    assert_eq!(nav.anchors.len(), 1);
    assert!((nav.anchors[0].x).abs() < f64::EPSILON);
    assert!((nav.anchors[0].y).abs() < f64::EPSILON);
}

// ── ZoomConfig defaults ──────────────────────────────────────────────

#[test]
fn zoom_default_step() {
    assert!((ZoomConfig::default().step - 1.1).abs() < f64::EPSILON);
}

#[test]
fn zoom_default_fit_padding() {
    assert!((ZoomConfig::default().fit_padding - 100.0).abs() < f64::EPSILON);
}

// ── InputConfig defaults ─────────────────────────────────────────────

#[test]
fn input_default_mod_key_is_super() {
    assert_eq!(InputConfig::default().mod_key, ModKey::Super);
}

#[test]
fn input_default_focus_follows_mouse_off() {
    assert!(!InputConfig::default().focus_follows_mouse);
}

#[test]
fn input_default_cycle_modifier_is_alt() {
    assert_eq!(InputConfig::default().cycle_modifier, CycleModifier::Alt);
}

#[test]
fn input_default_repeat_delay() {
    assert_eq!(InputConfig::default().repeat_delay, 200);
}

#[test]
fn input_default_repeat_rate() {
    assert_eq!(InputConfig::default().repeat_rate, 25);
}

#[test]
fn input_default_layout_independent_on() {
    assert!(InputConfig::default().layout_independent);
}

#[test]
fn input_default_keyboard_layout_us() {
    let input = InputConfig::default();
    assert_eq!(input.keyboard_layout.layout, "us");
    assert!(input.keyboard_layout.variant.is_empty());
    assert!(input.keyboard_layout.options.is_empty());
    assert!(input.keyboard_layout.model.is_empty());
}

#[test]
fn input_default_trackpad_settings() {
    let tp = TrackpadSettings::default();
    assert!(tp.tap_to_click);
    assert!(tp.natural_scroll);
    assert!(tp.tap_and_drag);
    assert!((tp.accel_speed).abs() < f64::EPSILON);
    assert_eq!(tp.accel_profile, AccelProfile::Adaptive);
    assert!(tp.click_method.is_none());
}

#[test]
fn input_default_mouse_device_settings() {
    let md = MouseDeviceSettings::default();
    assert!((md.accel_speed).abs() < f64::EPSILON);
    assert_eq!(md.accel_profile, AccelProfile::Flat);
    assert!(!md.natural_scroll);
}

#[test]
fn input_default_gesture_thresholds() {
    let gt = GestureThresholds::default();
    assert!((gt.swipe_distance - 12.0).abs() < f64::EPSILON);
    assert!((gt.pinch_in_scale - 0.85).abs() < f64::EPSILON);
    assert!((gt.pinch_out_scale - 1.15).abs() < f64::EPSILON);
}

// Config::from_toml("") matches sub-struct defaults

#[test]
fn empty_toml_nav_matches_defaults() {
    let config = Config::from_toml("").unwrap();
    let defaults = NavigationConfig::default();
    assert!((config.nav.trackpad_speed - defaults.trackpad_speed).abs() < f64::EPSILON);
    assert!((config.nav.friction - defaults.friction).abs() < f64::EPSILON);
    assert_eq!(config.nav.nudge_step, defaults.nudge_step);
    assert!((config.nav.pan_step - defaults.pan_step).abs() < f64::EPSILON);
    assert!((config.nav.animation_speed - defaults.animation_speed).abs() < f64::EPSILON);
}

#[test]
fn empty_toml_zoom_matches_defaults() {
    let config = Config::from_toml("").unwrap();
    let defaults = ZoomConfig::default();
    assert!((config.zoom.step - defaults.step).abs() < f64::EPSILON);
    assert!((config.zoom.fit_padding - defaults.fit_padding).abs() < f64::EPSILON);
}

#[test]
fn empty_toml_input_matches_defaults() {
    let config = Config::from_toml("").unwrap();
    let defaults = InputConfig::default();
    assert_eq!(config.input.mod_key, defaults.mod_key);
    assert_eq!(
        config.input.focus_follows_mouse,
        defaults.focus_follows_mouse
    );
    assert_eq!(config.input.repeat_delay, defaults.repeat_delay);
    assert_eq!(config.input.repeat_rate, defaults.repeat_rate);
    assert_eq!(config.input.layout_independent, defaults.layout_independent);
}
