//! The injectable clock boundary (P3-W02 / Blueprint §10.2's hard
//! architecture rule):
//!
//! > "No file in the reliability core may call the real system clock
//! > directly (`SystemTime::now()` or equivalent) for any time-dependent
//! > logic. All such reads go through an injectable `Clock` trait, with
//! > a `RealClock` implementation in production and a `VirtualClock` in
//! > tests."
//!
//! `event.rs`'s [`crate::event::ClockSource`] has tagged every event with
//! which kind of clock produced its timestamp since P1 — this module is
//! what actually implements either kind, and the one place
//! `SystemTime::now()` is legally called anywhere in this crate.
//! [`RealClock::now_ms`] is that one sanctioned call site; every other
//! module that has ever needed a timestamp (`console.rs`,
//! `person_decision.rs`) already takes `now_ms` as a plain caller-supplied
//! parameter rather than reading a clock itself — this module is a legal
//! *source* for that value at the top of a call chain, not a mandate to
//! inject a `Clock` into every function that already correctly avoids
//! reading time on its own.

use crate::event::ClockSource;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A source of the current time, injectable so reliability-core logic
/// never depends on real wall-clock time to be tested deterministically.
pub trait Clock: Send + Sync {
    /// Milliseconds since the Unix epoch.
    fn now_ms(&self) -> u64;

    /// Which [`ClockSource`] this clock represents, so a caller can tag
    /// a record correctly without hand-matching on the concrete clock
    /// type it holds.
    fn source(&self) -> ClockSource;

    /// Convenience: both values in one call, since almost every caller
    /// wants to record them together (e.g. building a `NewEvent`).
    fn now_ms_and_source(&self) -> (u64, ClockSource) {
        (self.now_ms(), self.source())
    }
}

/// Wraps the crate's one legal direct read of the real system clock.
/// Every other module reads time by being handed a value, not by calling
/// this (or `SystemTime::now()`) themselves.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealClock;

impl RealClock {
    pub fn new() -> Self {
        RealClock
    }
}

impl Clock for RealClock {
    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_millis() as u64
    }

    fn source(&self) -> ClockSource {
        ClockSource::Real
    }
}

/// A controllable clock that never touches real time. Starts at a
/// caller-chosen value and only moves when explicitly told to via
/// [`VirtualClock::advance`] — Blueprint §28.5's "clock advances 30s"
/// composite-scenario operation, made concrete. Backed by `AtomicU64`
/// (the same pattern already established in
/// `mutation_attribution.rs`'s nonce counter) so it is `Send + Sync`
/// without needing a `Mutex`/`RefCell` for a single integer.
#[derive(Debug)]
pub struct VirtualClock {
    now_ms: AtomicU64,
}

impl VirtualClock {
    /// Start the clock at `start_ms` milliseconds since the Unix epoch.
    pub fn at(start_ms: u64) -> Self {
        VirtualClock {
            now_ms: AtomicU64::new(start_ms),
        }
    }

    /// Start the clock at 0 — useful when tests only care about
    /// relative time, not an absolute starting point.
    pub fn new() -> Self {
        VirtualClock::at(0)
    }

    /// Move the clock forward by exactly `delta`. Deterministic and
    /// instant — no real waiting occurs, regardless of how large `delta`
    /// is.
    pub fn advance(&self, delta: Duration) {
        self.now_ms.fetch_add(delta.as_millis() as u64, Ordering::SeqCst);
    }
}

impl Default for VirtualClock {
    fn default() -> Self {
        VirtualClock::new()
    }
}

impl Clock for VirtualClock {
    fn now_ms(&self) -> u64 {
        self.now_ms.load(Ordering::SeqCst)
    }

    fn source(&self) -> ClockSource {
        ClockSource::Virtual
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_clock_is_sanity_bounded_against_real_wall_time() {
        // The one place in this crate a comparison against real time is
        // legitimate, since RealClock's entire purpose is to read it.
        let before = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let clock_value = RealClock::new().now_ms();
        let after = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        assert!(
            clock_value >= before && clock_value <= after,
            "RealClock::now_ms must fall between two real timestamps taken immediately around it"
        );
    }

    #[test]
    fn real_clock_source_is_real() {
        assert_eq!(RealClock::new().source(), ClockSource::Real);
    }

    #[test]
    fn virtual_clock_starts_at_the_specified_value() {
        let clock = VirtualClock::at(1_000_000);
        assert_eq!(clock.now_ms(), 1_000_000);
    }

    #[test]
    fn virtual_clock_default_start_is_zero() {
        assert_eq!(VirtualClock::new().now_ms(), 0);
    }

    #[test]
    fn advance_moves_the_clock_forward_by_exactly_the_given_duration() {
        let clock = VirtualClock::at(0);
        clock.advance(Duration::from_secs(30));
        assert_eq!(clock.now_ms(), 30_000, "advance must be exact, not approximate");
    }

    #[test]
    fn multiple_advances_accumulate_correctly() {
        let clock = VirtualClock::at(0);
        clock.advance(Duration::from_millis(500));
        clock.advance(Duration::from_secs(1));
        clock.advance(Duration::from_millis(250));
        assert_eq!(clock.now_ms(), 1_750);
    }

    #[test]
    fn advance_never_actually_waits() {
        let clock = VirtualClock::at(0);
        let start = std::time::Instant::now();
        clock.advance(Duration::from_secs(3600 * 24 * 365)); // one virtual year
        assert!(
            start.elapsed() < Duration::from_millis(50),
            "advancing the virtual clock must be instant regardless of the delta"
        );
    }

    #[test]
    fn virtual_clock_source_is_virtual() {
        assert_eq!(VirtualClock::new().source(), ClockSource::Virtual);
    }

    #[test]
    fn both_clocks_work_correctly_through_a_trait_object() {
        let clocks: Vec<Box<dyn Clock>> = vec![Box::new(RealClock::new()), Box::new(VirtualClock::at(42))];
        assert_eq!(clocks[0].source(), ClockSource::Real);
        assert_eq!(clocks[1].source(), ClockSource::Virtual);
        assert_eq!(clocks[1].now_ms(), 42);
    }

    #[test]
    fn now_ms_and_source_returns_both_values_together() {
        let clock = VirtualClock::at(999);
        assert_eq!(clock.now_ms_and_source(), (999, ClockSource::Virtual));
    }
}
