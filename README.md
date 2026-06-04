# [BrightDate](https://brightdate.org) — Workspace

> *Named in homage to Star Trek's Stardate — one universal scalar to rule them all.*

This workspace contains the BrightDate libraries: a scientifically grounded,
timezone-free decimal day representation anchored at **J2000.0** via a **TAI
substrate**. The v1 **`f64` decimal-day** value remains the ergonomic dashboard;
the v2 **canonical engine** stores **attoseconds since J2000.0** and derives
decimal days through an integer **lens** (Euclidean divmod).

---

## Crates

| Crate | Path | Description |
|-------|------|-------------|
| `brightdate` | `crates/brightdate/` | Core types, TAI conversions, leap second table, v2 attosecond engine |
| `bdate` | `crates/bdate/` | Date formatting and calendar utilities |
| `btime` | `crates/btime/` | Time-of-day utilities |
| `buptime` | `crates/buptime/` | Process uptime tracking |
| `bcal` | `crates/bcal/` | Calendar arithmetic |
| `bwatch` | `crates/bwatch/` | Stopwatch / interval utilities |
| `brightdate-ffi` | `crates/brightdate-ffi/` | C FFI shim for the Rust library |

## TypeScript library

| Package | Path | Description |
|---------|------|-------------|
| `@brightchain/brightdate` | `brightdate/` | TypeScript/JavaScript BrightDate library (npm) |

## Showcase

| App | Path | Description |
|-----|------|-------------|
| BrightDate Showcase | `brightdate/showcase/` | Vite+React live demo |

---

## The BrightDate v1.0 Design

### Epoch: J2000.0 — Astronomically Correct

J2000.0 is defined in **Terrestrial Time (TT)** as `2000-01-01T12:00:00 TT`.

| Timescale | Representation | Value |
|-----------|---------------|-------|
| TT (definition) | `2000-01-01T12:00:00.000` (no zone) | Unix s `946_728_000` |
| TAI | `2000-01-01T11:59:27.816` (no zone) | Unix s `946_727_967.816` |
| **UTC label** | **`2000-01-01T11:58:55.816Z`** | **Unix ms `946_727_935_816`** |

`BrightDate = 0` at `2000-01-01T11:58:55.816Z`.

### TAI Substrate

```
bd = (taiUnixSeconds − 946_727_967.816) / 86400
```

BrightDate ticks in exact SI seconds. Leap seconds exist only at UTC boundary conversions.

### Key Constants

| Constant | Value |
|----------|-------|
| `J2000_UTC_UNIX_MS` | `946_727_935_816` |
| `J2000_TAI_UNIX_S` | `946_727_967.816` |
| `J2000_TT_UNIX_S` | `946_728_000` |
| `TAI_UTC_OFFSET_AT_J2000` | `32` s |
| `TT_TAI_OFFSET_SECONDS` | `32.184` s |
| `CURRENT_TAI_UTC_OFFSET` | `37` s (as of 2017) |
| `GPS_EPOCH_UNIX_TAI` | `315_964_819` |

### Reference Epochs

| Event | UTC | BrightDate |
|-------|-----|-----------|
| J2000.0 anchor | `2000-01-01T11:58:55.816Z` | `0.000000000` |
| TT noon (definition) | `2000-01-01T12:00:00.000Z` | `≈ 0.000742870` |
| Y2K midnight | `2000-01-01T00:00:00Z` | `≈ −0.499257130` |
| Unix epoch | `1970-01-01T00:00:00Z` | `≈ −10957.499512` |
| GPS epoch | `1980-01-06T00:00:00Z` | `≈ −7300.499408` |

---

## Companion Types

Pick the representation that matches your boundary: **store** in an exact integer
engine, **compute** in `BrightDate` when `f64` is enough, **label** with `BD` /
`PBD`. All types share the same J2000.0 / TAI semantics; they differ only in
storage and how decimal days are produced.

| Type | Role | Representation | Use when… |
|------|------|----------------|-----------|
| **`ExactBrightAtto`** | **v2 canonical engine** | `i128` attoseconds since J2000.0 (`EBA1:` wire, 16-byte BE) | Default for new storage, spacetime intervals, and anything that must not accumulate `f64` error. One attosecond ≡ one **light-attosecond** under Bright Spacetime `c = 1`. |
| **`ExactBrightDate`** | Picosecond engine | `i128` picoseconds since J2000.0 (`EBD1:`) | You want picosecond ticks without attosecond width; converts to/from `ExactBrightAtto` exactly. |
| **`BrightDate`** | **v1 dashboard** | `f64` decimal days since J2000.0 | Math, logging, UI, sorting — the ergonomic scalar. Derived from integer ticks via the **lens** (lossy only in the final `f64` combine). |
| **`BrightInstant`** | Civil instant | `i64` TAI seconds + `u32` nanos since J2000.0 | Nanosecond-precision instants at any magnitude (GPS, distributed clocks). |

### Integer lens (engine ↔ decimal days)

