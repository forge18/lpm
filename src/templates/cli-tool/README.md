# {{project_name}}

A command-line tool written in Lua.

## Installation

```bash
# Install dependencies
lpm install

# Make executable (Unix)
chmod +x src/main.lua
```

## Usage

```bash
# Run the CLI tool
lua src/main.lua <input> [options]

# Or if made executable
./src/main.lua <input> [options]

# With options
lua src/main.lua input.txt -o output.txt --verbose
```

## Project Structure

- `src/main.lua` - Main CLI entry point
- `src/` - CLI tool source code
- `lib/` - Library code

## Development

```bash
# Install dependencies
lpm install

# Run tests
lpm run test
```

## Building

To create a standalone executable, you can use `lpm bundle`:

```bash
lpm bundle src/main.lua -o {{project_name}}
```

