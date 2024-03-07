# Rust Native Addin

This directory contains the Rust code for the native addin, as well as support files for building.

## Building Using Docker

The most straightforward way to build for both Windows and Linux is to use a local Docker (or compatible) runtime. With one installed, run the "scripts/build-docker.sh" script from a Linux/macOS shell (or a WSL prompt) or "scripts\build-docker.bat" from a Windows command prompt.

Once run, the built executables will be:

- Linux: target/x86_64-unknown-linux-gnu/release/rustaddin
- Windows: target/x86_64-pc-windows-gnu/release/rustaddin.exe

## Building Locally

### Environment Setup

#### Windows

Currently, Windows can compile for itself, but cross-compilation for Linux has not been figured out.

To install Rust, use the 64-bit Rustup installer from [https://www.rust-lang.org/learn/get-started](https://www.rust-lang.org/learn/get-started). During its installation, it will guide you through installing the Visual Studio compiler if needed, or you can install it separately.

#### Linux

To install Rust and its associated tools for Linux, use the install script from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

Additionally, install cross-compilation libraries for Windows. This command will differ based on your distribution's package manager; on Debian-derived systems like Ubuntu, run:

```
sudo apt install -y g++-mingw-w64-x86-64 gcc-x86-64-linux-gnu
```

#### macOS

Though macOS doesn't have Domino to run the addin, it is able to cross-compile for both Windows and Linux.

Though Homebrew has an installer for Rust, I found it preferable to use the install script from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install), as it contains related tools.

To install the Windows and Linux compilation toolchains, install [Homebrew](https://brew.sh/) and run:

```
brew install llvm mingw-w64 SergioBenitez/osxct/x86_64-unknown-linux-gnu
```

This may also prompt you to install the Xcode Command Line tools, which are required.

### Compiling

Regardless of platform, run `rustup toolchain install stable` to install the native tools, which `rustup` looks for even when compiling for another platform.

#### Compiling For The Current Platform

When building on Windows for Windows or Linux for Linux, you can compile using the command:

```
rustup run stable cargo rustc --release
```

The result binary will be in "target/release".

#### Cross-Compiling For Windows

When building for Windows, this requires the MinGW toolchain on macOS or Linux, installed as above.

First, run this once:

```
rustup target add x86_64-pc-windows-gnu --toolchain stable
```

The Windows executable can then be built via:

```
rustup run stable cargo rustc --release --target=x86_64-pc-windows-gnu -- -C linker=x86_64-w64-mingw32-gcc
```

The executable will be in `target/x86_64-pc-windows-gnu/release`.

#### Compiling for Linux

This can be built on at least macOS and Linux for Linux/x64.

First, run this once:

```
rustup target add x86_64-unknown-linux-gnu --toolchain stable
```

Then, you can build the executable via:

```
rustup run stable cargo rustc --release --target=x86_64-unknown-linux-gnu -- -C linker=x86_64-unknown-linux-gnu-gcc
```

#### Compiling for macOS

Though mostly a curiosity, since there's no Domino server on macOS, the executable can be built for that platform similarly to above. The primary consideration is that it's important to "cross-compile" for x64 when running on an ARM Mac, as libnotes.dylib is not yet available for ARM.

Similar to above, run this once:

```
rustup target add x86_64-apple-darwin --toolchain stable
```

Then, you can build the executable via:

```
rustup run stable cargo rustc --release --target=x86_64-apple-darwin
```