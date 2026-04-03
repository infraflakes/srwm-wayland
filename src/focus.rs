//! FocusTarget newtype — the SeatHandler focus type for keyboard, pointer, and touch.
//!
//! Required because `PopupGrab` needs `KeyboardFocus: From<PopupKind>`, and we
//! can't impl `From<PopupKind> for WlSurface` (orphan rule). All input-target
//! Keyboard methods route through `X11Surface` when applicable (for ICCCM
//! `SetInputFocus` + `WM_TAKE_FOCUS`); pointer/touch delegate to `WlSurface`.

use std::borrow::Cow;

use smithay::{
    backend::input::KeyState,
    desktop::PopupKind,
    input::{
        Seat,
        keyboard::{KeyboardTarget, KeysymHandle, ModifiersState},
        pointer::{
            AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent,
            GesturePinchBeginEvent, GesturePinchEndEvent, GesturePinchUpdateEvent,
            GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent, MotionEvent,
            PointerTarget, RelativeMotionEvent,
        },
        touch::{
            DownEvent as TouchDownEvent, MotionEvent as TouchMotionEvent, TouchTarget,
            UpEvent as TouchUpEvent,
        },
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{IsAlive, Serial},
    wayland::seat::WaylandFocus,
    xwayland::X11Surface,
};

use crate::state::Srwm;

// --- FocusTarget ---
// Newtype over WlSurface for use as SeatHandler focus types.
// Required because PopupGrab needs `KeyboardFocus: From<PopupKind>`,
// and we can't impl `From<PopupKind> for WlSurface` (orphan rule).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusTarget(pub WlSurface);

impl From<PopupKind> for FocusTarget {
    fn from(popup: PopupKind) -> Self {
        FocusTarget(popup.wl_surface().clone())
    }
}

impl IsAlive for FocusTarget {
    fn alive(&self) -> bool {
        self.0.alive()
    }
}

impl WaylandFocus for FocusTarget {
    fn wl_surface(&self) -> Option<Cow<'_, WlSurface>> {
        Some(Cow::Borrowed(&self.0))
    }
}

// Delegate all KeyboardTarget methods to the inner WlSurface using
// fully-qualified syntax to avoid clash with WlSurface::enter() protocol method.
impl KeyboardTarget<Srwm> for FocusTarget {
    fn enter(
        &self,
        seat: &Seat<Srwm>,
        data: &mut Srwm,
        keys: Vec<KeysymHandle<'_>>,
        serial: Serial,
    ) {
        if let Some(x11) = data.find_x11_surface_by_wl(&self.0) {
            <X11Surface as KeyboardTarget<Srwm>>::enter(&x11, seat, data, keys, serial);
        } else {
            <WlSurface as KeyboardTarget<Srwm>>::enter(&self.0, seat, data, keys, serial);
        }
    }

    fn leave(&self, seat: &Seat<Srwm>, data: &mut Srwm, serial: Serial) {
        if let Some(x11) = data.find_x11_surface_by_wl(&self.0) {
            <X11Surface as KeyboardTarget<Srwm>>::leave(&x11, seat, data, serial);
        } else {
            <WlSurface as KeyboardTarget<Srwm>>::leave(&self.0, seat, data, serial);
        }
    }

    fn key(
        &self,
        seat: &Seat<Srwm>,
        data: &mut Srwm,
        key: KeysymHandle<'_>,
        state: KeyState,
        serial: Serial,
        time: u32,
    ) {
        if let Some(x11) = data.find_x11_surface_by_wl(&self.0) {
            <X11Surface as KeyboardTarget<Srwm>>::key(&x11, seat, data, key, state, serial, time);
        } else {
            <WlSurface as KeyboardTarget<Srwm>>::key(&self.0, seat, data, key, state, serial, time);
        }
    }

    fn modifiers(
        &self,
        seat: &Seat<Srwm>,
        data: &mut Srwm,
        modifiers: ModifiersState,
        serial: Serial,
    ) {
        if let Some(x11) = data.find_x11_surface_by_wl(&self.0) {
            <X11Surface as KeyboardTarget<Srwm>>::modifiers(&x11, seat, data, modifiers, serial);
        } else {
            <WlSurface as KeyboardTarget<Srwm>>::modifiers(&self.0, seat, data, modifiers, serial);
        }
    }
}

impl PointerTarget<Srwm> for FocusTarget {
    fn enter(&self, seat: &Seat<Srwm>, data: &mut Srwm, event: &MotionEvent) {
        <WlSurface as PointerTarget<Srwm>>::enter(&self.0, seat, data, event);
    }

    fn motion(&self, seat: &Seat<Srwm>, data: &mut Srwm, event: &MotionEvent) {
        <WlSurface as PointerTarget<Srwm>>::motion(&self.0, seat, data, event);
    }

    fn relative_motion(&self, seat: &Seat<Srwm>, data: &mut Srwm, event: &RelativeMotionEvent) {
        <WlSurface as PointerTarget<Srwm>>::relative_motion(&self.0, seat, data, event);
    }

