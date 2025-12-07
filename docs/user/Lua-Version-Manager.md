# Lua Version Manager

LPM includes a built-in Lua version manager that lets you install and switch between different Lua versions without system-wide installations.

## Overview

The Lua version manager allows you to:
- Install multiple Lua versions (5.1, 5.3, 5.4)
- Switch between versions globally or per-project
- Use project-specific versions via `.lua-version` files
- Automatically use the correct version when running scripts

## Installation Location

Lua versions are installed to:
- **Unix/macOS**: `~/.lpm/versions/` (or `~/Library/Application Support/lpm/versions/` on macOS)
- **Windows**: `%APPDATA%\lpm\versions\`

Each version is installed in its own directory:
```
~/.lpm/
├── versions/
│   ├── 5.1.5/
│   │   └── bin/
│   │       ├── lua
│   │       └── luac
│   ├── 5.3.6/
│   └── 5.4.8/
├── bin/
│   ├── lua          # Wrapper (auto-detects version)
│   └── luac         # Wrapper (auto-detects version)
└── current          # Text file with current global version
```

## Installing Lua Versions

### Install Latest Version

```bash
lpm lua install latest
```

### Install Specific Version

```bash
lpm lua install 5.4.8
lpm lua install 5.3.6
lpm lua install 5.1.5
```

### List Available Versions

```bash
lpm lua list-remote
```

## Switching Versions

### Global Version

Switch the global default version:

```bash
lpm lua use 5.4.8
```

This updates `~/.lpm/current` and affects all projects that don't have a `.lua-version` file.

### Project-Specific Version

Set a version for the current project:

```bash
lpm lua local 5.3.6
```

This creates a `.lua-version` file in your project root. The wrappers will automatically use this version when you're in this project or its subdirectories.

### Check Current Version

```bash
# Show global version
lpm lua current

# Show which version will be used (respects .lua-version)
lpm lua which
```

## Using Lua Versions

### Setup PATH

Add `~/.lpm/bin/` to your PATH to use the `lua` and `luac` wrappers:

```bash
# Unix/macOS - add to ~/.bashrc, ~/.zshrc, etc.
export PATH="$HOME/.lpm/bin:$PATH"

# Or on macOS:
export PATH="$HOME/Library/Application Support/lpm/bin:$PATH"
```

### How Wrappers Work

The `lua` and `luac` wrappers in `~/.lpm/bin/`:
1. Walk up the directory tree looking for `.lua-version` files
2. If found, use that version
3. Otherwise, use the global version from `~/.lpm/current`
4. Execute the correct Lua binary with all arguments

### Example

```bash
# Project A uses Lua 5.3
cd project-a
lpm lua local 5.3.6
lua script.lua  # Uses 5.3.6

# Project B uses Lua 5.4
cd ../project-b
lpm lua local 5.4.8
lua script.lua  # Uses 5.4.8

# Outside projects, uses global version
cd ~
lua script.lua  # Uses global version (e.g., 5.4.8)
```

## Managing Versions

### List Installed Versions

```bash
lpm lua list
```

Output:
```
  5.3.6
  5.4.8 (current)
```

### Uninstall a Version

```bash
lpm lua uninstall 5.3.6
```

**Note**: You cannot uninstall the currently active version. Switch to another version first.

### Execute with Specific Version

Run a command with a specific version without switching:

```bash
lpm lua exec 5.3.6 lua script.lua
```

## Configuration

### Custom Binary Sources

By default, LPM downloads Lua binaries from `dyne/luabinaries`. You can configure alternative sources:

```bash
# Set default source for all versions
lpm config set lua_binary_source_url https://example.com/lua-binaries

# Set source for a specific version
lpm config set lua_binary_sources.5.4.8 https://custom-source.com/binaries
```

### Supported Versions

Known versions with pre-built binaries:
- Lua 5.1.5
- Lua 5.3.6
- Lua 5.4.8

Future versions are supported automatically - LPM dynamically parses version numbers to determine binary names.

## Integration with LPM Scripts

When you use `lpm run` or `lpm exec`, LPM automatically uses the correct Lua version:
- Checks for `.lua-version` in the project
- Falls back to global version
- Uses LPM-managed Lua binaries directly (no PATH dependency)

## Troubleshooting

### "No Lua version is currently selected"

Install and use a version:
```bash
lpm lua install latest
lpm lua use 5.4.8
```

### "Lua binary not found"

The version might not be installed. Check with:
```bash
lpm lua list
```

### Wrappers not working

Make sure `~/.lpm/bin/` is in your PATH:
```bash
echo $PATH | grep lpm
```

If not, add it to your shell profile.

