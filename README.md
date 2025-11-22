# Projectionist

Project-aware file navigation for Zed (and other editors) via `projections.json`.

A Rust port of Tim Pope's [vim-projectionist](https://github.com/tpope/vim-projectionist).

## Features

- **Alternate file navigation**: Switch between source and test files
- **Related file discovery**: Find all files related to the current file
- **Template-based file creation**: Create new files with predefined templates
- **14 transformations**: Transform placeholders (camelCase, snake_case, plural, etc.)

## Installation

```bash
# From source
cargo install --path .

# From crates.io (when published)
cargo install projectionist
```

## Quick Start

1. Create a `projections.json` configuration file in your project root:

   - `.projections.json` (vim-projectionist compatible)
   - `.zed/projections.json` (Zed-specific location)

   Example:

```json
{
  "src/*.ts": {
    "alternate": "test/{}.test.ts",
    "type": "source"
  },
  "test/*.test.ts": {
    "alternate": "src/{}.ts",
    "type": "test"
  }
}
```

2. Use the CLI:

```bash
# Find alternate file
projectionist alternate src/utils.ts

# Open alternate in Zed
projectionist alternate src/utils.ts --open

# Find related files
projectionist related src/components/Button.tsx

# Create file from template
projectionist create src/components/NewComponent.tsx

# Show projection info
projectionist info src/utils.ts
```

## Zed Integration

### Tasks (~/.config/zed/tasks.json)

```json
[
  {
    "label": "Switch to Alternate File",
    "command": "projectionist",
    "args": ["alternate", "$ZED_FILE", "--open"],
    "cwd": "$ZED_WORKTREE_ROOT",
    "reveal": "never",
    "hide": "always"
  },
  {
    "label": "Show Projection Info",
    "command": "projectionist",
    "args": ["info", "$ZED_FILE"],
    "cwd": "$ZED_WORKTREE_ROOT",
    "reveal": "always"
  }
]
```

### Keybindings (~/.config/zed/keymap.json)

> **Important**: You MUST include `"reevaluate_context": true` or Zed will cache the file path.

```json
[
  {
    "context": "Editor && !menu",
    "bindings": {
      "ctrl-shift-a": [
        "task::Spawn",
        { "task_name": "Switch to Alternate File", "reevaluate_context": true }
      ]
    }
  }
]
```

## Configuration File Location

The CLI searches for configuration files in the following order (first found wins):

| Location | Description |
|----------|-------------|
| `.projections.json` | Root of project (vim-projectionist compatible) |
| `.zed/projections.json` | Inside `.zed` directory (Zed-specific) |

If both files exist, `.projections.json` takes priority for vim-projectionist compatibility.

## Configuration Format

The configuration file maps glob patterns to configurations:

```json
{
  "src/components/*.tsx": {
    "alternate": "src/components/{}.test.tsx",
    "related": ["src/components/{}.module.css", "src/components/{}.stories.tsx"],
    "type": "component",
    "template": [
      "import React from 'react';",
      "",
      "export const {}: React.FC = () => {",
      "  return <div>{}</div>;",
      "};"
    ],
    "define": "export const {}"
  }
}
```

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `alternate` | `string` | Primary alternate file pattern |
| `related` | `string[]` | Array of related file patterns |
| `type` | `string` | File type identifier |
| `template` | `string[]` | Template lines for new file creation |
| `define` | `string` | Search pattern for auto-scroll |

### Placeholders

| Syntax | Description |
|--------|-------------|
| `{}` | The captured stem (everything matched by `*`) |
| `{name}` | Named placeholder |
| `{\|transform}` | Stem with transformation |
| `{\|t1\|t2}` | Chained transformations |

### Transformations

**Path separators:**
- `dot` - Replace `/` with `.`
- `underscore` - Replace `/` with `_`
- `backslash` - Replace `/` with `\`
- `colons` - Replace `/` with `::`

**Case:**
- `uppercase` - Convert to UPPERCASE
- `lowercase` - Convert to lowercase
- `camelcase` - Convert to camelCase
- `snakecase` - Convert to snake_case
- `hyphenate` - Replace `_` with `-`

**Path components:**
- `dirname` - Directory portion
- `basename` - File name only
- `file` - Name without extension
- `ext` - Extension only

**Inflection:**
- `plural` - Pluralize (user → users)
- `singular` - Singularize (users → user)

## Example Configurations

### React/TypeScript

```json
{
  "src/components/*.tsx": {
    "alternate": "src/components/{}.test.tsx",
    "related": ["src/components/{}.module.css"],
    "type": "component"
  },
  "src/components/*.test.tsx": {
    "alternate": "src/components/{}.tsx",
    "type": "test"
  }
}
```

### Go

```json
{
  "*.go": {
    "alternate": "{}_test.go",
    "type": "source"
  },
  "*_test.go": {
    "alternate": "{}.go",
    "type": "test"
  }
}
```

### Rails

```json
{
  "app/models/*.rb": {
    "alternate": "spec/models/{}_spec.rb",
    "related": ["db/schema.rb"],
    "define": "create_table \"{|plural}\"",
    "type": "model"
  },
  "app/controllers/*_controller.rb": {
    "alternate": "spec/controllers/{}_controller_spec.rb",
    "type": "controller"
  }
}
```

## CLI Reference

```
projectionist <COMMAND>

Commands:
  alternate  Find alternate file(s) for the given file
  related    Find related files for the given file
  create     Create a new file from template
  info       Show projection info for a file
  validate   Validate .projections.json configuration

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### alternate

```
projectionist alternate <FILE> [OPTIONS]

Options:
  -l, --line <LINE>        Line number for cursor-aware features
  -o, --open               Open result in Zed editor
  -j, --json               Output as JSON
      --create-if-missing  Create the alternate file from template if missing
  -v, --verbose            Show verbose matching information
```

### related

```
projectionist related <FILE> [OPTIONS]

Options:
  -l, --line <LINE>  Line number for cursor-aware features
  -j, --json         Output as JSON
      --fzf          Output one file per line for fzf/pipe usage
  -v, --verbose      Show verbose information
```

### create

```
projectionist create <FILE> [OPTIONS]

Options:
  -f, --force  Overwrite if file already exists
  -o, --open   Open the created file in Zed
```

### info

```
projectionist info <FILE> [OPTIONS]

Options:
  -l, --line <LINE>  Line number for cursor-aware features
  -j, --json         Output as JSON
  -v, --verbose      Show verbose information
```

## License

MIT
