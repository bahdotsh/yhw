# Why - Dependency Analysis CLI Tool

## Project Overview
"Why" is a CLI tool that analyzes project dependencies to help users understand:
- Where and how each dependency is used
- The importance of each dependency to the project
- Which dependencies can potentially be removed
- How extensively each dependency's features are utilized

Initially focusing on Rust projects (Cargo.toml), with plans to extend to other ecosystems.

## Features

### Core Features
- [x] Parse Cargo.toml files to extract dependencies
- [ ] Scan project files to identify where dependencies are imported/used
- [ ] Calculate dependency usage metrics (frequency, breadth, etc.)
- [ ] Identify unused or minimally used dependencies
- [ ] Generate comprehensive dependency usage reports
- [ ] Present findings in an interactive TUI interface using ratatui
- [ ] Integrate with cargo-deps for dependency graph visualization

### Future Extensions
- [ ] Support for other ecosystem manifest files (package.json, requirements.txt, etc.)
- [ ] Dependency graph visualization
- [ ] Detect transitive dependencies
- [ ] Suggest dependency alternatives
- [ ] Export reports to different formats (JSON, CSV, etc.)

## Technical Architecture

### Components

1. **Manifest Parser**
   - [ ] Parse Cargo.toml files
   - [ ] Extract dependencies, versions, and features
   - [ ] Handle workspace configurations
   
2. **Code Analyzer**
   - [ ] Scan Rust source files
   - [ ] Detect import statements and usage patterns
   - [ ] Track which modules/functions from dependencies are used
   - [ ] Analyze usage frequency and importance
   - [ ] Leverage cargo-deps for dependency graph generation
   
3. **Metrics Calculator**
   - [ ] Compute usage statistics for each dependency
   - [ ] Score dependencies based on usage patterns
   - [ ] Identify candidates for removal
   
4. **TUI Interface**
   - [ ] Design an intuitive terminal UI using ratatui
   - [ ] Create different views (overview, detailed, etc.)
   - [ ] Implement navigation and interaction
   - [ ] Add filters and sorting options

### Codebase Structure

```
src/
├── main.rs                 # Entry point
├── cli/                    # CLI argument handling
│   ├── mod.rs
│   └── args.rs
├── manifest/               # Manifest file parsing
│   ├── mod.rs
│   └── cargo.rs            # Cargo.toml specific parsing
├── analyzer/               # Code analysis
│   ├── mod.rs
│   ├── rust_analyzer.rs    # Rust specific code analysis
│   └── metrics.rs          # Dependency metrics calculation
├── tui/                    # Terminal UI
│   ├── mod.rs
│   ├── app.rs              # App state
│   ├── ui.rs               # UI components
│   ├── event.rs            # Event handling
│   └── views/              # Different views
│       ├── mod.rs
│       ├── overview.rs     # Overview of all dependencies
│       └── details.rs      # Detailed view of a dependency
└── utils/                  # Utility functions
    ├── mod.rs
    └── fs.rs               # File system utilities
```

## Dependencies

### Core Dependencies
- `clap`: Command line argument parsing
- `toml`: For parsing Cargo.toml files
- `syn`: For parsing Rust code
- `walkdir`: For traversing the project directory
- `ratatui`: For building the terminal UI
- `crossterm`: Terminal manipulation (used by ratatui)
- `serde`: For data serialization/deserialization

### Development Dependencies
- `pretty_assertions`: For more readable test assertions
- `tempfile`: For creating temporary files during tests
- `test-case`: For parameterized testing

## Implementation Plan

### Phase 1: Foundation and Basic Analysis
- [ ] Set up project structure and dependencies
- [ ] Implement basic CLI interface with clap
- [ ] Create Cargo.toml parser
- [ ] Implement file system traversal
- [ ] Basic Rust code parsing for imports
- [ ] Simple dependency usage detection

### Phase 2: Advanced Analysis
- [ ] Implement detailed usage tracking
- [ ] Add metrics calculation
- [ ] Identify unused dependencies
- [ ] Detect partially used dependencies
- [ ] Calculate importance scores
- [ ] Track which specific items from a dependency are used
- [ ] Integrate with cargo-deps for dependency graph visualization

### Phase 3: TUI Development
- [ ] Set up basic TUI structure with ratatui
- [ ] Implement overview view of all dependencies
- [ ] Create detailed view for individual dependencies
- [ ] Add navigation between views
- [ ] Implement sorting and filtering
- [ ] Design and implement usage visualizations

### Phase 4: Refinement and Extensions
- [ ] Add configuration options
- [ ] Improve performance for large codebases
- [ ] Add report generation
- [ ] Support for workspaces
- [ ] Initial support for another ecosystem (e.g., Node.js)
- [ ] Add comprehensive documentation

## Detailed Task Breakdown

### 1. Project Setup
- [x] Initialize new Rust project
- [ ] Set up GitHub repository
- [x] Add initial dependencies to Cargo.toml
- [x] Create README.md with project description
- [x] Set up basic project structure

### 2. Manifest Parsing
- [ ] Implement Cargo.toml parser
- [ ] Extract normal dependencies
- [ ] Extract dev-dependencies
- [ ] Extract build-dependencies
- [ ] Handle feature flags
- [ ] Support for workspace configurations
- [ ] Add tests for parser

