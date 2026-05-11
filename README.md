# BrightDate — Workspace

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

## Building

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
