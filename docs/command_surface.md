# BookDB Command Surface Documentation

## Overview
This document catalogs the complete command surface of BookDB.sh to ensure feature parity in the Rust port. All commands, flags, and behaviors are documented for reference.

## Core Commands

### Information & Status Commands
| Command | Aliases | Description | Example |
|---------|---------|-------------|---------|
| `status` | `st` | Display dashboard with system state and projects | `bookdb status` |
| `cursor` | `c` | Print current active cursor chain | `bookdb cursor` |
| `base` | - | Show current active base | `bookdb base` |
| `ls` | - | List data (bases/projects/keys/etc) | `bookdb ls all` |
| `find` | - | Find a key across all projects/keystores | `bookdb find API_KEY` |

### Variable Operations (CRUD)
| Command | Description | Example |
|---------|-------------|---------|
| `getv` | Get the value of a variable | `bookdb getv API_KEY` |
| `setv` | Create or update a variable | `bookdb setv API_KEY=secret123` |
| `delv` | Delete a variable | `bookdb delv API_KEY` |
| `incv` | Increment a numerical value | `bookdb incv COUNT 5` |
| `decv` | Decrement a numerical value | `bookdb decv COUNT 2` |

### Document Operations (Not yet in Rust port)
| Command | Description | Example |
|---------|-------------|---------|
| `getd` | Get a document | `bookdb getd README` |
| `setd` | Set a document | `bookdb setd README < file.md` |
| `deld` | Delete a document | `bookdb deld README` |

### Base Management (Multi-base Support)
| Command | Description | Example |
|---------|-------------|---------|
| `select` | Set persistent active base | `bookdb select work` |
| `rebase` | **DANGER**: Delete and recreate a base | `bookdb rebase temp` |
| `unbase` | **DANGER**: Delete a base completely | `bookdb unbase old_base` |

### Creation & Deletion
| Command | Description | Example |
|---------|-------------|---------|
| `new` | Create new base/project/keystore | `bookdb new base work` |
| `del` | Delete project/keystore | `bookdb del project old_proj` |

### File Publishing
| Command | Description | Example |
|---------|-------------|---------|
| `pub` | Publish key to external file | `bookdb pub API_KEY ~/.env` |
| `unpub` | Remove key from external file | `bookdb unpub API_KEY ~/.env` |

### Import/Export & Backup
| Command | Description | Example |
|---------|-------------|---------|
| `export` | Export keystore to .env file | `bookdb export keystore` |
| `import` | Import keys from .env file | `bookdb import config.env` |
| `backup` | Create backup (manual) | `bookdb backup --all` |
| `migrate` | Export all keystores to backup | `bookdb migrate --all` |

### Administrative Commands
| Command | Description | Example |
|---------|-------------|---------|
| `install` | Install/update bookdb application | `bookdb install` |
| `reset` | **DANGER**: Delete all data and artifacts | `bookdb reset` |
| `sanity` | Quick installation state check | `bookdb sanity` |

### Development & Debug Commands
| Command | Description | Example |
|---------|-------------|---------|
| `tables` | List all database tables | `bookdb tables` |
| `tdump` | Dump table contents | `bookdb tdump variables` |
| `dev_setup` | Create test data | `bookdb dev_setup` |
| `inspect` | Show system inspection info | `bookdb inspect` |
| `checksum` | Generate system checksum | `bookdb checksum` |
| `rc` | Show RC file info | `bookdb rc` |
| `sanity` | Quick installation state check | `bookdb sanity` |

### Developer '

### Meta Commands
| Command | Description | Example |
|---------|-------------|---------|
| `help` / `usage` | Show help text | `bookdb help` |
| `version` / `vers` | Show version info | `bookdb version` |
| `noop` | No operation (testing) | `bookdb noop test` |

## Command Flags

### Global Flags
| Flag | Long Form | Description |
|------|-----------|-------------|
| `-y` | `--yes` | Auto-confirm 'yes' to all prompts |
| `-p` | `--projdb <name>` | Set project namespace for command |
| `-k` | `--keydb <name>` | Set key-value namespace for command |
| `-d` | `--debug` | Enable debug output |
| `-t` | `--trace` | Enable trace output |
| `-q` | `--quiet` | Quiet mode |
| `-f` | `--force` | Force operation |
| `-D` | `--dev` | Enable development mode |

### Command-Specific Flags
| Command | Flag | Description |
|---------|------|-------------|
| `backup` | `--all` | Backup all bases instead of current |
| `migrate` | `--all` | Export all keystores |
| `reset` | `--soft` | Soft reset (cursors/config only) |
| `reset` | `--alias <name>` | Reset specific alias |

## Context Chain Syntax

### Full Context Chain Patterns
```bash
# Base persistence (cursor changes)
bookdb base@project.VAR.keystore getv KEY

# Read-only context (cursor unchanged)  
bookdb base%project.VAR.keystore getv KEY

# Current base with persistence
bookdb @project.VAR.keystore getv KEY

# Current base, read-only
bookdb %project.VAR.keystore getv KEY
```

