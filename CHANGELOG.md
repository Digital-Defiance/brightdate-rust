# Changelog

All notable changes to this workspace are documented here.

The workspace follows [Semantic Versioning](https://semver.org/) at the
crate-version level. The `brightdate` library crate is the canonical scalar
type and is shared by every binary in this repository (`bdate`, `btime`,
`buptime`, `bcal`, `bwatch`, and the FFI shim). Bumps to the `brightdate`
crate ripple through every consumer.

## [0.4.0] — 2026-05-18

### Removed
- **`pbd` module** — Tera-second paged-label system. The `BrightLabel`
  discriminated union (`{ kind, era, page }`), `to_pbd` / `from_pbd`,
  `to_exact_pbd` / `from_exact_pbd`, `compare_pbd`, `is_pbd_later`,
  `pbd_era`, `pbd_page`, `format_pbd` / `parse_pbd`,
  `to_bright_label` / `from_bright_label`, `format_bright_label` /
  `parse_bright_label`, `brightdate_to_label`, `bright_date_to_pbd` /
  `bright_date_from_pbd`, `bright_instant_to_pbd` / `bright_instant_from_pbd`,
  the `Pbd` / `ExactPbd` / `BrightLabel` types, and the
  `PBD_ERA_SECONDS` / `PBD_ERA_PICOSECONDS` / `DEFAULT_PBD_PRECISION`
  constants are all removed.

### Added
- **`display_label` module** — minimal `BD` / `PBD` prefix convention for
  rendering signed BrightDate scalars:
  - `format_bd(bd, precision) -> String` — renders `BD <bd>` for
    non-negative scalars, `PBD <abs(bd)>` for negative scalars. Never
    produces `PBD 0`.
  - `parse_bd(label) -> f64` — accepts both prefixes; rejects `PBD 0`
    and any negative numeric body.
  - `format_bd_label` / `parse_bd_label` — tuple-based companion API.
  - `compare_bd_labels` — total order on label tuples.
  - `BrightLabel` enum: `BD(f64)` (value ≥ 0) or `PBD(f64)` (value > 0).
  - `DEFAULT_BD_PRECISION` constant.

### Migration
The new convention is a sign-flipping display prefix only — the internal
scalar is unchanged. Callers that previously held a `BrightLabel`
tuple with `era` / `page` fields should switch to either the signed
`f64` directly or the new `BrightLabel::BD(_)` / `BrightLabel::PBD(_)`
variants. Sort order on the underlying scalars is unchanged. For
deep-time precision indefinitely far from J2000.0, use `BrightInstant`
or `ExactBrightDate` (both integer-backed) — the paged storage path
that `pbd` provided is no longer needed.

## [0.3.0] — 2026-05-18

### Added
- **`civil_time` module** — a single helper that maps a local wall-clock
  instant to its universal BrightDate scalar:
  - `bd_from_local_clock(reference, h, m, s, offset_days)` — BD value at the
    instant a local wall clock reads HH:MM:SS on the same local civil date
    as `reference`. **Does not introduce a "local fraction"**; the BD scalar
    it returns is universal. Routes through `to_unix_ms` / `from_unix_ms`,
    the documented UTC presentation boundary in spec §2.1, so leap seconds
    and the J2000 anchor are handled correctly.

### Deprecated
- `timezones::local_time_of_day` and `timezones::is_daytime` are now
  `#[deprecated(since = "0.3.0", ...)]`. They returned the fraction of a
  *BD day* offset by `offset_days`, not the fraction of any civil day,
  which is ~12 hours off from any wall-clock notion of "noon" or
  "daytime". Behavior is unchanged for binary compatibility; new code
  should not call them. BrightDate is intentionally timezone-free and
  there is exactly one BD fraction (`bd - floor(bd)`) — no civil flavor.
  For local-clock-to-BD mapping, use `civil_time::bd_from_local_clock`.

### Fixed
- Inline doc/test comments that conflated "BD-day fraction = 0.5" with
  "UTC noon" have been corrected. The BD-day boundary is the J2000.0
  anchor instant (UTC `2000-01-01T11:58:55.816Z`), not UTC midnight, so a
  BD-day fraction of `0.5` is **not** UTC noon. See specification §2.1.

### Notes for consumers
- No core scalar semantics changed. `BrightDate::from_unix_ms`,
  `BrightDate::to_unix_ms`, and the underlying TAI substrate are
  unchanged.
- `Cargo.toml` consumers should bump `brightdate = "0.3"` to pick up the
  new module. No imports break.

## [0.2.0] — earlier

### Added
- `ExactBrightDate` — picosecond-precision BigInt scalar for blockchain-grade
  archival.
- `BrightInstant` — TAI-second + nanosecond pair for distributed-systems
  timing.
- `pbd` module — Tera-second paged labels for pre-J2000.0 (deep-time)
  instants.

## [0.1.4] — earlier

- `bsh_brightdate_to_unix` FFI export.
- `bcal` BD annotations and clap argument fixes.
- `bwatch` elapsed-timing improvements.

## [0.1.3] — earlier

- `bdate`: strftime support, named calendar formats, local-tz output, `-j`
  shorthand.

## [0.1.2] — earlier

- Library-crate split (`brightdate-ffi` introduced).
