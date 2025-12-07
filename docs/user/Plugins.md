# LPM Plugins

LPM supports plugins as separate executables that extend core functionality. Plugins are automatically discovered and can be installed globally.

## Available Plugins

### `lpm-watch` - Dev Server / Watch Mode

Auto-reload your Lua applications on file changes. Perfect for Love2D, Neovim plugins, OpenResty, and general development.

#### Installation

```bash
lpm install -g lpm-watch
```

#### Basic Usage

```bash
# Watch and restart on changes
lpm watch

# Alias for watch
lpm watch dev

# Don't clear screen on reload
lpm watch --no-clear

# Enable WebSocket server for browser reload
lpm watch --websocket-port 35729
```

#### Features

- **Multiple commands**: Run multiple commands in parallel
- **Custom file type handlers**: Configure different actions per file extension
- **WebSocket support**: Browser auto-reload for HTML/CSS/JS files
- **Enhanced terminal UI**: Colored output with timestamps and status indicators
- File watching with debouncing
- Automatic process restart
- Configurable ignore patterns
- Screen clearing (optional)
- Works with `lpm run` scripts

#### Configuration

Configure `lpm-watch` in your `package.yaml`:

```yaml
watch:
  # Single command (legacy support)
  command: "lua src/main.lua"
  
  # Multiple commands (run in parallel)
  commands:
    - "lua src/server.lua"
    - "lua src/worker.lua"
  
  # Paths to watch
  paths:
    - "src"
    - "lib"
    - "assets"
  
  # Patterns to ignore
  ignore:
    - "**/*.test.lua"
    - "**/tmp/**"
    - "**/.git/**"
  
  # WebSocket server port (0 = disabled)
  websocket_port: 35729
  
  # Custom file type handlers
  file_handlers:
    lua: restart      # Restart command on .lua changes
    yaml: restart    # Restart command on .yaml changes
    html: reload     # Send reload signal to browser
    css: reload      # Send reload signal to browser
    js: reload       # Send reload signal to browser
    txt: ignore      # Ignore .txt file changes
  
  # Debounce delay in milliseconds
  debounce_ms: 300
  
  # Clear screen on restart
  clear: true
```

#### File Actions

Configure how different file types are handled:

- **`restart`**: Restart the command(s) when files of this type change
- **`reload`**: Send reload signal via WebSocket (for browser reload)
- **`ignore`**: No action when files of this type change

#### WebSocket Browser Reload

Enable browser auto-reload by setting `websocket_port` in your configuration or using the `--websocket-port` flag:

```bash
lpm watch --websocket-port 35729
```

Then add this script to your HTML files:

```html
<script>
  const ws = new WebSocket('ws://localhost:35729');
  ws.onmessage = function(event) {
    const data = JSON.parse(event.data);
    if (data.type === 'reload') {
      location.reload();
    }
  };
</script>
```

When HTML, CSS, or JS files change, the browser will automatically reload.

#### Multiple Commands

Run multiple commands in parallel:

```yaml
watch:
  commands:
    - "lua src/server.lua"
    - "lua src/worker.lua"
    - "lua src/scheduler.lua"
```

All commands will start simultaneously and restart together when watched files change.

#### CLI Options

```bash
lpm watch [OPTIONS]

Options:
  -c, --command <COMMAND>    Command to run (can be specified multiple times)
  -p, --paths <PATHS>        Paths to watch (default: src/, lib/)
  -i, --ignore <PATTERNS>    Patterns to ignore
      --no-clear             Don't clear screen on restart
  -s, --script <SCRIPT>      Script name from package.yaml to run
      --websocket-port <PORT>  WebSocket port for browser reload (0 = disabled)
```

### `lpm-bundle` - Package Bundling (Experimental)

Bundle multiple Lua files into a single file for distribution or embedding.

#### Installation

```bash
lpm install -g lpm-bundle
```

#### Usage

```bash
# Bundle src/main.lua to dist/bundle.lua
lpm bundle bundle

# Custom entry point
lpm bundle bundle -e src/init.lua

# Custom output
lpm bundle bundle -o dist/app.lua

# Minify output
lpm bundle bundle --minify

# Generate source map
lpm bundle bundle --source-map

# Strip comments (without minifying)
lpm bundle bundle --no-comments

# Enable tree-shaking
lpm bundle bundle --tree-shake

# Track dynamic requires
lpm bundle bundle --dynamic-requires

# Incremental bundling (only rebuild changed modules)
lpm bundle bundle --incremental

# Watch mode (auto-rebundle on changes)
lpm bundle watch
```

#### Features

- Static dependency analysis using Lua parser (full_moon)
- Circular dependency detection
- Basic tree-shaking (remove unused code) - basic implementation
- Basic minification (whitespace and comment removal)
- Source map generation
- Standalone bundle with custom require runtime
- Watch mode for automatic re-bundling
- Incremental bundling support (checks file modification times)
- Dynamic requires tracking (warns about dynamic require() calls)

#### Limitations

- Dynamic requires (`require(variable)`) are not detected (warnings shown)
- C modules cannot be bundled (warnings shown)
- Marked as experimental

## Installing Plugins

Plugins are installed globally and become available as `lpm <plugin-name>`:

```bash
# Install a plugin
lpm install -g lpm-watch
lpm install -g lpm-bundle

# Plugins are automatically discovered
lpm watch --help
lpm bundle --help
```

## Plugin Locations

Plugins are installed to:
- **macOS**: `~/Library/Application Support/lpm/bin/`
- **Linux**: `~/.config/lpm/bin/`
- **Windows**: `%APPDATA%\lpm\bin\`
- **Legacy**: `~/.lpm/bin/` (for backwards compatibility)

Plugins can also be installed anywhere in your PATH.

## Managing Plugins

Use the `lpm plugin` commands to manage plugins:

```bash
# List installed plugins
lpm plugin list

# Show plugin information
lpm plugin info <plugin-name>

# Update plugins
lpm plugin update
lpm plugin update <plugin-name>

# Check for outdated plugins
lpm plugin outdated

# Search for plugins
lpm plugin search <query>

# Configure plugins
lpm plugin config get <plugin> <key>
lpm plugin config set <plugin> <key> <value>
lpm plugin config show <plugin>
```

## Creating Plugins

See the [Plugin Development Guide](../contributing/Plugin-Development.md) for details on creating your own plugins.

