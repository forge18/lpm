# LPM - Lua Package Manager

**Local package management for Lua. Because global installs are legacy.**

## Overview

LPM provides local, project-scoped package management for Lua, similar to npm, cargo, or bundler. It solves the problem of global package installations that cause dependency conflicts and make CI/CD difficult.

## Features

- **Local installation** - Dependencies install to `./lua_modules/`, not globally
- **Lua version manager** - Manage multiple Lua versions (5.1, 5.3, 5.4) with `lpm lua`
- **Global tool installation** - Install dev tools globally with `lpm install -g` (like npm)
- **Lockfile support** - Reproducible builds with `package.lock`
- **SemVer version resolution** - Proper dependency conflict resolution
- **LuaRocks compatible** - Uses LuaRocks as upstream package source
- **Rust extensions** - Build native Lua modules with Rust using Zig cross-compilation (macOS/Linux) or cargo-xwin (Windows)
- **Supply chain security** - Strong checksums, no postinstall scripts, sandboxed builds

## Supported Build Types

LPM supports all LuaRocks build types:

### Build Type Support

- **`builtin`** / **`none`**: Pure Lua modules - files copied directly, no compilation needed
- **`make`**: Builds from source using `make` - requires Makefile in package
- **`cmake`**: Builds from source using `cmake` - requires CMakeLists.txt in package
- **`command`**: Builds from source using custom command (e.g., build script)
- **`rust`** / **`rust-mlua`**: Builds Rust extensions using `cargo` - requires Cargo.toml and Rust toolchain

**What this means:**
- ✅ Can install pure Lua packages (builtin/none)
- ✅ Can build C/C++ extensions from source (make/cmake/command)
- ✅ Can build Rust extensions from source (rust/rust-mlua) - supports packages using `luarocks-build-rust-mlua`
- ✅ Can install packages with pre-built binaries included in the archive
- ✅ Can download pre-built binaries from external URLs (via `binary_urls` in rockspec metadata)
- ✅ All LuaRocks build types are fully supported, including Rust via build backends

### Examples

- **`builtin`** / **`none`**: Pure Lua modules - files copied directly
- **`make`**: LPM runs `make` and `make install` to build and install native extensions
- **`cmake`**: LPM runs `cmake`, `cmake --build`, and `cmake --install` to build and install
- **`command`**: LPM runs the custom build command specified in the rockspec

### Prerequisites for Building

To build packages from source, you need:
- **For `make`**: `make` installed on your system
- **For `cmake`**: `cmake` installed on your system
- **For `command`**: The required build tools specified by the package
- **For `rust`/`rust-mlua`**: Rust toolchain (`rustc`, `cargo`) installed - LPM can install Rust packages that use `luarocks-build-rust-mlua` build backend

## Installation

### Prerequisites

- **Lua 5.1, 5.3, or 5.4** - Optional: LPM includes a built-in Lua version manager, so you don't need to install Lua separately

### Install LPM

#### Option 1: Pre-built Binaries (Recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/yourusername/lpm/releases):

**macOS (Apple Silicon):**
```bash
curl -L https://github.com/yourusername/lpm/releases/latest/download/lpm-aarch64-apple-darwin.tar.gz | tar xz
sudo mv lpm /usr/local/bin/
```

**macOS (Intel):**
```bash
curl -L https://github.com/yourusername/lpm/releases/latest/download/lpm-x86_64-apple-darwin.tar.gz | tar xz
sudo mv lpm /usr/local/bin/
```

**Linux (x86_64):**
```bash
curl -L https://github.com/yourusername/lpm/releases/latest/download/lpm-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv lpm /usr/local/bin/
```

**Linux (ARM64):**
```bash
curl -L https://github.com/yourusername/lpm/releases/latest/download/lpm-aarch64-unknown-linux-gnu.tar.gz | tar xz
sudo mv lpm /usr/local/bin/
```

