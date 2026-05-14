# [BrightDate](https://brightdate.org) — Workspace

> *Named in homage to Star Trek's Stardate — one universal scalar to rule them all.*

This workspace contains the BrightDate v1.0 libraries: a scientifically grounded, timezone-free decimal day representation anchored at **J2000.0** via a **TAI substrate**.

---

## Crates

| Crate | Path | Description |
|-------|------|-------------|
| `brightdate` | `brightdate/` (Rust) | Core BrightDate type, TAI conversions, leap second table |
| `bdate` | `bdate/` | Date formatting and calendar utilities |
| `btime` | `btime/` | Time-of-day utilities |
| `buptime` | `buptime/` | Process uptime tracking |
| `bcal` | `bcal/` | Calendar arithmetic |
| `bwatch` | `bwatch/` | Stopwatch / interval utilities |

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

BrightDate ships in three flavors so you can pick the right trade-off between ergonomics and precision. All three anchor on the same J2000.0 / TAI substrate — they only differ in how the value is stored.

| Type | Representation | Precision | Range | Use when… |
|------|----------------|-----------|-------|-----------|
| **`BrightDate`** | `f64` decimal days since J2000.0 | ~190 ns in the current era; widens with magnitude | ±287,000 years from J2000 | The 99% case — math, astronomy, scheduling, logging, display. Sorts, diffs, and serializes natively. |
| **`BrightInstant`** | `i64` TAI seconds + `u32` nanos since J2000.0 | **1 ns exactly, everywhere** | Effectively unlimited | You need nanosecond precision at any magnitude — distributed systems, GPS engineering, interplanetary mission timing. The rigorous companion to `BrightDate`. |
| **`ExactBrightDate`** *(TypeScript only)* | `BigInt` picoseconds since J2000.0 | **1 ps exactly, everywhere** | Effectively unlimited | You must round-trip arbitrary Unix milliseconds bit-for-bit — blockchain consensus, archival storage, byte-identical reconstruction. |

**Rust** ships `BrightDate` + `BrightInstant`. **TypeScript** ships all three. You can convert freely between them: store in the exact form at storage boundaries, compute in `BrightDate` for speed and convenience.

```rust
use brightdate::{BrightDate, BrightInstant};

let bd  = BrightDate::now();           // f64 ergonomic form
let inst = BrightInstant::from(bd);    // exact ns-precision form
let back: BrightDate = inst.into();    // round-trips for the f64 range
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
brightdate = "0.1"
```

```rust
use brightdate::BrightDate;

let now = BrightDate::now();
println!("{:.5}", now);                  // e.g. 9603.57128
println!("{}", now.to_iso8601());        // 2026-05-11T12:34:56.789Z
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

## License

MIT © Digital Defiance
