//! A flippable two-faced panel widget for ratatui.
//!
//! Pack twice as much content into one panel: a *front* face and a
//! *back* face. Toggle between them with a smooth horizontal-squish
//! animation that approximates a 3-D card flip on terminals that can't
//! actually rotate text.
//!
//! # Quick start
//!
//! ```no_run
//! use std::time::Duration;
//! use ratatui::{widgets::Paragraph, Frame};
//! use ratatui_flip_panel::{FlipPanel, FlipState};
//!
//! // Long-lived; survives across render calls.
//! let mut state = FlipState::new(Duration::from_millis(300));
//!
//! // On the user pressing `f`:
//! state.flip();
//!
//! // Each frame:
//! fn draw(frame: &mut Frame, state: &mut FlipState) {
//!     let widget = FlipPanel::new(
//!         |area, buf| Paragraph::new("Front face").render(area, buf),
//!         |area, buf| Paragraph::new("Back face").render(area, buf),
//!     );
//!     frame.render_stateful_widget(widget, frame.area(), state);
//! }
//! # use ratatui::widgets::Widget;
//! ```
//!
//! The widget uses the [`StatefulWidget`] trait so animation progress
//! lives in your app state, not in the widget instance.
//!
//! # When to use this vs tabs
//!
//! Reach for tabs (or [`tui-tabs`]) when:
//!
//! - You have **three or more** logically distinct views.
//! - You need the navigation strip to be visible at all times.
//! - Your terminal renders are static (no per-frame tick loop).
//!
//! Reach for `FlipPanel` when:
//!
//! - You have **exactly two** related views of the same data (a "front"
//!   summary and a "back" detail, for example).
//! - Your app already runs a per-frame render loop (≥ ~20 Hz). The
//!   animation needs frequent re-renders during its 200-400 ms window
//!   to look smooth.
//! - You want the cuteness of a flip to reinforce the "two sides of
//!   the same thing" mental model.
//!
//! [`tui-tabs`]: https://crates.io/crates/tui-tabs

#![warn(missing_docs)]
#![deny(unsafe_code)]

use std::borrow::Cow;
use std::time::{Duration, Instant};

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use ratatui_core::widgets::StatefulWidget;

/// Default minimum width at which the inner face renders. Below this
/// the widget draws a single vertical bar (the panel "edge-on").
const DEFAULT_MIN_WIDTH: u16 = 4;

/// Glyph drawn when the animation has narrowed the panel below
/// [`FlipPanel::min_width`].
const EDGE_GLYPH: &str = "│";

/// Long-lived animation state for a [`FlipPanel`].
///
/// Holds the current face (front or back) and the start time of an
/// in-progress flip, if any. The widget reads `progress()` and
/// `showing_back_after_swap()` from this each render; the application
/// calls `flip()` when the user requests a side change.
///
/// Persist this across frames — typically as a field on your
/// application state struct alongside other UI state.
#[derive(Debug, Clone)]
pub struct FlipState {
    /// Side that's stable when no animation is running.
    showing_back: bool,
    /// When the in-flight animation started, if any.
    started: Option<Instant>,
    /// How long the flip animation should take from start to settle.
    duration: Duration,
}

impl FlipState {
    /// Construct a state that starts on the front face with the given
    /// animation `duration`.
    ///
    /// Typical durations are 200-400 ms. Below ~150 ms the swap can
    /// look like a glitch; above ~500 ms it feels sluggish.
    #[must_use]
    pub fn new(duration: Duration) -> Self {
        Self {
            showing_back: false,
            started: None,
            duration,
        }
    }

    /// Begin (or reverse) a flip. If a flip is already in flight this
    /// is a no-op — wait for it to complete, then call again.
    ///
    /// Returns `true` if a new flip was started, `false` if one was
    /// already in flight.
    pub fn flip(&mut self) -> bool {
        if self.is_animating() {
            return false;
        }
        self.started = Some(Instant::now());
        true
    }