Canonical ticks use Euclidean divmod; decimal days are a **presentation lens**:

```text
bd = days + rem / ticks_per_day     (days, rem from divmod on attoseconds or picoseconds)
```

Exported helpers: `ticks_to_brightdate`, `brightdate_to_attoseconds`,
`brightdate_to_picoseconds`, and day constants such as `ATTOSECONDS_PER_DAY`.

**Rust** and **TypeScript** both ship these types. `BrightDate::from_unix_ms` /
`to_unix_ms` apply the leap-second table on the UTC label; `ExactBrightAtto` uses
linear Unix-ms offset from the J2000 UTC anchor — use the engine at storage
boundaries and the dashboard for human-facing decimal days.

### Unit hierarchies (do not conflate)

| Family | Examples | Meaning |
|--------|----------|---------|
| **Decimal day** | milliday (`md`), microday (`μd`), nanoday (`nd`) | Fractions of one **BrightDate day** (`f64` or lens-derived) |
| **Bright time** | Bright-Second (`bs`), kilobright-second (`kbs`) | SI seconds on the Bright timeline (86400 `bs` = 1 day) |

```rust
use brightdate::{
    format_bd, ticks_to_brightdate, ATTOSECONDS_PER_DAY, BrightDate, BrightInstant,
    ExactBrightAtto, ExactBrightDate,
};

// v1 dashboard
let bd = BrightDate::now();
let inst = BrightInstant::from_brightdate(bd.value).unwrap();
let back = BrightDate::from_value(inst.to_brightdate());

// v2 canonical engine
let atto = ExactBrightAtto::from_unix_ms(1_700_000_000_000);
assert_eq!(atto.encode(), format!("EBA1:{}", atto.attoseconds()));
assert!((atto.to_brightdate() - ticks_to_brightdate(atto.attoseconds(), ATTOSECONDS_PER_DAY)).abs() < 1e-10);

// Bridge: dashboard decimal days match the lens
assert!((BrightDate::from_exact_bright_atto(atto).value - atto.to_brightdate()).abs() < 1e-10);
// Whole-day attosecond ticks round-trip exactly through the f64 bridge
let whole_day = ExactBrightAtto::from_attoseconds(ATTOSECONDS_PER_DAY);
assert_eq!(
    BrightDate::from_exact_bright_atto(whole_day)
        .to_exact_bright_atto()
        .unwrap(),
    whole_day,
);

// Picosecond engine (optional)
let ps = ExactBrightDate::from_unix_ms(1_700_000_000_000);
let from_ps = ExactBrightAtto::from_exact_brightdate(ps);

// Pre-J2000.0 instants: PBD display label (no leading minus in user strings)
let pre = BrightDate::from_value(-11125.154);
assert_eq!(format_bd(pre.value, 3).unwrap(), "PBD 11125.154");
```

---

## Installing

### From crates.io

The `brightdate` library and all five CLI tools are published on [crates.io](https://crates.io/crates/brightdate):

```bash
# Library (for use in Rust projects)
cargo add brightdate

# CLI tools
cargo install bdate     # date(1) replacement
cargo install btime     # time(1) replacement
cargo install buptime   # uptime(1) replacement
cargo install bcal      # cal(1) replacement
cargo install bwatch    # watch(1) replacement
```

### From Homebrew (macOS / Linux)

The CLI tools are available via the [Digital Defiance Homebrew tap](https://github.com/Digital-Defiance/homebrew-tap):

```bash
brew tap digital-defiance/tap
brew install digital-defiance/tap/bdate
brew install digital-defiance/tap/btime
brew install digital-defiance/tap/buptime
brew install digital-defiance/tap/bcal
brew install digital-defiance/tap/bwatch
```

After tapping you can also use the short names: `brew install bdate`, etc.

### Using the Rust library

```toml
# Cargo.toml
[dependencies]
brightdate = "0.5"
```

```rust
use brightdate::{BrightDate, ExactBrightAtto};

let now = BrightDate::now();
println!("{:.5}", now);           // e.g. 9603.57128
println!("{}", now.to_iso());     // 2026-05-11T12:34:56.789Z

let exact = ExactBrightAtto::from_unix_ms(now.to_unix_ms() as i64);
println!("{}", exact.encode());   // EBA1:… attoseconds since J2000.0
```

See the [crates.io docs](https://docs.rs/brightdate) for the full API.

---

## Building from source

### Rust

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

### TypeScript

```bash
cd brightdate
yarn install
yarn build
yarn test
```

### Showcase

```bash
cd brightdate/showcase
yarn install
yarn dev
```

---

## Format characters

BrightDate `%` extensions (`%Wt`, `%Ws`, `%dE`, …) are documented in
[`FORMAT-SPEC.md`](FORMAT-SPEC.md). The `%W*` family matches
[bright-findutils](https://findutils.digitaldefiance.org) `bfind -printf`;
`btime -f` adds timing-only letters and `%d*` milliday durations aligned with
BSH `TIMEFMT`.

---

## License

MIT © Digital Defiance
