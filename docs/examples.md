# Why CLI: Usage Examples

This document provides detailed examples of how to use the `why` CLI tool for analyzing dependencies in your projects.

## Basic Analysis

### Analyzing the Current Project

To analyze the dependencies in your current project, simply run:

```bash
why analyze
```

This will:
1. Automatically detect your project type (Rust or Node.js)
2. Parse your manifest file (Cargo.toml or package.json)
3. Analyze how dependencies are used in your codebase
4. Display the results in an interactive TUI interface

### Analyzing a Specific Project

To analyze a project in a different directory:

```bash
why analyze --path /path/to/your/project
```

### Focusing on a Specific Dependency

If you're interested in a particular dependency:

```bash
why analyze --dep serde
```

This will show detailed information about how the "serde" dependency is used throughout your project.

## Exporting Results

### Exporting to JSON

To export the analysis results to a JSON file:

```bash
why export --output dependencies.json
```

This creates a structured JSON file containing all dependency data, which is useful for:
- Sharing with team members
- Further processing with other tools
- Storing historical data for tracking dependency usage over time

Example JSON output:

```json
{
  "dependencies": [
    {
      "name": "serde",
      "version": "1.0.152",
      "usage_count": 15,
      "importance_score": 0.85,
      "removable": false,
      "used_features": ["derive", "std"],
      "unused_features": []
    },
    {
      "name": "unused-dep",
      "version": "0.1.0",
      "usage_count": 0,
      "importance_score": 0.0,
      "removable": true,
      "used_features": [],
      "unused_features": []
    }
  ]
}
```

### Exporting to CSV

For spreadsheet processing or data analysis:

```bash
why export --output dependencies.csv --format csv
```

Example CSV output:
```
Dependency,Version,Usage Count,Importance Score,Removable
serde,1.0.152,15,0.85,false
unused-dep,0.1.0,0,0.0,true
```

### Exporting for a Specific Dependency

To export analysis for a single dependency:

```bash
why export --dep tokio --output tokio-analysis.json
```

## Advanced Usage

### Analyzing a Rust Workspace

For a Rust workspace with multiple crates:

```bash
why analyze --path /path/to/workspace/root
```

This will recursively analyze all crates in the workspace and show the aggregated dependency usage.

### Comparing Dependency Usage Across Projects

You can use the export feature to compare how dependencies are used across different projects:

```bash
# Export project A
cd /path/to/project-a
why export --output project-a-deps.json

# Export project B
cd /path/to/project-b
why export --output project-b-deps.json

# Now you can compare these files using diff tools or custom scripts
```

## Using Analysis Results

The dependency analysis can help you:

1. **Identify Unused Dependencies**: Dependencies with a usage count of 0 are likely candidates for removal.

2. **Find Minimally Used Dependencies**: Dependencies with very few imports might be replaceable with simpler solutions.

3. **Optimize Feature Flags**: The tool shows which features of dependencies are actually used, allowing you to minimize your feature set.

4. **Prioritize Maintenance**: The importance score helps you focus maintenance efforts on the most critical dependencies.

5. **Understand Adoption Patterns**: See which dependencies are widely used throughout your codebase vs. those only used in specific modules.

## TUI Navigation

Within the TUI interface:

- **Tab**: Switch between overview and detailed view
- **↑/↓ arrows**: Navigate through dependencies
- **Enter**: Show detailed information for the selected dependency
- **q**: Quit the application
- **h or ?**: Show help

## Batch Processing

For CI/CD pipelines or batch processing multiple projects:

```bash
#!/bin/bash
# Example script to process multiple projects

PROJECTS=("/path/to/project1" "/path/to/project2" "/path/to/project3")

for project in "${PROJECTS[@]}"; do
  echo "Analyzing $project"
  why export --path "$project" --output "$project/dependency-analysis.json"
done
``` 