    /// True when an animation is in flight or just finished waiting
    /// for a `tick()` to finalise the side swap. Stays true from the
    /// call to `flip()` until `tick()` clears the in-flight marker —
    /// host loops should keep ticking while this returns true.
    #[must_use]
    pub fn is_animating(&self) -> bool {
        self.started.is_some()
    }

    /// True when the wall-clock animation window is still in progress
    /// (`elapsed < duration`). Use this if you specifically need to
    /// distinguish "visually mid-flip" from "finished but awaiting
    /// finalisation by `tick()`". Most callers want
    /// [`Self::is_animating`] instead.
    #[must_use]
    pub fn is_in_visual_window(&self) -> bool {
        match self.started {
            Some(t) => t.elapsed() < self.duration,
            None => false,
        }
    }

    /// Linear progress 0.0 → 1.0 across the configured duration.
    /// Returns 0.0 when no animation is running, 1.0 when one just
    /// finished and `tick()` has not yet observed it.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn progress(&self) -> f32 {
        let Some(t) = self.started else {
            return 0.0;
        };
        let elapsed = t.elapsed().as_micros() as f32;
        let total = self.duration.as_micros() as f32;
        (elapsed / total).clamp(0.0, 1.0)
    }

    /// Suggested delay until the next animation frame should render,
    /// or `None` when no animation is in flight.
    ///
    /// Use this as the event-poll timeout in event-driven hosts to
    /// keep the animation smooth without busy-looping or hard-coding
    /// a 30 Hz render loop. Returns `Some(~33 ms)` (~30 fps) during
    /// the animation, then `Some(0)` for one tick to drive
    /// finalisation, then `None`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::time::Duration;
    /// # use ratatui_flip_panel::FlipState;
    /// # fn handle_input() {}
    /// # fn redraw() {}
    /// # let mut state = FlipState::default();
    /// // In your main loop:
    /// let timeout = state.next_tick_in().unwrap_or(Duration::from_secs(60));
    /// // crossterm: `event::poll(timeout)?` — wakes either on a real
    /// // event or when the animation needs its next frame.
    /// handle_input();
    /// redraw(); // calls `frame.render_stateful_widget(widget, area, &mut state)`
    /// ```
    #[must_use]
    pub fn next_tick_in(&self) -> Option<Duration> {
        let started = self.started?;
        let elapsed = started.elapsed();
        if elapsed >= self.duration {
            // Animation window is over; ask for an immediate redraw so
            // `tick()` can finalise the side swap.
            return Some(Duration::ZERO);
        }
        let frame = Duration::from_millis(33); // ~30 fps target
        let remaining = self.duration.saturating_sub(elapsed);
        Some(frame.min(remaining))
    }

    /// Drive the state machine. Call once per render. When an
    /// animation completes, finalises the side swap and clears the
    /// in-flight marker. Returns `true` iff a completion happened
    /// this tick (useful for triggering a one-shot redraw).
    pub fn tick(&mut self) -> bool {
        if let Some(t) = self.started {
            if t.elapsed() >= self.duration {
                self.started = None;
                self.showing_back = !self.showing_back;
                return true;
            }
        }
        false
    }

    /// Triangle-wave width fraction for the current animation frame:
    /// 1.0 → 0.0 → 1.0 over progress 0.0 → 1.0. Returns 1.0 when not
    /// animating. Useful when consuming code wants to drive its own
    /// rendering path (e.g. multi-panel coordination at the app
    /// layer) instead of using [`FlipPanel`] directly.
    #[must_use]
    pub fn width_ratio(&self) -> f32 {
        width_ratio_for_progress(self.progress(), self.is_animating())
    }

    /// True when the back face is the resting / post-animation face.
    /// During an in-flight animation, this is still the *target* face
    /// only after the midpoint — the widget renders the visible face
    /// based on this together with `progress()`.
    #[must_use]
    pub fn showing_back(&self) -> bool {
        // Effective side: pre-midpoint shows old; post-midpoint shows new.
        if self.is_animating() && self.progress() >= 0.5 {
            !self.showing_back
        } else {
            self.showing_back
        }
    }

    /// Set the visible face explicitly without playing the animation.
    /// Use to restore state on app startup or after a reset.
    pub fn set_showing_back(&mut self, b: bool) {
        self.showing_back = b;
        self.started = None;
    }
}

