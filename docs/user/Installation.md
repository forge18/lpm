# Installation Guide

## Prerequisites

- Lua 5.1, 5.3, or 5.4 installed

## Installation Methods

### Option 1: Pre-built Binaries (Recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/yourusername/lpm/releases):

#### macOS (Apple Silicon)
```bash
curl -L https://github.com/yourusername/lpm/releases/latest/download/lpm-aarch64-apple-darwin.tar.gz | tar xz
sudo mv lpm /usr/local/bin/
```

#### macOS (Intel)
```bash
curl -L https://github.com/yourusername/lpm/releases/latest/download/lpm-x86_64-apple-darwin.tar.gz | tar xz
sudo mv lpm /usr/local/bin/
```

#### Linux (x86_64)
```bash
curl -L https://github.com/yourusername/lpm/releases/latest/download/lpm-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv lpm /usr/local/bin/
```

#### Linux (ARM64)
```bash
curl -L https://github.com/yourusername/lpm/releases/latest/download/lpm-aarch64-unknown-linux-gnu.tar.gz | tar xz
sudo mv lpm /usr/local/bin/
```

#### Windows
1. Download `lpm-x86_64-pc-windows-msvc.zip` from [GitHub Releases](https://github.com/yourusername/lpm/releases/latest)
2. Extract the zip file
3. Add the directory containing `lpm.exe` to your PATH

### Option 2: Build Locally (Requires Rust)

If you have Rust installed ([rustup.rs](https://rustup.rs/)):

```bash
# Clone the repository
git clone https://github.com/yourusername/lpm.git
cd lpm

# Build the release executable
cargo build --release

# The executable will be at: target/release/lpm (or target/release/lpm.exe on Windows)
# Copy it wherever you want:
cp target/release/lpm /usr/local/bin/lpm  # Unix/macOS
# Or on Windows, add target/release/ to your PATH
```

### Option 3: Install via Cargo (Requires Rust)

```bash
# Install from crates.io (when published)
cargo install lpm

# Or install from local source
cargo install --path .
```

## Setup PATH

After installation, ensure `lpm` is in your PATH:

### Unix/macOS/Linux

Run the setup command:
```bash
lpm setup-path
```

Or manually add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Windows

Add `%USERPROFILE%\.cargo\bin` to your PATH:
- Open System Properties → Environment Variables
- Add `%USERPROFILE%\.cargo\bin` to your User PATH
- Or run in PowerShell (as Administrator):
```powershell
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";%USERPROFILE%\.cargo\bin", "User")
```

## Verify Installation

Check that LPM is installed correctly:

```bash
lpm --version
```

You should see the version number. If you get a "command not found" error, make sure:
1. LPM is installed
2. The installation directory is in your PATH
3. You've restarted your terminal (or reloaded your shell profile)

