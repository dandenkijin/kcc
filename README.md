# Kernel Config Checker (kcc)

A fast and flexible command-line tool to check Linux kernel configuration flags against required settings for container runtimes and other system features.

## Features

- **Fast Rust Implementation**: Compiled binary for high performance
- **Flexible Input Formats**: Supports both `FLAG` and `FLAG=value` formats in flag files
- **Colored Output**: Color-coded results for easy readability (can be disabled)
- **Multiple Flag Files**: Check against multiple kernel requirement files
- **Compressed Config Support**: Automatically handles `/proc/config.gz` and other compressed configs
- **Exit Codes**: Returns proper exit codes for automation (0=success, 1=missing flags)

## Usage

### Basic Usage

```bash
# Check current kernel config against default flags
kcc

# Check specific kernel config file
kcc -c /path/to/.config

# Check against custom flags file
kcc -f /path/to/flags.txt

# Check custom config against custom flags (with no colors)
kcc -c /path/to/.config -f /path/to/flags.txt -n
```

### Command Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `-c, --config <CONFIG>` | Path to kernel config file | `/proc/config.gz` |
| `-f, --flags <FILE>` | Path to flags file (can be specified multiple times) | Required |
| `--set-flags <FLAGS>` | Specific kernel config flags to check (comma-separated) | Optional |
| `--set` | Set flags from file (adds missing flags to .config file) | Optional |
| `-n, --no-color` | Disable colored output | `false` |
| `-h, --help` | Print help information | - |
| `-V, --version` | Print version information | - |

### Flag File Formats

The tool supports two formats in flag files:

#### Simple Format (One flag per line)
```
NAMESPACES
NET_NS
PID_NS
CGROUPS
MEMCG
```

#### Key-Value Format (Flag with value)
```
CONFIG_NAMESPACES=y
CONFIG_NET_NS=y
CONFIG_PID_NS=y
CONFIG_CGROUPS=m
```

#### Mixed Format (Both formats supported)
```
NAMESPACES
CONFIG_NET_NS=y
PID_NS
# This is a comment
CGROUPS=m
```

## Examples

### Example 1: Basic Container Runtime Check

```bash
kcc -f flags.txt
```

### Example 2: Docker Compatibility Check

```bash
kcc -f flags-docker.txt
```

### Example 3: Custom Kernel Config Check

```bash
kcc -c /boot/config-$(uname -r) -f my-flags.txt
```

## Output Format

- ✅ **Green**: Flag is enabled in the kernel
- ✅ **Green (as module)**: Flag is enabled as a loadable module
- ❌ **Red**: Flag is missing/not enabled
- ⚠️ **Yellow**: Flag doesn't exist in kernel configuration options

### Sample Output

```
🔍 Kernel Config Checker - Checking kernel configuration flags from: /proc/config.gz
📋 Reading flags from files: flags.txt, docker-flags.txt

✅ CONFIG_NAMESPACES
✅ CONFIG_CGROUPS (as module)
❌ CONFIG_USER_NS
⚠️ CONFIG_INVALID_FLAG (invalid flag)

⚠️ Flags in your list that don't exist in kernel config options:
   - CONFIG_INVALID_FLAG

❌ Some required kernel flags are missing!
```

## Enhanced Flag Validation

The tool now includes enhanced validation that detects:

1. **Missing Flags**: Flags that exist in the kernel configuration but are not enabled in your current config
2. **Invalid Flags**: Flags that don't exist in the kernel configuration options at all

### Flag Validation Benefits

- **Early Detection**: Catch typos and invalid flag names before deployment
- **Better Feedback**: Clear distinction between missing and invalid flags
- **Suggestion Engine**: Automatically suggests using `--set` to add missing flags to your config

## Flag Files

The repository includes several example flag files:

- `flags.txt` - Basic container runtime requirements
- `flags-docker.txt` - Docker-specific kernel requirements
- `sound-flags.txt` - Audio/sound system requirements

### Custom Flag Files

Create your own flag files to check for specific kernel features:

```bash
# Create a custom flag file
cat > my-requirements.txt << EOF
# Required for my application
NAMESPACES
CGROUPS
NET_NS
PID_NS
# Optional but recommended
USER_NS
SECCOMP
EOF

# Check against your requirements
kcc -f my-requirements.txt
```

## Exit Codes

| Exit Code | Meaning |
|-----------|---------|
| `0` | All required flags are enabled |
| `1` | One or more required flags are missing |
| `2` | Invalid command line arguments |
| `3` | File I/O errors (config or flags file not found) |
| `4` | Invalid kernel flags (non-existent configuration options) |

## Development

### Prerequisites

- Rust toolchain (latest stable)
- Cargo (comes with Rust)

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt
```


## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- 📋 [Documentation](https://github.com/yourusername/kcc/wiki)
- 🐛 [Issue Tracker](https://github.com/yourusername/kcc/issues)
- 💬 [Discussions](https://github.com/yourusername/kcc/discussions)

## Changelog

### v0.1.0
- Initial release
- Support for both simple and key-value flag formats
- Colored output with configurable disabling
- Multiple flag file support
- Compressed kernel config support
- Enhanced flag validation (detects invalid/non-existent kernel flags)
- Improved error reporting and suggestions
