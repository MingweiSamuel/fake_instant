// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! # fake_clock
//!
//! A crate providing a virtual clock mimicking `std::time::Instant`'s interface, enabling full
//! control over the flow of time during testing.

// For explanation of lint checks, run `rustc -W help`.
#![forbid(
    bad_style,
    arithmetic_overflow,
    mutable_transmutes,
    no_mangle_const_items,
    unknown_crate_types
)]
#![deny(
    missing_docs,
    overflowing_literals,
    unsafe_code,
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_must_use
)]

use std::cell::Cell;
use std::convert::TryInto;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration;

thread_local! {
    static FAKE_TIME: Cell<u64> = Default::default();
}

/// Struct representing a fake instant.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FakeInstant {
    time_created: u64,
}

impl FakeInstant {
    /// Sets the thread-local fake time to the given value, returning the old
    /// fake time.
    pub fn set_time(time: u64) -> u64 {
        FAKE_TIME.with(|c| c.replace(time))
    }

    /// Advances the thread-local fake time by the given amount of
    /// milliseconds, returns the new fake time.
    pub fn advance_time(millis: u64) -> u64 {
        FAKE_TIME.with(|c| {
            let new_time = c.get() + millis;
            c.set(new_time);
            new_time
        })
    }

    /// Returns the current thread-local fake time.
    pub fn time() -> u64 {
        FAKE_TIME.with(|c| c.get())
    }

    /// Returns a `FakeInstant` instance representing the current thread-local
    /// fake time.
    pub fn now() -> Self {
        let time = Self::time();
        Self { time_created: time }
    }

    /// Returns the duration that passed between `self` and `earlier`.
    ///
    /// Previously this panicked when `earlier` was later than `self`.
    /// Currently this method returns a `Duration` of zero in that case. Future
    /// versions may reintroduce the panic in some circumstances.
    pub fn duration_since(self, earlier: Self) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    /// Returns the amount of fake time elapsed from another `FakeInstant` to
    /// this one, or `None` if that `FakeInstant` is earlier than this one.
    pub fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
        self.time_created
            .checked_sub(earlier.time_created)
            .map(Duration::from_millis)
    }

    /// Returns the amount of fake time elapsed from another `FakeInstant` to
    /// this one, or zero duration if that `FakeInstant` is earlier than this
    /// one.
    pub fn saturating_duration_since(&self, earlier: Self) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    /// Returns the duration of time between the creation of `self` until
    /// the thread-local fake time.
    ///
    /// Sending a `FakeInstant` across threads will result in this being
    /// computed relative to the destination thread's fake time.
    ///
    /// Previously this panicked when the current fakse time was earlier than
    /// `self`. Currently this method returns a `Duration` of zero in that
    /// case. Future versions may reintroduce the panic in some circumstances.
    pub fn elapsed(self) -> Duration {
        Duration::from_millis(Self::time() - self.time_created)
    }

    /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be
    /// represented as `FakeInstant`, `None` otherwise.
    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        duration
            .as_millis()
            .checked_add(self.time_created as u128)
            .and_then(|time| time.try_into().ok())
            .map(|time| Self { time_created: time })
    }

    /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be
    /// represented as `FakeInstant`, `None` otherwise.
    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        duration
            .as_millis()
            .try_into()
            .ok()
            .and_then(|dur| self.time_created.checked_sub(dur))
            .map(|time| Self { time_created: time })
    }
}

impl Add<Duration> for FakeInstant {
    type Output = Self;
    fn add(self, other: Duration) -> Self {
        self.checked_add(other)
            .expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for FakeInstant {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub<Duration> for FakeInstant {
    type Output = Self;
    fn sub(self, other: Duration) -> Self {
        self.checked_sub(other)
            .expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for FakeInstant {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl Sub<Self> for FakeInstant {
    type Output = Duration;
    fn sub(self, other: Self) -> Duration {
        self.duration_since(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_traits() {
        use std::any::Any;
        use std::panic::{RefUnwindSafe, UnwindSafe};

        fn check_traits<T: 'static + RefUnwindSafe + Send + Sync + Unpin + UnwindSafe + Any>() {}
        check_traits::<FakeInstant>();
    }

    #[test]
    fn test_advance_time() {
        const DUR: u64 = 5300;
        let clock = FakeInstant::now();
        FakeInstant::advance_time(DUR);
        assert_eq!(Duration::from_millis(DUR), clock.elapsed());
    }

    #[test]
    fn test_checked_add_some() {
        FakeInstant::set_time(0);

        let inst = FakeInstant::now();
        let dur = Duration::from_millis(std::u64::MAX);
        FakeInstant::set_time(std::u64::MAX);

        assert_eq!(Some(FakeInstant::now()), inst.checked_add(dur));
    }

    #[test]
    fn test_checked_add_none() {
        FakeInstant::set_time(1);

        let inst = FakeInstant::now();
        let dur = Duration::from_millis(std::u64::MAX);

        assert_eq!(None, inst.checked_add(dur));
    }

    #[test]
    fn test_checked_sub_some() {
        FakeInstant::set_time(std::u64::MAX);

        let inst = FakeInstant::now();
        let dur = Duration::from_millis(std::u64::MAX);
        FakeInstant::set_time(0);

        assert_eq!(Some(FakeInstant::now()), inst.checked_sub(dur));
    }

    #[test]
    fn test_checked_sub_none() {
        FakeInstant::set_time(std::u64::MAX - 1);

        let inst = FakeInstant::now();
        let dur = Duration::from_millis(std::u64::MAX);

        assert_eq!(None, inst.checked_sub(dur));
    }

    #[test]
    fn checked_duration_since_some() {
        FakeInstant::set_time(0);
        let inst0 = FakeInstant::now();
        FakeInstant::set_time(std::u64::MAX);
        let inst_max = FakeInstant::now();

        assert_eq!(
            Some(Duration::from_millis(std::u64::MAX)),
            inst_max.checked_duration_since(inst0)
        );
    }

    #[test]
    fn checked_duration_since_none() {
        FakeInstant::set_time(1);
        let inst1 = FakeInstant::now();
        FakeInstant::set_time(0);
        let inst0 = FakeInstant::now();

        assert_eq!(None, inst0.checked_duration_since(inst1));
    }

    #[test]
    fn saturating_duration_since_nonzero() {
        FakeInstant::set_time(0);
        let inst0 = FakeInstant::now();
        FakeInstant::set_time(std::u64::MAX);
        let inst_max = FakeInstant::now();

        assert_eq!(
            Duration::from_millis(std::u64::MAX),
            inst_max.saturating_duration_since(inst0)
        );
    }

    #[test]
    fn saturating_duration_since_zero() {
        FakeInstant::set_time(1);
        let inst1 = FakeInstant::now();
        FakeInstant::set_time(0);
        let inst0 = FakeInstant::now();

        assert_eq!(Duration::new(0, 0), inst0.saturating_duration_since(inst1));
    }

    #[test]
    fn test_debug() {
        let inst = FakeInstant::now();
        assert_eq!("FakeInstant { time_created: 0 }", format!("{:?}", inst));
    }

    #[test]
    fn test_threads() {
        FakeInstant::set_time(200);
        let inst1 = FakeInstant::now();
        assert!(std::thread::spawn(move || {
            FakeInstant::set_time(500);
            let inst2 = FakeInstant::now();
            assert_eq!(Duration::from_millis(300), inst1.elapsed());
            assert_eq!(Duration::from_millis(0), inst2.elapsed());
        })
        .join()
        .is_ok());
    }
}