    fn button(&self, seat: &Seat<Srwm>, data: &mut Srwm, event: &ButtonEvent) {
        <WlSurface as PointerTarget<Srwm>>::button(&self.0, seat, data, event);
    }

    fn axis(&self, seat: &Seat<Srwm>, data: &mut Srwm, frame: AxisFrame) {
        <WlSurface as PointerTarget<Srwm>>::axis(&self.0, seat, data, frame);
    }

    fn frame(&self, seat: &Seat<Srwm>, data: &mut Srwm) {
        <WlSurface as PointerTarget<Srwm>>::frame(&self.0, seat, data);
    }

    fn gesture_swipe_begin(
        &self,
        seat: &Seat<Srwm>,
        data: &mut Srwm,
        event: &GestureSwipeBeginEvent,
    ) {
        <WlSurface as PointerTarget<Srwm>>::gesture_swipe_begin(&self.0, seat, data, event);
    }

    fn gesture_swipe_update(
        &self,
        seat: &Seat<Srwm>,
        data: &mut Srwm,
        event: &GestureSwipeUpdateEvent,
    ) {
        <WlSurface as PointerTarget<Srwm>>::gesture_swipe_update(&self.0, seat, data, event);
    }

    fn gesture_swipe_end(&self, seat: &Seat<Srwm>, data: &mut Srwm, event: &GestureSwipeEndEvent) {
        <WlSurface as PointerTarget<Srwm>>::gesture_swipe_end(&self.0, seat, data, event);
    }

    fn gesture_pinch_begin(
        &self,
        seat: &Seat<Srwm>,
        data: &mut Srwm,
        event: &GesturePinchBeginEvent,
    ) {
        <WlSurface as PointerTarget<Srwm>>::gesture_pinch_begin(&self.0, seat, data, event);
    }

    fn gesture_pinch_update(
        &self,
        seat: &Seat<Srwm>,
        data: &mut Srwm,
        event: &GesturePinchUpdateEvent,
    ) {
        <WlSurface as PointerTarget<Srwm>>::gesture_pinch_update(&self.0, seat, data, event);
    }

    fn gesture_pinch_end(&self, seat: &Seat<Srwm>, data: &mut Srwm, event: &GesturePinchEndEvent) {
        <WlSurface as PointerTarget<Srwm>>::gesture_pinch_end(&self.0, seat, data, event);
    }

    fn gesture_hold_begin(
        &self,
        seat: &Seat<Srwm>,
        data: &mut Srwm,
        event: &GestureHoldBeginEvent,
    ) {
        <WlSurface as PointerTarget<Srwm>>::gesture_hold_begin(&self.0, seat, data, event);
    }

    fn gesture_hold_end(&self, seat: &Seat<Srwm>, data: &mut Srwm, event: &GestureHoldEndEvent) {
        <WlSurface as PointerTarget<Srwm>>::gesture_hold_end(&self.0, seat, data, event);
    }

    fn leave(&self, seat: &Seat<Srwm>, data: &mut Srwm, serial: Serial, time: u32) {
        <WlSurface as PointerTarget<Srwm>>::leave(&self.0, seat, data, serial, time);
    }
}

impl TouchTarget<Srwm> for FocusTarget {
    fn down(&self, seat: &Seat<Srwm>, data: &mut Srwm, event: &TouchDownEvent, seq: Serial) {
        <WlSurface as TouchTarget<Srwm>>::down(&self.0, seat, data, event, seq);
    }

    fn up(&self, seat: &Seat<Srwm>, data: &mut Srwm, event: &TouchUpEvent, seq: Serial) {
        <WlSurface as TouchTarget<Srwm>>::up(&self.0, seat, data, event, seq);
    }

    fn motion(&self, seat: &Seat<Srwm>, data: &mut Srwm, event: &TouchMotionEvent, seq: Serial) {
        <WlSurface as TouchTarget<Srwm>>::motion(&self.0, seat, data, event, seq);
    }

    fn frame(&self, seat: &Seat<Srwm>, data: &mut Srwm, seq: Serial) {
        <WlSurface as TouchTarget<Srwm>>::frame(&self.0, seat, data, seq);
    }

    fn cancel(&self, seat: &Seat<Srwm>, data: &mut Srwm, seq: Serial) {
        <WlSurface as TouchTarget<Srwm>>::cancel(&self.0, seat, data, seq);
    }

    fn shape(
        &self,
        seat: &Seat<Srwm>,
        data: &mut Srwm,
        event: &smithay::input::touch::ShapeEvent,
        seq: Serial,
    ) {
        <WlSurface as TouchTarget<Srwm>>::shape(&self.0, seat, data, event, seq);
    }

    fn orientation(
        &self,
        seat: &Seat<Srwm>,
        data: &mut Srwm,
        event: &smithay::input::touch::OrientationEvent,
        seq: Serial,
    ) {
        <WlSurface as TouchTarget<Srwm>>::orientation(&self.0, seat, data, event, seq);
    }
}
