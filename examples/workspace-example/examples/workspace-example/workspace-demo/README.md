# workspace-demo

A Java workspace managed with jbuild.

## Structure

This workspace contains multiple Java projects. Each project should be in its own subdirectory.

## Building

```bash
# Build all projects in the workspace
jbuild workspace build

# Build specific goals
jbuild workspace build compile test
```

## Adding Projects

```bash
# Create a new project
jbuild new my-project

# Or initialize an existing project
cd existing-project
jbuild init
jbuild workspace add ../existing-project
```

## Commands

- `jbuild workspace list` - List all workspace members
- `jbuild workspace add <path>` - Add a project to the workspace
- `jbuild workspace remove <path>` - Remove a project from the workspace
- `jbuild workspace build` - Build all workspace members