### 3. Code Analysis
- [ ] Implement project file traversal
- [ ] Create Rust source file parser
- [ ] Detect and track `use` statements
- [ ] Map `use` statements to dependencies
- [ ] Analyze function calls to dependencies
- [ ] Track macro usage from dependencies
- [ ] Calculate usage statistics
- [ ] Integrate with cargo-deps for dependency relationship information
- [ ] Add tests for analyzer

### 4. Metrics and Scoring
- [ ] Define metrics for dependency importance
- [ ] Implement usage frequency calculation
- [ ] Calculate code coverage percentage for each dependency
- [ ] Score dependencies based on their usage
- [ ] Identify unused dependencies
- [ ] Highlight minimally used dependencies
- [ ] Add tests for metrics

### 5. TUI Implementation
- [ ] Set up basic TUI application structure
- [ ] Implement event handling
- [ ] Create app state management
- [ ] Design and implement overview screen
- [ ] Design and implement detailed dependency view
- [ ] Add navigation between different views
- [ ] Implement sorting and filtering
- [ ] Add usage visualization components
- [ ] Add tests for TUI components

### 6. CLI Interface
- [ ] Set up command-line argument parsing
- [ ] Implement different commands and options
- [ ] Add help text and documentation
- [ ] Create verbose output mode
- [ ] Add path specification options
- [ ] Support for configuration files
- [ ] Add tests for CLI

### 7. Documentation and Polishing
- [ ] Write comprehensive documentation
- [ ] Add usage examples
- [ ] Create user guide
- [ ] Performance optimization
- [ ] Error handling improvements
- [ ] Add logging
- [ ] Final testing and bug fixes

## Testing Strategy

### Unit Tests
- [ ] Test manifest parsing
- [ ] Test code analysis functions
- [ ] Test metrics calculations
- [ ] Test TUI components

### Integration Tests
- [ ] Test full analysis pipeline
- [ ] Test CLI interface
- [ ] Test with sample projects

### Test Projects
- [ ] Simple project with few dependencies
- [ ] Complex project with many dependencies
- [ ] Project with workspace configuration

## Future Roadmap

### Short-term
- [ ] Support for other Rust manifest formats (Cargo.toml alternatives)
- [ ] Enhanced visualization of dependency relationships
- [ ] Plugin system for custom analyzers

### Medium-term
- [ ] Support for Node.js (package.json)
- [ ] Support for Python (requirements.txt, pyproject.toml)
- [ ] Support for Java/Kotlin (gradle, maven)
- [ ] Export findings to different formats

### Long-term
- [ ] Web interface
- [ ] Continuous monitoring mode
- [ ] Integration with CI/CD pipelines
- [ ] Vulnerability scanning
- [ ] License compliance checking

## Progress Tracking

- [x] Phase 1 completed
  - [x] Project setup
  - [x] Basic CLI interface with clap
  - [x] Cargo.toml parser
  - [x] File system traversal
  - [x] Basic Rust code parsing for imports 
  - [x] Simple dependency usage detection
- [x] Phase 2 completed
  - [x] Implement detailed usage tracking
  - [x] Add metrics calculation
  - [x] Identify unused dependencies
  - [x] Detect partially used dependencies
  - [x] Calculate importance scores
  - [x] Track which specific items from a dependency are used
  - [x] Integrate with dependency graph visualization
- [x] Phase 3 completed
  - [x] Set up basic TUI structure with ratatui
  - [x] Implement overview view of all dependencies
  - [x] Create detailed view for individual dependencies
  - [x] Add navigation between views
  - [x] Implement sorting and filtering
  - [x] Design and implement usage visualizations
- [x] Phase 4 completed
  - [x] Add configuration options
  - [x] Add export to different formats
  - [x] Support for Node.js ecosystem
  - [x] Add comprehensive documentation
- [ ] Version 1.0.0 released

## Current Task List

- [x] Set up project structure and dependencies
- [x] Create initial project files (README.md, plan.md)
- [x] Create module structure (analyzer, cli, manifest, tui, utils)
- [x] Implement basic CLI argument handling
- [x] Complete Cargo.toml manifest parser
- [x] Complete Rust code analyzer
- [x] Fix linter errors in existing code
- [x] Implement simple TUI with ratatui
- [x] Add tests for core functionality
- [x] Enhance metrics calculation
- [x] Implement advanced code analysis
- [x] Integrate dependency graph visualization
- [x] Enhance TUI with detailed usage information
- [x] Implement TUI navigation between views
- [x] Implement sorting and filtering in TUI
- [x] Add comprehensive documentation
- [x] Add export to different formats
- [x] Support for other ecosystems
- [x] Add configuration options
- [ ] Final testing and bug fixes
- [ ] Release version 1.0.0

## Notes and Considerations

- Performance might be an issue for very large codebases
- Accurate detection of indirect usage might be challenging
- Need to handle conditional compilation (cfg features)
- Consider integration with existing tools like cargo-deps
- For Rust projects, leverage cargo-deps to build the initial dependency graph and enhance it with usage information 