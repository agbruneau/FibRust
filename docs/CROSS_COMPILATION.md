# Cross-Compilation Guide

This guide covers building FibCalc-rs for all supported target platforms, including native builds, cross-compilation from a different host, static linking with musl, GMP feature considerations, and CI/CD automation.

## Table of Contents

- [Supported Targets](#supported-targets)
- [Prerequisites](#prerequisites)
- [Installing Target Toolchains](#installing-target-toolchains)
- [Native Builds](#native-builds)
- [Cross-Compilation with `cross`](#cross-compilation-with-cross)
- [Linux Cross-Compilation (gnu)](#linux-cross-compilation-gnu)
- [musl Static Linking](#musl-static-linking)
- [Windows MSVC Builds](#windows-msvc-builds)
- [macOS Universal Binaries](#macos-universal-binaries)
- [GMP Feature Challenges](#gmp-feature-challenges)
- [Cargo Configuration Per Target](#cargo-configuration-per-target)
- [Release Build Optimizations](#release-build-optimizations)
- [Testing Cross-Compiled Binaries with QEMU](#testing-cross-compiled-binaries-with-qemu)
- [CI/CD Matrix (GitHub Actions)](#cicd-matrix-github-actions)
- [Troubleshooting](#troubleshooting)

---

## Supported Targets

| Target Triple | Priority | Description |
|---|---|---|
| `x86_64-unknown-linux-gnu` | **P0** | Primary Linux target, dynamically linked against glibc |
| `x86_64-unknown-linux-musl` | P1 | Statically linked Linux binary, fully portable |
| `x86_64-pc-windows-msvc` | P1 | Windows with Microsoft Visual C++ runtime |
| `x86_64-apple-darwin` | P1 | macOS on Intel |
| `aarch64-apple-darwin` | P1 | macOS on Apple Silicon (M1/M2/M3/M4) |

The default build (`cargo build`) uses the host platform. FibCalc-rs is pure Rust by default (no external C dependencies), making cross-compilation straightforward for the default feature set.

## Prerequisites

### Rust Toolchain

Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify the minimum supported Rust version (MSRV 1.80+):

```bash
rustc --version
# rustc 1.80.0 or later
```

### System Dependencies

**Default build (pure Rust):** No external dependencies required. The `num-bigint` crate provides big number arithmetic in pure Rust.

**With `gmp` feature:** Requires libgmp development headers for each target platform. See [GMP Feature Challenges](#gmp-feature-challenges).

### Cross-Compilation Tooling

For Docker-based cross-compilation (recommended for Linux targets from any host):

```bash
cargo install cross
```

`cross` requires Docker or Podman to be installed and running.

## Installing Target Toolchains

Add each desired target via `rustup`:

```bash
# P0 - Primary Linux
rustup target add x86_64-unknown-linux-gnu

# P1 - Linux static (musl)
rustup target add x86_64-unknown-linux-musl

# P1 - Windows
rustup target add x86_64-pc-windows-msvc

# P1 - macOS Intel
rustup target add x86_64-apple-darwin

# P1 - macOS Apple Silicon
rustup target add aarch64-apple-darwin
```

Add all targets at once:

```bash
rustup target add \
  x86_64-unknown-linux-gnu \
  x86_64-unknown-linux-musl \
  x86_64-pc-windows-msvc \
  x86_64-apple-darwin \
  aarch64-apple-darwin
```

List installed targets:

```bash
rustup target list --installed
```

## Native Builds

For native builds on the host platform, no cross-compilation setup is needed:

```bash
# Debug build
cargo build

# Release build with full optimizations
cargo build --release

# Release build with GMP support
cargo build --release --features gmp
```

The workspace `Cargo.toml` defines the following release profile:

```toml
[profile.release]
lto = true           # Link-time optimization (full)
codegen-units = 1    # Single codegen unit for maximum optimization
strip = true         # Strip debug symbols from binary
opt-level = 3        # Maximum optimization level
panic = "abort"      # Abort on panic (smaller binary, no unwinding)
```

The `.cargo/config.toml` sets `target-cpu=native` by default:

```toml
[build]
rustflags = ["-C", "target-cpu=native"]
```

> **Note:** The `target-cpu=native` flag optimizes for the host CPU's instruction set. When cross-compiling, you should override this to avoid generating instructions that the target CPU does not support. See [Cargo Configuration Per Target](#cargo-configuration-per-target).

## Cross-Compilation with `cross`

The [`cross`](https://github.com/cross-rs/cross) tool uses Docker containers with pre-configured toolchains, making cross-compilation reliable and reproducible. This is the recommended approach for building Linux targets from any host OS.

### Installation

```bash
cargo install cross
```

### Usage

`cross` is a drop-in replacement for `cargo`:

```bash
# Build for Linux (glibc)
cross build --release --target x86_64-unknown-linux-gnu

# Build for Linux (musl, static)
cross build --release --target x86_64-unknown-linux-musl

# Run tests under emulation
cross test --target x86_64-unknown-linux-gnu
```

### Custom Docker Image (for GMP)

If you need the `gmp` feature, create a `Cross.toml` in the project root:

```toml
[target.x86_64-unknown-linux-gnu]
image = "ghcr.io/cross-rs/x86_64-unknown-linux-gnu:main"
pre-build = [
    "apt-get update && apt-get install -y libgmp-dev"
]

[target.x86_64-unknown-linux-musl]
image = "ghcr.io/cross-rs/x86_64-unknown-linux-musl:main"
pre-build = [
    "apt-get update && apt-get install -y libgmp-dev"
]
```

Then build with:

```bash
cross build --release --features gmp --target x86_64-unknown-linux-gnu
```

## Linux Cross-Compilation (gnu)

### From macOS

To cross-compile for `x86_64-unknown-linux-gnu` from macOS, use `cross` (recommended) or install a GCC cross-toolchain:

**Using `cross` (recommended):**

```bash
cross build --release --target x86_64-unknown-linux-gnu
```

**Using a manual toolchain:**

```bash
# Install the cross-compiler
brew install FiloSottile/musl-cross/musl-cross

# Or for glibc targets, use a Linux GCC toolchain
brew tap messense/macos-cross-toolchains
brew install x86_64-unknown-linux-gnu
```

Set the linker in `.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "x86_64-unknown-linux-gnu-gcc"
```

Then:

```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

### From Windows

Cross-compiling for Linux from Windows is best done via `cross` with Docker Desktop, or through WSL2:

**Using WSL2:**

```bash
# Inside a WSL2 Ubuntu instance
sudo apt update && sudo apt install -y build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo build --release
```

**Using `cross` with Docker Desktop:**

```powershell
cross build --release --target x86_64-unknown-linux-gnu
```

## musl Static Linking

Building with the `x86_64-unknown-linux-musl` target produces a fully statically linked binary with no runtime dependencies. This is ideal for deployment in containers (e.g., `FROM scratch`) or minimal environments.

### From Linux

```bash
# Install musl tools
sudo apt install -y musl-tools

# Build
cargo build --release --target x86_64-unknown-linux-musl
```

### From macOS

```bash
# Using cross (recommended)
cross build --release --target x86_64-unknown-linux-musl

# Or install musl-cross manually
brew install FiloSottile/musl-cross/musl-cross
```

Set the linker:

```toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
```

### Verifying Static Linking

```bash
# Verify the binary is statically linked
file target/x86_64-unknown-linux-musl/release/fibcalc
# Expected: ELF 64-bit LSB executable, x86-64, statically linked

ldd target/x86_64-unknown-linux-musl/release/fibcalc
# Expected: "not a dynamic executable"
```

### Minimal Docker Image

```dockerfile
FROM scratch
COPY target/x86_64-unknown-linux-musl/release/fibcalc /fibcalc
ENTRYPOINT ["/fibcalc"]
```

## Windows MSVC Builds

### Native Build on Windows

Building natively on Windows with MSVC is the simplest approach:

**Requirements:**
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) or full Visual Studio
- "Desktop development with C++" workload selected during installation
- Rust installed via `rustup` (the `x86_64-pc-windows-msvc` target is the default on Windows)

```powershell
# Build (MSVC is the default target on Windows)
cargo build --release

# Explicit target
cargo build --release --target x86_64-pc-windows-msvc
```

### Cross-Compiling for Windows from Linux

Cross-compiling for Windows MSVC from Linux is not directly supported since the MSVC linker is Windows-only. Alternatives:

1. **Use `x86_64-pc-windows-gnu`** (MinGW target, not an official project target but functional):

   ```bash
   sudo apt install -y gcc-mingw-w64-x86-64
   rustup target add x86_64-pc-windows-gnu
   cargo build --release --target x86_64-pc-windows-gnu
   ```

2. **Use CI/CD** with a Windows runner for MSVC builds (see [CI/CD Matrix](#cicd-matrix-github-actions)).

3. **Use `cross`** with an experimental Windows container (limited support).

## macOS Universal Binaries

macOS supports Universal Binaries (fat binaries) that contain code for both Intel (`x86_64`) and Apple Silicon (`aarch64`) architectures.

### Building Both Architectures

```bash
# Build for Intel
cargo build --release --target x86_64-apple-darwin

# Build for Apple Silicon
cargo build --release --target aarch64-apple-darwin
```

### Creating a Universal Binary with `lipo`

```bash
# Combine into a universal binary
lipo -create \
  target/x86_64-apple-darwin/release/fibcalc \
  target/aarch64-apple-darwin/release/fibcalc \
  -output target/fibcalc-universal

# Verify
lipo -info target/fibcalc-universal
# Expected: Architectures in the fat file: x86_64 arm64
```

### Cross-Compiling Between macOS Architectures

On Apple Silicon Macs, you can cross-compile for Intel:

```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

On Intel Macs, you can cross-compile for Apple Silicon:

```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

Both directions work natively on macOS because Apple provides universal SDKs. No additional linker or toolchain setup is needed.

## GMP Feature Challenges

The `gmp` feature enables GMP-based big number arithmetic via the `rug` crate (LGPL licensed). This introduces native C library dependencies that complicate cross-compilation.

### The Problem

- `rug` depends on `gmp-mpfr-sys`, which builds GMP from source or links to a system-installed libgmp
- Each target requires a compatible C compiler and GMP build for that target's architecture and OS
- Static linking of GMP requires the static library (`libgmp.a`) built for the target
- LGPL licensing requires dynamic linking or providing the ability for users to re-link

### Per-Target GMP Setup

**Linux (native):**

```bash
sudo apt install -y libgmp-dev
cargo build --release --features gmp
```

**Linux (musl, static):**

```bash
# Build a static GMP for musl
sudo apt install -y musl-tools
wget https://gmplib.org/download/gmp/gmp-6.3.0.tar.xz
tar xf gmp-6.3.0.tar.xz && cd gmp-6.3.0
CC=musl-gcc ./configure --prefix=/usr/local/musl --enable-static --disable-shared --host=x86_64-linux-musl
make -j$(nproc) && sudo make install

# Build FibCalc with GMP
GMP_DIR=/usr/local/musl cargo build --release --features gmp --target x86_64-unknown-linux-musl
```

**macOS:**

```bash
brew install gmp
cargo build --release --features gmp
```

**Windows (MSVC):**

GMP on Windows MSVC is challenging. The `gmp-mpfr-sys` crate will attempt to build GMP from source using MSYS2/MinGW:

```powershell
# Install MSYS2, then in an MSYS2 shell:
pacman -S mingw-w64-x86_64-gmp

# Set environment for rug/gmp-mpfr-sys
$env:GMP_DIR = "C:\msys64\mingw64"
cargo build --release --features gmp
```

### Recommendation

For maximum portability, ship the **default build** (pure Rust, `num-bigint`) for all platforms. Offer the `gmp` feature only for Linux x86_64 where GMP is readily available and performance gains are most significant.

## Cargo Configuration Per Target

The `.cargo/config.toml` file supports per-target configuration. The default project configuration uses `target-cpu=native`, which must be overridden for cross-compilation.

### Recommended Configuration

```toml
[build]
# Default: optimize for host CPU
rustflags = ["-C", "target-cpu=native"]

# Linux glibc
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "target-cpu=x86-64-v2"]
# linker = "x86_64-linux-gnu-gcc"  # Uncomment for cross-compilation

# Linux musl (static)
[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "target-cpu=x86-64-v2", "-C", "target-feature=+crt-static"]
# linker = "x86_64-linux-musl-gcc"  # Uncomment for cross-compilation

# Windows MSVC
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-cpu=x86-64-v2"]

# macOS Intel
[target.x86_64-apple-darwin]
rustflags = ["-C", "target-cpu=x86-64-v2"]

# macOS Apple Silicon
[target.aarch64-apple-darwin]
rustflags = ["-C", "target-cpu=apple-m1"]
```

### CPU Microarchitecture Levels

| Level | Instructions | Recommended For |
|---|---|---|
| `x86-64` | Baseline SSE2 | Maximum compatibility |
| `x86-64-v2` | SSE4.2, POPCNT | General-purpose builds (recommended) |
| `x86-64-v3` | AVX2, BMI2 | Performance-optimized builds (2015+ CPUs) |
| `x86-64-v4` | AVX-512 | Server/HPC builds (limited CPU support) |
| `apple-m1` | NEON, AES | Apple Silicon Macs |
| `native` | Auto-detect host | Local development only |

## Release Build Optimizations

The workspace release profile is already configured for maximum performance:

```toml
[profile.release]
lto = true           # Full link-time optimization
codegen-units = 1    # Single codegen unit (slower compilation, faster binary)
strip = true         # Remove debug symbols
opt-level = 3        # Maximum optimization
panic = "abort"      # No unwinding overhead
```

### Per-Platform Considerations

**Linux:**
- LTO works well with both `gcc` and `lld` linkers
- Consider using `mold` for faster link times during development: `rustflags = ["-C", "link-arg=-fuse-ld=mold"]`

**macOS:**
- LTO is fully supported with the default Apple linker
- `strip = true` uses `strip -x` behavior on macOS

**Windows MSVC:**
- LTO uses MSVC's LTCG (Link-Time Code Generation)
- `strip = true` removes PDB debug info from the binary
- PDB files are still generated alongside the binary for debugging

### Binary Size Comparison

Approximate release binary sizes (default features, no GMP):

| Target | Approximate Size |
|---|---|
| `x86_64-unknown-linux-gnu` | ~2-4 MB |
| `x86_64-unknown-linux-musl` | ~3-5 MB |
| `x86_64-pc-windows-msvc` | ~2-4 MB |
| `x86_64-apple-darwin` | ~2-4 MB |
| `aarch64-apple-darwin` | ~2-4 MB |
| Universal macOS binary | ~4-8 MB |

## Testing Cross-Compiled Binaries with QEMU

QEMU user-mode emulation allows running cross-compiled binaries on the host system without a full VM.

### Setup

```bash
# Install QEMU user-mode emulation (Ubuntu/Debian)
sudo apt install -y qemu-user qemu-user-static binfmt-support

# Verify
qemu-x86_64 --version
```

### Running Tests with QEMU

For Linux targets built on a different architecture (e.g., building aarch64 on x86_64):

```bash
# Add the aarch64 target
rustup target add aarch64-unknown-linux-gnu
sudo apt install -y gcc-aarch64-linux-gnu

# Set linker
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

# Run tests under QEMU emulation
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER=qemu-aarch64

cargo test --target aarch64-unknown-linux-gnu
```

### Using `cross` for Emulated Testing

`cross` handles QEMU setup automatically:

```bash
# Tests run under emulation transparently
cross test --target aarch64-unknown-linux-gnu
cross test --target x86_64-unknown-linux-musl
```

## CI/CD Matrix (GitHub Actions)

The following GitHub Actions workflow builds and tests FibCalc-rs across all supported targets:

```yaml
name: Cross-Platform Build

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          # P0 - Primary Linux
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            cross: false

          # P1 - Linux musl (static)
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            cross: true

          # P1 - Windows MSVC
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            cross: false

          # P1 - macOS Intel
          - target: x86_64-apple-darwin
            os: macos-13
            cross: false

          # P1 - macOS Apple Silicon
          - target: aarch64-apple-darwin
            os: macos-latest
            cross: false

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross
        if: matrix.cross
        run: cargo install cross

      - name: Install musl-tools (Linux musl)
        if: matrix.target == 'x86_64-unknown-linux-musl' && !matrix.cross
        run: sudo apt-get update && sudo apt-get install -y musl-tools

      - name: Cache cargo registry and build
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-${{ matrix.target }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-${{ matrix.target }}-cargo-

      - name: Override target-cpu for cross builds
        run: |
          mkdir -p .cargo
          echo '[build]' > .cargo/config.toml
          echo 'rustflags = ["-C", "target-cpu=x86-64-v2"]' >> .cargo/config.toml
        shell: bash

      - name: Build (cross)
        if: matrix.cross
        run: cross build --release --target ${{ matrix.target }}

      - name: Build (cargo)
        if: "!matrix.cross"
        run: cargo build --release --target ${{ matrix.target }}

      - name: Test (cross)
        if: matrix.cross
        run: cross test --target ${{ matrix.target }}

      - name: Test (cargo)
        if: "!matrix.cross"
        run: cargo test --target ${{ matrix.target }}

      - name: Upload binary artifact
        uses: actions/upload-artifact@v4
        with:
          name: fibcalc-${{ matrix.target }}
          path: |
            target/${{ matrix.target }}/release/fibcalc
            target/${{ matrix.target }}/release/fibcalc.exe

  # macOS Universal Binary
  universal-macos:
    needs: build
    runs-on: macos-latest
    steps:
      - name: Download Intel binary
        uses: actions/download-artifact@v4
        with:
          name: fibcalc-x86_64-apple-darwin
          path: x86_64

      - name: Download ARM binary
        uses: actions/download-artifact@v4
        with:
          name: fibcalc-aarch64-apple-darwin
          path: aarch64

      - name: Create universal binary
        run: |
          chmod +x x86_64/fibcalc aarch64/fibcalc
          lipo -create x86_64/fibcalc aarch64/fibcalc -output fibcalc-universal
          lipo -info fibcalc-universal

      - name: Upload universal binary
        uses: actions/upload-artifact@v4
        with:
          name: fibcalc-universal-macos
          path: fibcalc-universal

  # Release job (only on tags)
  release:
    if: startsWith(github.ref, 'refs/tags/v')
    needs: [build, universal-macos]
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/download-artifact@v4

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: |
            fibcalc-x86_64-unknown-linux-gnu/fibcalc
            fibcalc-x86_64-unknown-linux-musl/fibcalc
            fibcalc-x86_64-pc-windows-msvc/fibcalc.exe
            fibcalc-universal-macos/fibcalc-universal
```

## Troubleshooting

### `target-cpu=native` Errors During Cross-Compilation

**Symptom:** Build errors about unsupported CPU features or illegal instructions at runtime.

**Cause:** The default `.cargo/config.toml` sets `rustflags = ["-C", "target-cpu=native"]`, which optimizes for the host CPU.

**Fix:** Override for the specific target:

```bash
# Override via environment variable
RUSTFLAGS="-C target-cpu=x86-64-v2" cargo build --release --target x86_64-unknown-linux-gnu

# Or add a target-specific section to .cargo/config.toml (see above)
```

### Linker Not Found

**Symptom:** `error: linker 'cc' not found` or `error: linker 'x86_64-linux-gnu-gcc' not found`

**Fix:** Install the appropriate cross-compilation toolchain:

```bash
# Ubuntu/Debian for Linux GNU targets
sudo apt install -y gcc-x86-64-linux-gnu

# For musl targets
sudo apt install -y musl-tools

# macOS for Linux targets
brew tap messense/macos-cross-toolchains
brew install x86_64-unknown-linux-gnu
```

Or use `cross` to avoid managing toolchains manually.

### Missing `ring` or OpenSSL Symbols

**Symptom:** Build failures mentioning `ring`, `openssl-sys`, or missing symbols.

**Cause:** Some transitive dependencies may require a C compiler for the target.

**Fix:**

```bash
# Set the C compiler for the target
export CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc
export AR_x86_64_unknown_linux_gnu=x86_64-linux-gnu-ar

# Or use cross
cross build --release --target x86_64-unknown-linux-gnu
```

### musl Build Fails with `undefined reference`

**Symptom:** Linker errors about missing symbols when building for musl.

**Fix:** Ensure `musl-tools` is installed and the correct linker is configured:

```bash
sudo apt install -y musl-tools

# Verify musl-gcc is available
musl-gcc --version
```

If linking against system libraries (e.g., GMP), they must be compiled for musl. See [GMP Feature Challenges](#gmp-feature-challenges).

### `rug`/GMP Compilation Failures

**Symptom:** `gmp-mpfr-sys` fails to build GMP from source during cross-compilation.

**Fix:**

1. Pre-install a cross-compiled libgmp for the target
2. Set the `GMP_DIR` environment variable:

   ```bash
   export GMP_DIR=/path/to/cross-compiled/gmp
   ```

3. Or build without GMP (the default `num-bigint` backend has no native dependencies):

   ```bash
   cargo build --release --target x86_64-unknown-linux-gnu
   # No --features gmp
   ```

### Windows: `link.exe` Not Found

**Symptom:** `error: linker 'link.exe' not found` on Windows.

**Fix:** Install Visual Studio Build Tools with the "Desktop development with C++" workload:

1. Download [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. In the installer, select "Desktop development with C++"
3. Ensure "MSVC v143 - VS 2022 C++ x64/x86 build tools" is checked
4. Restart your terminal after installation

### macOS: `lipo` Fails with Architecture Mismatch

**Symptom:** `lipo: can't figure out the architecture type` or architecture conflicts.

**Fix:** Ensure both binaries were built for the correct architectures:

```bash
# Check architectures
file target/x86_64-apple-darwin/release/fibcalc
file target/aarch64-apple-darwin/release/fibcalc

# Clean and rebuild both targets
cargo clean
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

### Cargo Cache Conflicts Between Targets

**Symptom:** Unexpected recompilation or stale artifacts when switching between targets.

**Fix:** Use `--target` explicitly (builds go to `target/<triple>/`) rather than relying on the default target directory. If issues persist:

```bash
# Clean a specific target
cargo clean --target x86_64-unknown-linux-gnu

# Or clean everything
cargo clean
```

### Binary Size Too Large

**Symptom:** Release binary is larger than expected.

**Fix:** Verify the release profile is active and consider additional stripping:

```bash
# Build with release profile
cargo build --release --target x86_64-unknown-linux-gnu

# Additional stripping (Linux/macOS)
strip target/x86_64-unknown-linux-gnu/release/fibcalc

# Check binary size
ls -lh target/x86_64-unknown-linux-gnu/release/fibcalc
```

The workspace release profile already includes `strip = true` and `lto = true`. If the binary is still large, check for debug symbols in dependencies:

```bash
# Audit binary sections
objdump -h target/x86_64-unknown-linux-gnu/release/fibcalc | head -20
```