impl Default for FlipState {
    /// Default state: front face, 300 ms flip duration.
    fn default() -> Self {
        Self::new(Duration::from_millis(300))
    }
}

/// A two-faced panel that swaps front ↔ back with a horizontal-squish
/// animation.
///
/// The animation collapses the panel's width to zero then expands it
/// back. The visible face swaps at the midpoint, giving the optical
/// illusion of a card flipping. On terminals that can't rotate text,
/// this is the closest approximation to a 3-D card flip.
///
/// `F` and `B` are render callbacks invoked at each frame. They get a
/// (possibly narrowed) `Rect` and a `Buffer` to draw into. Closing
/// over application state by reference is the typical pattern.
///
/// # Example
///
/// ```no_run
/// # use std::time::Duration;
/// # use ratatui::{Frame, widgets::{Paragraph, Widget}};
/// # use ratatui_flip_panel::{FlipPanel, FlipState};
/// # fn draw(frame: &mut Frame, state: &mut FlipState) {
/// let widget = FlipPanel::new(
///     |area, buf| Paragraph::new("front").render(area, buf),
///     |area, buf| Paragraph::new("back").render(area, buf),
/// );
/// frame.render_stateful_widget(widget, frame.area(), state);
/// # }
/// ```
pub struct FlipPanel<F, B> {
    front: F,
    back: B,
    min_width: u16,
    edge_style: Style,
    edge_glyph: Cow<'static, str>,
}

impl<F, B> FlipPanel<F, B>
where
    F: Fn(Rect, &mut Buffer),
    B: Fn(Rect, &mut Buffer),
{
    /// Construct a panel from two render callbacks. `front` runs when
    /// the front face is visible, `back` when the back face is.
    #[must_use]
    pub fn new(front: F, back: B) -> Self {
        Self {
            front,
            back,
            min_width: DEFAULT_MIN_WIDTH,
            edge_style: Style::default(),
            edge_glyph: Cow::Borrowed(EDGE_GLYPH),
        }
    }

    /// Minimum width at which the inner face renders. Below this the
    /// panel draws a single vertical bar — the visual "edge-on" state
    /// at the animation midpoint. Default: 4.
    #[must_use]
    pub fn min_width(mut self, w: u16) -> Self {
        self.min_width = w;
        self
    }

    /// Style for the edge glyph drawn at the animation midpoint.
    /// Default: terminal default.
    #[must_use]
    pub fn edge_style(mut self, style: Style) -> Self {
        self.edge_style = style;
        self
    }

    /// Glyph drawn at the animation midpoint (the "edge-on" view).
    /// Default: `│`. Typically a single grapheme — multi-cell strings
    /// will be clipped by ratatui's centred paragraph rendering. Pass
    /// any `&'static str`, `String`, or `Cow<'static, str>`.
    #[must_use]
    pub fn edge_char(mut self, glyph: impl Into<Cow<'static, str>>) -> Self {
        self.edge_glyph = glyph.into();
        self
    }

    /// Shared render path used by both the by-value and by-reference
    /// `StatefulWidget` impls. Takes `&self` so the closures can be
    /// invoked through either ownership form (they're `Fn`, not
    /// `FnOnce`, so this works).
    fn render_inner(&self, area: Rect, buf: &mut Buffer, state: &mut FlipState) {
        state.tick();

        let ratio = width_ratio_for_progress(state.progress(), state.is_animating());
        let narrow = narrowed_rect(area, ratio);
        let face_is_back = state.showing_back();

        if narrow.width >= self.min_width {
            if face_is_back {
                (self.back)(narrow, buf);
            } else {
                (self.front)(narrow, buf);
            }
        } else {
            let mid_x = narrow.x + narrow.width / 2;
            let glyph = self.edge_glyph.as_ref();
            for y in narrow.y..narrow.y.saturating_add(narrow.height) {
                if let Some(cell) = buf.cell_mut((mid_x, y)) {
                    cell.set_symbol(glyph).set_style(self.edge_style);
                }
            }
        }
    }
}

