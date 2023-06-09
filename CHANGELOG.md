# Fake Clock - Change Log

## [0.5.0]
- Rename `FakeClock` to `FakeInstant`.
- Use rust edition 2021.
- Add `Hash` impl.
- Add `AddAssign<Duration>`, `SubAssign<Duration>` impls.
- Make `duration_since` saturate instead of panic, matching `Instant`'s current behavior.
- Add `gh-actions` CI tests.
- Update `README.md`.

## [0.4.0]
- Add `checked_add`, `checked_sub`.
- Add `checked_duration_since`, `saturating_duration_since`.
- Internally use `Cell` instead of `RefCell`.

## [0.3.0]
- Use rust 1.22.1 stable / 2017-12-02 nightly
- rustfmt 0.9.0 and clippy-0.0.175

## [0.2.0]
- Use rust 1.19 stable / 2017-07-20 nightly
- rustfmt 0.9.0 and clippy-0.0.144
- Replace -Zno-trans with cargo check
- Make appveyor script using fixed version of stable

## [0.1.0]
- Initial implementation
