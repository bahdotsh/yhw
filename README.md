# Why

`why` is a CLI tool that helps developers understand their project dependencies by analyzing how and where each dependency is used. This tool aims to provide insights into dependency usage patterns, identify unused dependencies, and help maintain a cleaner, more efficient codebase.

## Features

- Analyzes project dependencies from various ecosystems:
  - Rust projects (Cargo.toml)
  - Node.js projects (package.json)
- Scans project files to identify where dependencies are imported and used
- Calculates dependency usage metrics (frequency, importance, etc.)
- Identifies unused or minimally used dependencies
- Presents findings in an interactive TUI interface
- Exports dependency analysis to different formats (JSON, CSV)
- Integrates with cargo-deps for dependency graph visualization

## Installation

### From Source

```bash
git clone https://github.com/yourusername/why.git
cd why
cargo install --path .
```

### From Crates.io (Coming Soon)

```bash
cargo install why
```

## Usage

### Basic Usage

Run `why` in your project directory:

```bash
# Analyze in the current directory
why analyze

# Specify a project path
why analyze --path /path/to/your/project

# Show detailed information for a specific dependency
why analyze --dep serde
```

### Exporting Analysis Results

Export the analysis results to JSON or CSV:

```bash
# Export to JSON (default)
why export --output dependencies.json

# Export to CSV
why export --output dependencies.csv --format csv

# Export analysis for a specific dependency
why export --dep serde --output serde-analysis.json
```

### Supported Project Types

- **Rust Projects**: Place `why` in a directory with a `Cargo.toml` file
- **Node.js Projects**: Place `why` in a directory with a `package.json` file

### TUI Navigation

The TUI interface supports the following key bindings:

- `↑`/`↓`: Navigate up/down in the list
- `Tab`: Switch between views
- `Enter`: Show detailed information for the selected dependency
- `q`: Quit the application
- `?`: Show help

## Screenshots

(Coming soon)

## Development

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/why.git
cd why

# Build the project
cargo build

# Run tests
cargo test
```

### Project Structure

See [plan.md](plan.md) for detailed information about the project structure and development roadmap.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Roadmap

- Support for more ecosystems:
  - Python (requirements.txt, pyproject.toml)
  - Java/Kotlin (gradle, maven)
- Enhanced visualization of dependency relationships
- Web interface for larger projects
- Continuous monitoring mode
- Integration with CI/CD pipelines 