impl<F, B> StatefulWidget for FlipPanel<F, B>
where
    F: Fn(Rect, &mut Buffer),
    B: Fn(Rect, &mut Buffer),
{
    type State = FlipState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_inner(area, buf, state);
    }
}

/// Reference impl so callers can render through `&FlipPanel` without
/// consuming the widget. Useful when the panel is a long-lived field
/// on an app struct.
impl<F, B> StatefulWidget for &FlipPanel<F, B>
where
    F: Fn(Rect, &mut Buffer),
    B: Fn(Rect, &mut Buffer),
{
    type State = FlipState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_inner(area, buf, state);
    }
}

/// Compute the panel's width fraction for a given animation progress.
/// 1.0 at progress=0 and progress=1, collapsing to 0.0 at progress=0.5.
/// When not animating, returns 1.0.
fn width_ratio_for_progress(progress: f32, is_animating: bool) -> f32 {
    if !is_animating {
        return 1.0;
    }
    if progress < 0.5 {
        1.0 - (progress * 2.0)
    } else {
        (progress - 0.5) * 2.0
    }
}

/// Centre-shrink `area` to `width_ratio` of its width, preserving y/height.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn narrowed_rect(area: Rect, width_ratio: f32) -> Rect {
    if area.width == 0 || area.height == 0 {
        return area;
    }
    let mut new_width = (f32::from(area.width) * width_ratio).max(1.0) as u16;
    if new_width > area.width {
        new_width = area.width;
    }
    let x_offset = (area.width.saturating_sub(new_width)) / 2;
    Rect::new(area.x + x_offset, area.y, new_width, area.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_starts_on_front() {
        let s = FlipState::default();
        assert!(!s.showing_back());
        assert!(!s.is_animating());
    }

    #[test]
    fn flip_starts_animation_and_swaps_after_duration() {
        let mut s = FlipState::new(Duration::from_millis(1));
        assert!(s.flip());
        std::thread::sleep(Duration::from_millis(5));
        assert!(s.tick());
        assert!(s.showing_back());
        assert!(!s.is_animating());
    }

    #[test]
    fn flip_during_animation_is_noop() {
        let mut s = FlipState::new(Duration::from_secs(10));
        assert!(s.flip());
        assert!(!s.flip(), "second flip during animation must be a no-op");
    }

    #[test]
    fn set_showing_back_skips_animation() {
        let mut s = FlipState::default();
        s.set_showing_back(true);
        assert!(s.showing_back());
        assert!(!s.is_animating());
    }

    #[test]
    fn width_ratio_collapses_then_expands() {
        assert!((width_ratio_for_progress(0.0, true) - 1.0).abs() < 1e-6);
        assert!((width_ratio_for_progress(0.5, true) - 0.0).abs() < 1e-6);
        assert!((width_ratio_for_progress(1.0, true) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn width_ratio_is_one_when_not_animating() {
        assert!((width_ratio_for_progress(0.42, false) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn narrowed_rect_centres_within_original() {
        let area = Rect::new(0, 0, 100, 10);
        let narrow = narrowed_rect(area, 0.5);
        assert_eq!(narrow.width, 50);
        assert_eq!(narrow.x, 25);
        assert_eq!(narrow.height, 10);
        assert_eq!(narrow.y, 0);
    }

    #[test]
    fn narrowed_rect_clamps_to_at_least_one_column() {
        let area = Rect::new(0, 0, 100, 10);
        let narrow = narrowed_rect(area, 0.0);
        assert_eq!(narrow.width, 1);
    }
}