### Document Store Context
```bash
# Document operations
bookdb base@project.DOC.document_name getd
bookdb base@project.DOC.document_name setd < file.txt
```

## Multi-Base Architecture

### Base Operations
- **Creating**: `bookdb new base <name>`
- **Selecting**: `bookdb select <name>` 
- **Listing**: `bookdb ls bases`
- **Current**: `bookdb base`
- **Destroying**: `bookdb unbase <name>` (DANGEROUS)
- **Recreating**: `bookdb rebase <name>` (DANGEROUS)

### Base Persistence
- Current active base stored in cursor files
- Context chains can reference any base via `base@` syntax
- Default base is `home` for new installations

## Special Behaviors

### Automatic Installation
- DEV_MODE enables auto-installation on first run
- Production requires explicit `bookdb install`

### Confirmation Prompts
- Dangerous operations require confirmation unless `-y` flag
- `reset`, `rebase`, `unbase`, `del` commands protected

### Error Handling
- Alias detection prevents certain operations
- Validation for numeric operations (incv/decv)
- Database existence checks before operations

### Development Features
- `$` prefix for dev sub-calls
- `#` prefix for dev driver calls  
- DEV_MODE enables additional functionality
- Inspection and debugging commands

## File Structure Integration
- XDG+ compliant directory structure
- Shell profile integration for installation
- RC file management for state persistence
- Backup/restore functionality with tar compression Command Surface
The `# BookDB Command Surface Documentation

## Overview
This document catalogs the complete command surface of BookDB.sh to ensure feature parity in the Rust port. All commands, flags, and behaviors are documented for reference.

## Core Commands

### Information & Status Commands
| Command | Aliases | Description | Example |
|---------|---------|-------------|---------|
| `status` | `st` | Display dashboard with system state and projects | `bookdb status` |
| `cursor` | `c` | Print current active cursor chain | `bookdb cursor` |
| `base` | - | Show current active base | `bookdb base` |
| `ls` | - | List data (bases/projects/keys/etc) | `bookdb ls all` |
| `find` | - | Find a key across all projects/keystores | `bookdb find API_KEY` |

### Variable Operations (CRUD)
| Command | Description | Example |
|---------|-------------|---------|
| `getv` | Get the value of a variable | `bookdb getv API_KEY` |
| `setv` | Create or update a variable | `bookdb setv API_KEY=secret123` |
| `delv` | Delete a variable | `bookdb delv API_KEY` |
| `incv` | Increment a numerical value | `bookdb incv COUNT 5` |
| `decv` | Decrement a numerical value | `bookdb decv COUNT 2` |

### Document Operations (Not yet in Rust port)
| Command | Description | Example |
|---------|-------------|---------|
| `getd` | Get a document | `bookdb getd README` |
| `setd` | Set a document | `bookdb setd README < file.md` |
| `deld` | Delete a document | `bookdb deld README` |

### Base Management (Multi-base Support)
| Command | Description | Example |
|---------|-------------|---------|
| `select` | Set persistent active base | `bookdb select work` |
| `rebase` | **DANGER**: Delete and recreate a base | `bookdb rebase temp` |
| `unbase` | **DANGER**: Delete a base completely | `bookdb unbase old_base` |

### Creation & Deletion
| Command | Description | Example |
|---------|-------------|---------|
| `new` | Create new base/project/keystore | `bookdb new base work` |
| `del` | Delete project/keystore | `bookdb del project old_proj` |

### File Publishing
| Command | Description | Example |
|---------|-------------|---------|
| `pub` | Publish key to external file | `bookdb pub API_KEY ~/.env` |
| `unpub` | Remove key from external file | `bookdb unpub API_KEY ~/.env` |

### Import/Export & Backup
| Command | Description | Example |
|---------|-------------|---------|
| `export` | Export keystore to .env file | `bookdb export keystore` |
| `import` | Import keys from .env file | `bookdb import config.env` |
| `backup` | Create backup (manual) | `bookdb backup --all` |
| `migrate` | Export all keystores to backup | `bookdb migrate --all` |

### Administrative Commands
| Command | Description | Example |
|---------|-------------|---------|
| `install` | Install/update bookdb application | `bookdb install` |
| `reset` | **DANGER**: Delete all data and artifacts | `bookdb reset` |
| `sanity` | Quick installation state check | `bookdb sanity` |

 command provides direct access to internal functions for development and testing:

| Function | Description | Example |
|----------|-------------|---------|
| `dev_create_base` | **[KEY]** Create a new base file (.sqlite3) | `bookdb $ dev_create_base test_base` |
| `dev_rem_base` | **[KEY]** Remove base file(s) with filtering | `bookdb $ dev_rem_base` |
| `dev_dump_bases` | List all available bases | `bookdb $ dev_dump_bases` |
| `dev_show_sqlite` | Show SQLite stream info | `bookdb $ dev_show_sqlite` |
| `random_pick_array` | **[KEY]** Pick random element from array | `bookdb $ random_pick_array BASE_LIST` |
| `super_substring_filter` | **[KEY]** Advanced filtering with include/exclude patterns | `bookdb $ super_substring_filter "%test" "!prod"` |
| `stream_array` | Stream array contents | `bookdb $ stream_array some_array` |
| `array_diff` | Calculate array set difference | `bookdb $ array_diff arr1 arr2 result` |
| `list_filter` | Add bullet points to stream | `bookdb $ list_filter` |
| `color_filter` | Colorize stream output | `bookdb $ color_filter red` |
| `noop_filter` | Test filter (no-op) | `bookdb $ noop_filter` |

### Developer '#' Command Surface
The `#` command provides another dev dispatcher for inspection functions:

