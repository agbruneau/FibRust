# Installation Guide

## Prerequisites

- **Rust 1.80+** (MSRV). Verify with:

  ```bash
  rustc --version   # Must show 1.80.0 or higher
  cargo --version
  ```

  Install Rust via [rustup](https://rustup.rs/) if needed:

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

## Quick Install (Pure Rust)

The default build has **zero system dependencies** beyond the Rust toolchain:

```bash
git clone https://github.com/agbruneau/FibRust.git
cd FibRust
cargo install --path crates/fibcalc
```

The `fibcalc` binary is now in `~/.cargo/bin/` (or `%USERPROFILE%\.cargo\bin\` on Windows).

Alternatively, build without installing:

```bash
cargo build --release
# Binary: target/release/fibcalc (or fibcalc.exe on Windows)
```

## With GMP Support (Optional)

GMP (GNU Multiple Precision Arithmetic Library) provides faster big-integer
arithmetic through the `rug` crate. This is **optional** -- the pure-Rust
default works on all platforms without system dependencies.

### Install libgmp

**Linux (Debian/Ubuntu):**

```bash
sudo apt-get update
sudo apt-get install libgmp-dev
```

**Linux (Fedora/RHEL):**

```bash
sudo dnf install gmp-devel
```

**macOS (Homebrew):**

```bash
brew install gmp
```

**Windows:**

GMP on Windows requires MSYS2 or a prebuilt GMP library. For simplicity, the
pure-Rust default is recommended on Windows.

### Build with GMP

```bash
cargo install --path crates/fibcalc --features gmp
```

Or without installing:

```bash
cargo build --release --features gmp
```

> **Note**: The `gmp` feature links against libgmp (LGPL). When using
> `--features gmp`, the combined work must comply with LGPL terms.

## Platform-Specific Notes

### Windows

Pure Rust works out of the box with no extra dependencies:

```bash
cargo build --release
```

The release binary is at `target\release\fibcalc.exe`.

### Linux

Pure Rust works out of the box. For GMP support, install `libgmp-dev` (see
above).

The release build uses `target-cpu=native` via `.cargo/config.toml` for maximum
performance on the local CPU.

### macOS

Pure Rust works out of the box. For GMP support, install `gmp` via Homebrew
(see above).

Supported architectures: `x86_64-apple-darwin` and `aarch64-apple-darwin`
(Apple Silicon).

## Docker

Minimal multi-stage Dockerfile for a statically linked binary:

```dockerfile
# Build stage
FROM rust:1.80 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked

# Runtime stage
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/fibcalc /usr/local/bin/
ENTRYPOINT ["fibcalc"]
```

Build and run:

```bash
docker build -t fibcalc .
docker run --rm fibcalc -n 1000 -c
```

For GMP support in Docker, add `libgmp-dev` to the builder and `libgmp10` to
the runtime:

```dockerfile
FROM rust:1.80 AS builder
RUN apt-get update && apt-get install -y libgmp-dev
WORKDIR /app
COPY . .
RUN cargo build --release --locked --features gmp

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libgmp10 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/fibcalc /usr/local/bin/
ENTRYPOINT ["fibcalc"]
```

## Troubleshooting

### GMP not found

If you get linker errors mentioning `gmp`:

```
error: could not find native static library `gmp`
```

Ensure libgmp is installed (see platform-specific instructions above). On Linux,
verify with:

```bash
dpkg -l | grep libgmp-dev   # Debian/Ubuntu
rpm -q gmp-devel             # Fedora/RHEL
```

### MSRV too old

If you see errors about unsupported Rust features:

```
error[E0658]: use of unstable library feature
```

Update your toolchain:

```bash
rustup update stable
rustc --version   # Must be 1.80+
```

### `target-cpu=native` on CI

The `.cargo/config.toml` sets `rustflags = ["-C", "target-cpu=native"]` for
optimal local performance. On CI runners or when cross-compiling, this may cause
issues.

**Fix**: Override the flag in CI:

```bash
RUSTFLAGS="" cargo build --release
```

Or remove the line from `.cargo/config.toml` before cross-compiling.

See [CROSS_COMPILATION.md](CROSS_COMPILATION.md) for detailed cross-compilation
instructions.

### Build takes too long

The release profile uses LTO and single codegen unit for maximum performance,
which increases build time. For development, use debug builds:

```bash
cargo build          # Debug build (fast compile)
cargo run -- -n 100 -c  # Run in debug mode
```
