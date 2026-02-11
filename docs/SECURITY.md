# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | Yes                |
| < 0.1   | No                 |

Only the latest release in each supported version line receives security updates.

## Scope

FibCalc-rs is a **local computation library and CLI tool**. It does not listen on network ports, handle authentication, or process untrusted remote input. Its security surface is limited to:

- **Input validation**: Fibonacci index values passed via CLI arguments or TUI input.
- **File I/O**: Reading/writing calibration profiles (`~/.fibcalc_calibration.json`) and configuration files.
- **Dependency supply chain**: Third-party crate vulnerabilities.
- **Memory safety**: Correctness of `unsafe` code blocks.

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, please report vulnerabilities by email:

- **Email**: [security@fibcalc.example.com](mailto:security@fibcalc.example.com)
- **Subject line**: `[SECURITY] Brief description of the issue`

Include as much of the following information as possible:

- Description of the vulnerability
- Steps to reproduce
- Affected versions
- Potential impact
- Suggested fix (if any)

### Response Timeline

| Action                          | Timeframe       |
|---------------------------------|-----------------|
| Acknowledgment of report        | Within 48 hours |
| Initial assessment and triage   | Within 7 days   |
| Fix development and testing     | Within 90 days  |
| Public disclosure (coordinated) | After fix ships  |

We follow a coordinated disclosure process. We ask that reporters refrain from public disclosure until a fix is available and a new release has been published.

## Security Measures

### Dependency Auditing

We use multiple tools to continuously monitor our dependency chain:

- **`cargo audit`** -- Checks dependencies against the [RustSec Advisory Database](https://rustsec.org/) for known vulnerabilities.
- **`cargo deny`** -- Enforces policies defined in [`deny.toml`](../deny.toml):
  - **Advisories**: Known vulnerabilities are denied; unmaintained, yanked, and notice-level advisories produce warnings.
  - **Licenses**: Unlicensed crates are denied. Allowed licenses: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, Zlib, BSL-1.0, CC0-1.0, MPL-2.0, and others. Copyleft licenses produce warnings.
  - **Bans**: Multiple versions of the same crate produce warnings.
  - **Sources**: Only crates from the official crates.io registry are allowed. Unknown registries and git sources produce warnings.

### License Compliance

The project is licensed under **Apache-2.0**. All dependencies must have compatible licenses as enforced by `cargo deny`. The optional `gmp` feature pulls in `rug` (LGPL), which is clearly documented and gated behind a non-default feature flag.

### Unsafe Code Policy

- **Maximum 5 `unsafe` blocks** across the entire codebase.
- Every `unsafe` block must include a `// SAFETY:` comment explaining why the invariants are upheld.
- **`cargo geiger`** is used to audit unsafe usage in the dependency tree.
- Unsafe code is only permitted where it provides measurable performance benefits (e.g., SIMD intrinsics, FFT hot paths) and where safe alternatives would impose unacceptable overhead.

### Fuzz Testing

Fuzz targets are maintained in `fuzz/fuzz_targets/` and exercised with `cargo fuzz`. This helps discover edge cases, panics, and potential memory safety issues in algorithm implementations.

### Build Hardening

Release builds use the following `Cargo.toml` profile settings:

- `lto = true` -- Link-time optimization for dead code elimination.
- `codegen-units = 1` -- Single codegen unit for maximum optimization.
- `strip = true` -- Strip debug symbols from the binary.
- `panic = "abort"` -- Abort on panic instead of unwinding, reducing attack surface.

## Supply Chain Security

### Registry Restrictions

As configured in `deny.toml`, only crates from the [official crates.io index](https://github.com/rust-lang/crates.io-index) are permitted. Git dependencies and unknown registries produce warnings and require explicit approval.

### Dependency Minimization

The project uses a curated set of well-maintained, widely-used crates (see the dependency table in [CLAUDE.md](../CLAUDE.md)). New dependencies are reviewed for:

- License compatibility
- Maintenance status (active maintainers, recent releases)
- Advisory history
- Transitive dependency footprint

### Lockfile

`Cargo.lock` is committed to the repository to ensure reproducible builds and prevent supply chain attacks through dependency resolution changes.

## Secure Development Practices

- **Strict linting**: `cargo clippy -- -W clippy::pedantic` with zero warnings as the target.
- **Formatting**: Enforced via `cargo fmt --check`.
- **Testing**: Unit tests, golden file tests, end-to-end tests, property-based tests (proptest), and fuzz testing.
- **Code coverage**: Target of >75% via `cargo tarpaulin`.
- **Error handling**: Typed errors via `thiserror` in library code; `anyhow` at the application boundary. No `unwrap()` on user-facing paths.

## Known Limitations

- FibCalc-rs performs **arbitrary-precision arithmetic** and can allocate significant memory for very large Fibonacci indices. There is no hard memory cap by default; users should apply OS-level resource limits if needed.
- The calibration file (`~/.fibcalc_calibration.json`) is written with default file permissions. Users in shared environments should verify permissions are appropriate.
- When built with the `gmp` feature, the binary links against `libgmp`, a C library. Vulnerabilities in `libgmp` would affect FibCalc-rs. The default pure-Rust build (`num-bigint`) avoids this external dependency.