| Function | Description | Example |
|----------|-------------|---------|
| `__low_inspect` | Low-level system inspection | `bookdb # __low_inspect` |

### Key Development Utilities

#### **Multi-Base Testing Functions**
- **`dev_create_base`**: Creates new .sqlite3 base files in the data directory
- **`dev_rem_base`**: Interactive base deletion with filtering and safety checks
- **`random_pick_array`**: Selects random names from BASE_LIST for testing

#### **Advanced Filtering System**
- **`super_substring_filter`**: Hierarchical pattern matching with precedence rules
  - Exclusions (`!`, `!#`) always beat inclusions (`%`, `#`)
  - Exact matches (`#`) override fuzzy matches (`%`)
  - Supports complex filter combinations

#### **Array Utilities**
- **`stream_array`**: Converts arrays to streamable format
- **`array_diff`**: Set operations for array manipulation
- **Safety validation**: Ensures array references are valid before processing

#### **Visual Utilities**
- **`color_filter`**: Apply colors to stream output
- **`list_filter`**: Add bullet formatting to streams
- **Column formatting**: Integration with `pr` and `column` commands

### Meta Commands
| Command | Description | Example |
|---------|-------------|---------|
| `help` / `usage` | Show help text | `bookdb help` |
| `version` / `vers` | Show version info | `bookdb version` |
| `noop` | No operation (testing) | `bookdb noop test` |

## Command Flags

### Global Flags
| Flag | Long Form | Description |
|------|-----------|-------------|
| `-y` | `--yes` | Auto-confirm 'yes' to all prompts |
| `-p` | `--projdb <name>` | Set project namespace for command |
| `-k` | `--keydb <name>` | Set key-value namespace for command |
| `-d` | `--debug` | Enable debug output |
| `-t` | `--trace` | Enable trace output |
| `-q` | `--quiet` | Quiet mode |
| `-f` | `--force` | Force operation |
| `-D` | `--dev` | Enable development mode |

### Command-Specific Flags
| Command | Flag | Description |
|---------|------|-------------|
| `backup` | `--all` | Backup all bases instead of current |
| `migrate` | `--all` | Export all keystores |
| `reset` | `--soft` | Soft reset (cursors/config only) |
| `reset` | `--alias <name>` | Reset specific alias |

## Context Chain Syntax

### Full Context Chain Patterns
```bash
# Base persistence (cursor changes)
bookdb base@project.VAR.keystore getv KEY

# Read-only context (cursor unchanged)  
bookdb base%project.VAR.keystore getv KEY

# Current base with persistence
bookdb @project.VAR.keystore getv KEY

# Current base, read-only
bookdb %project.VAR.keystore getv KEY
```

### Document Store Context
```bash
# Document operations
bookdb base@project.DOC.document_name getd
bookdb base@project.DOC.document_name setd < file.txt
```

## Multi-Base Architecture

### Base Operations
- **Creating**: `bookdb new base <name>`
- **Selecting**: `bookdb select <name>` 
- **Listing**: `bookdb ls bases`
- **Current**: `bookdb base`
- **Destroying**: `bookdb unbase <name>` (DANGEROUS)
- **Recreating**: `bookdb rebase <name>` (DANGEROUS)

### Base Persistence
- Current active base stored in cursor files
- Context chains can reference any base via `base@` syntax
- Default base is `home` for new installations

## Special Behaviors

### Automatic Installation
- DEV_MODE enables auto-installation on first run
- Production requires explicit `bookdb install`

### Confirmation Prompts
- Dangerous operations require confirmation unless `-y` flag
- `reset`, `rebase`, `unbase`, `del` commands protected

### Error Handling
- Alias detection prevents certain operations
- Validation for numeric operations (incv/decv)
- Database existence checks before operations

### Development Features
- `$` prefix for dev sub-calls
- `#` prefix for dev driver calls  
- DEV_MODE enables additional functionality
- Inspection and debugging commands

## File Structure Integration
- XDG+ compliant directory structure
- Shell profile integration for installation
- RC file management for state persistence
- Backup/restore functionality with tar compression