**Windows:**
1. Download `lpm-x86_64-pc-windows-msvc.zip` from [GitHub Releases](https://github.com/yourusername/lpm/releases/latest)
2. Extract the zip file
3. Add the directory containing `lpm.exe` to your PATH

#### Option 2: Build Locally (Requires Rust)

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

#### Creating Installer Executables

Build installers using the provided script:

```bash
# Build installers for all platforms
./scripts/build-installer.sh all

# Or specify a platform
./scripts/build-installer.sh macos
./scripts/build-installer.sh linux
./scripts/build-installer.sh windows
```

**Prerequisites:**
- **macOS/Linux**: `zig` and `cargo-zigbuild` (installed automatically if missing)
- **Windows**: `clang` (install via `brew install llvm` on macOS) and `cargo-xwin` (installed automatically if missing)
- **macOS**: Xcode Command Line Tools (for `pkgbuild`)

This will create platform-specific installers in the `releases/v{VERSION}/` directory:
- **macOS**: `releases/v{VERSION}/lpm-v{VERSION}-macos-{arch}.pkg` (e.g., `lpm-v0.1.0-macos-aarch64.pkg`)
- **Linux**: `releases/v{VERSION}/lpm-v{VERSION}-linux-{arch}.tar.gz` (e.g., `lpm-v0.1.0-linux-x86_64.tar.gz`)
- **Windows**: `releases/v{VERSION}/lpm-v{VERSION}-windows-x86_64.zip` (includes `lpm.exe` and `install.bat`)

All installers follow the naming convention: `lpm-v{VERSION}-{platform}-{arch}.{ext}`

#### Option 3: Install via Cargo (Requires Rust)

```bash
# Install from crates.io (when published)
cargo install lpm

# Or install from local source
cargo install --path .
```

### Setup PATH

After installation, ensure `lpm` is in your PATH:

#### Unix/macOS/Linux

**Option 1: Automatic setup (recommended)**
```bash
lpm setup-path
```

**Option 2: Manual setup**
Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

#### Windows

**Option 1: Using PowerShell (as Administrator)**
```powershell
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\.cargo\bin", "User")
```

**Option 2: Manual setup**
1. Open System Properties > Environment Variables
2. Edit the `Path` variable in User variables
3. Add `%USERPROFILE%\.cargo\bin`

### Verify Installation

```bash
lpm --version
```

LPM will automatically detect if it's not in PATH and provide setup instructions on first run.

## Quick Start

```bash
# Initialize a new project (interactive wizard)
lpm init

# Or use a template
lpm init --template love2d

# Or non-interactive mode
lpm init --yes

# Install dependencies
lpm install

# Add a package
lpm install luasocket@3.0.0

# Interactive package installation (search and select)
lpm install --interactive

# Run scripts
lpm run start

# Manage Lua versions
lpm lua install latest
lpm lua use 5.4.8

# Install global tools
lpm install -g luacheck
luacheck my_file.lua  # Available everywhere!
```

## Interactive Features

LPM includes powerful interactive features to improve your workflow:

### Interactive Project Initialization

The `lpm init` wizard guides you through project setup:

```bash
lpm init
```

The wizard will prompt you for:
- Project name and version
- Description
- License selection
- Lua version requirement
- Template selection (optional)
- Initial dependencies (optional)
- Common scripts setup (dev, test, build, start)

**Non-interactive mode**: Skip the wizard with `--yes` or `-y`:
```bash
lpm init --yes
lpm init --template love2d --yes
```

### Interactive Package Installation

Search and install packages interactively:

```bash
lpm install --interactive
# or
lpm install -i
```

Features:
- **Fuzzy search**: Find packages by name with intelligent matching
- **Version selection**: Choose from all available versions
- **Dependency type**: Select production or development dependency per package
- **Package metadata**: View description, license, homepage, and dependencies
- **Batch selection**: Install multiple packages at once

### Project Templates

LPM includes built-in templates for common project types:

```bash
# List available templates
lpm template list

# Search templates
lpm template list --search love

# Use a template
lpm init --template love2d
```

**Available templates:**
- `basic-lua` - Basic Lua project structure
- `love2d` - Love2D game development
- `neovim-plugin` - Neovim Lua plugin
- `lapis-web` - OpenResty/Lapis web application
- `cli-tool` - CLI tool with argument parsing

**Create custom templates:**
```bash
# From an existing project
cd my-project
lpm template create my-template --description "My custom template"
```

## Plugins

LPM supports plugins as separate executables that extend core functionality. Plugins are automatically discovered and can be installed globally.

### Available Plugins

#### `lpm-watch` - Dev Server / Watch Mode

Auto-reload your Lua applications on file changes. Perfect for Love2D, Neovim plugins, OpenResty, and general development.

```bash
# Install
lpm install -g lpm-watch

# Use
lpm watch              # Watch and restart on changes
lpm watch dev          # Alias for watch
lpm watch --no-clear   # Don't clear screen on reload
lpm watch --websocket-port 35729  # Enable browser reload
```

**Features:**
- **Multiple commands**: Run multiple commands in parallel
- **Custom file type handlers**: Configure different actions per file extension (restart, reload, ignore)
- **WebSocket support**: Browser auto-reload for HTML/CSS/JS files
- **Better terminal UI**: Colored output with timestamps and status indicators
- File watching with debouncing
- Automatic process restart
- Configurable ignore patterns
- Screen clearing (optional)
- Works with `lpm run` scripts

**Configuration** (in `package.yaml`):
```yaml
watch:
  # Single command (legacy)
  command: "lua src/main.lua"
  
  # Multiple commands (run in parallel)
  commands:
    - "lua src/server.lua"
    - "lua src/worker.lua"
  
  paths:
    - "src"
    - "lib"
  ignore:
    - "**/*.test.lua"
    - "**/tmp/**"
  
  # WebSocket server for browser reload
  websocket_port: 35729
  
  # Custom file type handlers
  file_handlers:
    lua: restart      # Restart command on .lua changes
    yaml: restart    # Restart command on .yaml changes
    html: reload     # Send reload signal to browser
    css: reload      # Send reload signal to browser
    js: reload       # Send reload signal to browser
    txt: ignore      # Ignore .txt file changes
  
  debounce_ms: 100
  clear: true
```

**File Actions:**
- `restart`: Restart the command(s) when files of this type change
- `reload`: Send reload signal via WebSocket (for browser reload)
- `ignore`: No action when files of this type change

#### `lpm-bundle` - Package Bundling (Experimental)

Bundle multiple Lua files into a single file for distribution or embedding.

```bash
# Install
lpm install -g lpm-bundle

# Use
lpm bundle bundle                    # Bundle src/main.lua to dist/bundle.lua
lpm bundle bundle -e src/init.lua   # Custom entry point
lpm bundle bundle -o dist/app.lua    # Custom output
lpm bundle bundle --minify           # Minify output
lpm bundle bundle --source-map       # Generate source map
lpm bundle bundle --no-comments      # Strip comments
lpm bundle bundle --tree-shake       # Enable tree-shaking
lpm bundle bundle --incremental      # Incremental bundling
lpm bundle watch                     # Watch mode (auto-rebundle)
```

**Features:**
- Static dependency analysis using Lua parser (full_moon)
- Circular dependency detection
- Basic minification (whitespace and comment removal)
- Source map generation
- Standalone bundle with custom require runtime
- Watch mode for automatic re-bundling
- Incremental bundling support
- Dynamic requires tracking (warns about dynamic require() calls)

**⚠️ Limitations:**
- Dynamic requires (`require(variable)`) are not detected
- C modules cannot be bundled (warnings shown)
- Minifier is basic and may not work for all code
- Marked as experimental

### Installing Plugins

Plugins are installed globally and become available as `lpm <plugin-name>`:

```bash
# Install a plugin
lpm install -g lpm-watch
lpm install -g lpm-bundle

# Plugins are automatically discovered
lpm watch --help
lpm bundle --help
```

### Plugin Locations

Plugins are installed to:
- **macOS**: `~/Library/Application Support/lpm/bin/`
- **Linux**: `~/.config/lpm/bin/`
- **Windows**: `%APPDATA%\lpm\bin\`
- **Legacy**: `~/.lpm/bin/` (for backwards compatibility)

Plugins can also be installed anywhere in your PATH.

### Creating Plugins

See the [Plugin Development Guide](docs/contributing/Plugin-Development.md) for details on creating your own plugins.

## Documentation

### API Documentation

LPM provides a Rust library API for programmatic package management:

#### Core Types

```rust
use lpm::{LpmError, LpmResult};

// Error handling
pub type LpmResult<T> = Result<T, LpmError>;

pub enum LpmError {
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
    Http(reqwest::Error),
    Path(String),
    Config(String),
    Package(String),
    Version(String),
    Cache(String),
    // ...
}
```

#### Package Management

```rust
use lpm::package::manifest::PackageManifest;

// Load package.yaml
let manifest = PackageManifest::load(&project_root)?;

// Access dependencies
for (name, version) in &manifest.dependencies {
    println!("{}: {}", name, version);
}

// Save updated manifest
manifest.save(&project_root)?;
```

#### Dependency Resolution

```rust
use lpm::resolver::{DependencyResolver, DependencyGraph};

let resolver = DependencyResolver::new();
let graph = resolver.resolve(&manifest)?;

// Check for conflicts
if let Some(conflict) = graph.detect_conflicts() {
    eprintln!("Version conflict: {}", conflict);
}
```

#### LuaRocks Integration

```rust
use lpm::luarocks::client::LuaRocksClient;
use lpm::cache::Cache;

let cache = Cache::new(cache_dir()?)?;
let client = LuaRocksClient::new(cache);

// Search for packages
let results = client.search("socket").await?;

// Download package
let package = client.download_package("luasocket", "3.0.0").await?;
```

#### Building Rust Extensions

```rust
use lpm::build::builder::RustBuilder;
use lpm::build::targets::Target;

let builder = RustBuilder::new(&manifest, &project_root)?;
let target = Target::default();

// Build for current platform
builder.build(Some(&target)).await?;

// Build for all targets
for target in SUPPORTED_TARGETS {
    builder.build(Some(target)).await?;
}
```

### Migration Guide from LuaRocks

#### Step 1: Initialize LPM Project

```bash
# In your existing LuaRocks project directory
lpm init
```

This creates a `package.yaml` file. You can keep your existing `rockspec` files - LPM will use them as a reference.

#### Step 2: Convert Dependencies

LPM automatically converts LuaRocks dependencies when you install packages:

```bash
# Install your existing dependencies
lpm install luasocket@3.0.0
lpm install penlight@^1.13.0
```

Or manually edit `package.yaml`:

```yaml
dependencies:
  luasocket: "3.0.0"
  penlight: "^1.13.0"
```

#### Step 3: Update Your Code

**Before (LuaRocks global install):**
```lua
-- Works because luasocket is globally installed
local socket = require("socket")
```

**After (LPM local install):**
```lua
-- Still works! LPM sets up package.path automatically
local socket = require("socket")
```

LPM's `lpm.loader` module automatically configures `package.path` and `package.cpath` to include `./lua_modules/`.

#### Step 4: Update Build Scripts

**Before:**
```bash
luarocks install --only-deps
lua main.lua
```

**After:**
```bash
lpm install
lpm run start  # or: lua main.lua
```

#### Step 5: Update CI/CD

**Before:**
```yaml
# .github/workflows/test.yml
- run: luarocks install --only-deps
- run: lua tests/run.lua
```

**After:**
```yaml
# .github/workflows/test.yml
- run: cargo install --path .
- run: lpm install
- run: lpm run test
```

#### Key Differences

| LuaRocks | LPM |
|----------|-----|
| Global installation | Local installation (`./lua_modules/`) |
| `luarocks install` | `lpm install` |
| `luarocks list` | `lpm list` |
| `luarocks remove` | `lpm remove` |
| No lockfile | `package.lock` for reproducibility |
| Global `package.path` | Project-scoped `package.path` |

### Examples and Tutorials

#### Example 1: Interactive Project Setup with Template

```bash
# 1. Initialize project with interactive wizard
lpm init
# Follow the prompts:
#   - Project name: my-game
#   - Version: 1.0.0
#   - Description: A Love2D game
#   - License: MIT
#   - Lua version: 5.4
#   - Template: love2d
#   - Add initial dependencies: Yes (then search and select packages)
#   - Scripts: dev, test

# 2. Or use non-interactive mode with template
lpm init --template love2d --yes

# 3. Add dependencies interactively
lpm install --interactive
# Search for packages, select versions, choose dev/prod

# 4. Or add dependencies directly
lpm install luasocket@3.0.0
lpm install penlight@^1.13.0

# 5. Your package.yaml now looks like:
# name: my-game
# version: 1.0.0
# description: A Love2D game
# license: MIT
# lua_version: "5.4"
# dependencies:
#   luasocket: "3.0.0"
#   penlight: "^1.13.0"
# scripts:
#   dev: "lpm watch dev"
#   test: "lua tests/run.lua"

# 6. Use in your Lua code
# local socket = require("socket")
# local pl = require("pl")
```

#### Example 2: Rust Extension Project

```yaml
# package.yaml
name: my-rust-module
version: "1.0.0"
build:
  type: rust
  manifest: Cargo.toml
  modules:
    mymodule: "target/release/libmymodule.so"  # or .dylib/.dll
```

```bash
# Build Rust extension
lpm build

# Use in Lua
# local mymodule = require("mymodule")
```

#### Example 3: Scripts and Automation

```yaml
# package.yaml
scripts:
  test: "lua tests/run.lua"
  start: "lua src/main.lua"
  lint: "lua tools/lint.lua"
```

```bash
# Run scripts
lpm run test
lpm run start
lpm run lint
```

#### Example 4: Dev Dependencies

```yaml
# package.yaml
dependencies:
  luasocket: "3.0.0"

dev_dependencies:
  luaunit: "^3.4"
  busted: "^2.0.0"
```

```bash
# Install all dependencies (including dev)
lpm install

# Install only production dependencies
lpm install --no-dev

# Install only dev dependencies
lpm install --dev-only
```

#### Example 5: Workspace/Monorepo

```bash
# Project structure:
# workspace/
#   ├── package.yaml          # Root workspace
#   ├── package-a/
#   │   └── package.yaml
#   └── package-b/
#       └── package.yaml

# Install all workspace dependencies
lpm install  # Automatically detects workspace
```

### Security Best Practices Guide

#### 1. Always Use Lockfiles

Commit `package.lock` to version control:

```bash
git add package.lock
git commit -m "Add lockfile for reproducible builds"
```

The lockfile ensures:
- Exact versions are installed
- Checksums verify package integrity
- Reproducible builds across environments

#### 2. Verify Package Checksums

```bash
# Verify all installed packages match lockfile checksums
lpm verify
```

This ensures packages haven't been tampered with.

#### 3. Regular Security Audits

```bash
# Check for known vulnerabilities
lpm audit
```

LPM checks packages against:
- OSV (Open Source Vulnerabilities) database
- GitHub Security Advisories

#### 4. Review Dependency Updates

```bash
# Check for outdated packages
lpm outdated

# Review changes before updating
lpm update --dry-run  # (if implemented)

# Update with review
lpm update
```

#### 5. Use Version Constraints Wisely

**Recommended:**
```yaml
dependencies:
  luasocket: "^3.0.0"    # Allows patch and minor updates
  penlight: "~1.13.0"    # Allows only patch updates
```

**Avoid:**
```yaml
dependencies:
  luasocket: "*"         # Too permissive - any version
  penlight: ">=1.0.0"    # Too broad - major version changes
```

**For critical dependencies:**
```yaml
dependencies:
  crypto-lib: "1.2.3"    # Exact version for security-critical packages
```

#### 6. Sandboxed Builds

LPM automatically sandboxes:
- Rockspec parsing (no filesystem/network access)
- Rust extension builds (isolated environment)

#### 7. No Postinstall Scripts

LPM does **not** execute postinstall scripts for security. If a package requires setup, do it manually or use LPM scripts:

```yaml
scripts:
  postinstall: "lua scripts/setup.lua"
```

#### 8. Credential Storage

LPM stores credentials securely using OS keychains:
- macOS: Keychain
- Windows: Credential Manager
- Linux: Secret Service (libsecret)

```bash
# Login credentials are stored securely
lpm login
```

#### 9. Cache Security

LPM cache is stored in:
- Unix/macOS: `~/.cache/lpm/` or `~/.lpm/cache/`
- Windows: `%LOCALAPPDATA%\lpm\cache\`

Cache files are verified with checksums before use.

#### 10. Supply Chain Security Checklist

- [ ] Review all dependencies before adding
- [ ] Use `lpm audit` regularly
- [ ] Keep dependencies up to date
- [ ] Use exact versions for security-critical packages
- [ ] Commit and review `package.lock`
- [ ] Verify checksums with `lpm verify`
- [ ] Review dependency updates with `lpm outdated`

## Development

```bash
# Build
cargo build

# Run
cargo run -- install

# Test
cargo test
```

## License

MIT

