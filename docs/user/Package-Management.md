# Package Management

Complete guide to managing dependencies with LPM.

## package.yaml Format

The `package.yaml` file is your project manifest:

```yaml
name: my-project
version: 1.0.0
description: "My awesome Lua project"
author: "Your Name"
license: "MIT"
lua_version: "5.4"  # Or ">=5.1", "5.1 || 5.3 || 5.4", etc.

dependencies:
  luasocket: "3.0.0"           # Exact version
  penlight: "^1.13.0"           # Compatible version
  lua-cjson: "~2.1.0"           # Patch version
  inspect: "*"                  # Any version

dev_dependencies:
  busted: "^2.0.0"              # Test framework
  luacheck: "^1.0.0"            # Linter

scripts:
  test: "busted tests/"
  lint: "luacheck src/"
  build: "lua build.lua"
```

## Version Constraints

LPM uses Semantic Versioning (SemVer) constraints:

- `"3.0.0"` - Exact version
- `"^1.13.0"` - Compatible version (>=1.13.0 <2.0.0)
- `"~2.1.0"` - Patch version (>=2.1.0 <2.2.0)
- `">=1.0.0"` - Greater than or equal
- `"<2.0.0"` - Less than
- `"1.0.0 || 2.0.0"` - Either version
- `"*"` - Any version

## Dependency Resolution

LPM automatically resolves dependency conflicts:

1. **Version Selection**: Chooses the highest compatible version
2. **Conflict Detection**: Warns if dependencies conflict
3. **Lockfile Generation**: Creates `package.lock` with exact versions

### Example Resolution

```yaml
# package.yaml
dependencies:
  package-a: "^1.0.0"  # Needs package-b ^2.0.0
  package-b: "^1.0.0"  # Conflicts with package-a's requirement
```

LPM will detect this conflict and suggest a resolution.

## Lockfile (package.lock)

The `package.lock` file ensures reproducible builds:

```yaml
packages:
  luasocket:
    version: "3.0.0"
    checksum: "sha256:abc123..."
    dependencies: {}
  penlight:
    version: "1.13.0"
    checksum: "sha256:def456..."
    dependencies:
      luafilesystem: "1.8.0"
```

**Important**: Commit `package.lock` to version control for reproducible builds.

## Dev Dependencies

Dev dependencies are only installed in development:

```yaml
dev_dependencies:
  busted: "^2.0.0"
  luacheck: "^1.0.0"
```

```bash
# Install all dependencies (including dev)
lpm install

# Skip dev dependencies (production)
lpm install --no-dev

# Install only dev dependencies
lpm install --dev-only
```

## Workspace Support

LPM supports monorepos with multiple packages:

```
workspace/
├── package.yaml          # Workspace root
├── package-a/
│   └── package.yaml
└── package-b/
    └── package.yaml
```

Shared dependencies are managed at the workspace level.

## Local Dependencies

Install packages from local paths:

```bash
lpm install --path ./local-package
```

Or in `package.yaml`:

```yaml
dependencies:
  local-pkg:
    path: "./local-package"
```

## Global Installation

Install packages globally so they're available everywhere (like `npm install -g`):

```bash
# Install globally
lpm install -g luacheck
lpm install -g busted

# Now available everywhere (after adding ~/.lpm/bin/ to PATH)
luacheck my_file.lua
busted
```

**Global installation directory:**
- Packages: `~/.lpm/global/lua_modules/`
- Executables: `~/.lpm/bin/`

**Setup PATH for global tools:**

```bash
# Unix/macOS - add to ~/.bashrc, ~/.zshrc, etc.
export PATH="$HOME/.lpm/bin:$PATH"

# Or on macOS:
export PATH="$HOME/Library/Application Support/lpm/bin:$PATH"
```

**Note**: Global tools use LPM-managed Lua versions automatically. Make sure you have a Lua version installed with `lpm lua install latest`.

## Updating Dependencies

### Update All

```bash
lpm update
```

Updates all dependencies to their latest compatible versions.

### Update Specific Package

```bash
lpm update luasocket
```

### Check for Updates

```bash
lpm outdated
```

Shows which packages have newer versions available.

## Removing Dependencies

```bash
lpm remove luasocket
```

Removes the package from `package.yaml` and `lua_modules/`.

## Verifying Dependencies

Verify package integrity:

```bash
lpm verify
```

Checks all package checksums against `package.lock`.

## Building from Source

LPM supports building packages from source for `make`, `cmake`, `command`, and `rust` build types:

- **`make`**: Runs `make` and `make install` to build and install native extensions
- **`cmake`**: Runs `cmake`, `cmake --build`, and `cmake --install` to build and install
- **`command`**: Runs custom build commands specified in the rockspec
- **`rust`** / **`rust-mlua`**: Builds Rust extensions using `cargo build --release` - supports packages using `luarocks-build-rust-mlua` build backend

### Prerequisites

- **For `make`**: `make` must be installed
- **For `cmake`**: `cmake` must be installed
- **For `command`**: Required build tools as specified by the package
- **For `rust`/`rust-mlua`**: Rust toolchain (`rustc`, `cargo`) must be installed

## Binary Package Support

LPM supports downloading pre-built binaries from external URLs. This is useful for packages with native extensions that don't include binaries in their source archives.

### Using Binary URLs in Rockspecs

Packages can specify binary URLs in their rockspec metadata:

```lua
-- rockspec file
metadata = {
  binary_urls = {
    ["5.4-x86_64-unknown-linux-gnu"] = "https://example.com/binary-linux-x64.so",
    ["5.4-aarch64-apple-darwin"] = "https://example.com/binary-macos-arm64.dylib",
    ["5.4-x86_64-pc-windows-msvc"] = "https://example.com/binary-windows-x64.dll",
  }
}
```

Or directly in the rockspec:

```lua
binary_urls = {
  ["5.4-x86_64-unknown-linux-gnu"] = "https://example.com/binary-linux-x64.so",
}
```

LPM will:
1. Check for a binary URL matching your Lua version and platform
2. Download the binary if available
3. Cache it for future use
4. Fall back to source installation if no binary URL is found

### Performance Benefits

- **Parallel Downloads**: LPM downloads multiple packages in parallel (up to 10 concurrent downloads) for faster installation
- **Incremental Lockfile Updates**: Only changed packages are rebuilt when updating the lockfile
- **Manifest Caching**: LuaRocks manifest is cached locally for faster lookups

