# BrightDate format characters

Canonical reference for `%` format extensions across the BrightDate ecosystem.
Scalar semantics follow [brightdate-rust](https://github.com/Digital-Defiance/brightdate-rust)
(TAI substrate, J2000.0 epoch, SI days).

## Tier A — `%W`<letter> absolute BrightDate scalar

**Output:** decimal SI days since J2000.0, **`%.9f`**, no unit suffix.

| Specifier | bright-findutils `bfind -printf` | brightdate-rust `btime -f` |
|-----------|----------------------------------|----------------------------|
| `%Wt` | File mtime | Command end wall time |
| `%Ws` | — (not applicable) | Command start wall time |
| `%Wa` | File atime | — |
| `%Wc` | File ctime | — |
| `%WB` | File birth time | — |
| `%W`<other> | Same as `%Wt` (mtime) | GNU swap count (`%W` not followed by `t`/`s`) |

**Color:** `bfind --color` and `btime` default report colorize on TTY; `-printf` / `-f` output is plain text.

**Aliases in `btime -f` only:** `%N` ≡ `%Ws`, `%n` ≡ `%Wt` (same 9 d.p. values).

## Tier B — duration in millidays

| Specifier | Output | `bsh` `TIMEFMT` | `btime -f` |
|-----------|--------|-----------------|------------|
| `%dE` | Elapsed (real) | `%.6f md` | `%.6f md` |
| `%dU` | User CPU | `%.6f md` | `%.6f md` |
| `%dS` | System CPU | `%.6f md` | `%.6f md` |
| `%b` | Elapsed millidays | — | `%.6f` (machine; no suffix) |

Conversion: `millidays = seconds ÷ 86.4`.

## Tier C — `btime -f` timing extensions (GNU `-f` namespace)

These letters reuse GNU `time -f` slots with BrightDate meaning **inside `btime -f` only**.
Do not use in `bfind -printf` (different GNU meanings there).

| Specifier | Meaning | Output |
|-----------|---------|--------|
| `%B` | Elapsed wall time | `{:.9}` BrightDate days |
| `%b` | Elapsed wall time | `{:.6}` millidays (no suffix) |
| `%N` / `%n` | Start / end wall time | `{:.9}` (same as `%Ws` / `%Wt`) |

Standard GNU letters (`%E`, `%e`, `%U`, `%S`, `%P`, `%C`, `%M`, …) behave as GNU time 1.10.

## Tier D — display surfaces (human, not `-printf`)

Fixed formatting, not user format strings:

| Surface | Typical precision | Notes |
|---------|-------------------|-------|
| BSH prompt `%P` | 6 d.p. | Current BrightDate |
| `stat` / `ls -l` | 6 d.p. | File timestamps |
| `btime` default report | 6–9 d.p. | Multi-line, optional color |
| BSH `TIMEFMT` `%d*` | 6 d.p. + ` md` | Shell `time` keyword |

Machine interchange and log correlation should prefer **`%W*` at 9 d.p.**

## Tier E — bright-iputils (suffix notation)

bright-iputils does **not** implement `-printf` specifiers. It uses explicit unit suffixes:

| Suffix | Meaning |
|--------|---------|
| `md` | Millidays |
| `ud` | Microdays |
| `nd` | Nanodays |
| `d` | Days |

Example: `bclockdiff -B` → `115740.740740741ud`. Semantically equivalent to BrightDate sub-day units; different syntax from `%W*`.

## Intentionally not unified

| Pattern | Reason |
|---------|--------|
| `%b` in `bfind` | GNU 512-byte blocks |
| `%B` in `bfind` | GNU birth-time strftime (`%Bk`) |
| `%n` in `bfind` | GNU hard-link count |
| `%*E`, `%m*`, `%u*`, `%n*` in TIMEFMT | zsh/BSH shell `time` keyword only |
| `%b` / `%B` in `bdate --strftime` | POSIX month names (chrono), not millidays |

## References

- Implementation: `crates/btime/src/format.rs` (timing), `bright-findutils/find/print.c` (files)
- BSH TIMEFMT: `bsh/Src/jobs.c`
- Ecosystem docs: [bsh](https://github.com/Digital-Defiance/bsh), [bright-findutils](https://findutils.digitaldefiance.org), [bright-iputils](https://github.com/Digital-Defiance/bright-iputils)
