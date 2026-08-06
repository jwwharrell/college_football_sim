## Why

Product roadmap state has been migrated to GitHub Project #1, where prioritization, lifecycle, and discussion can be managed without maintaining custom repository tooling. Keeping the YAML model, generated Markdown, CLI commands, and CI checks would duplicate that external source and impose unnecessary maintenance.

## What Changes

- **BREAKING** Remove the repository-local `roadmap.yaml` source and generated `ROADMAP.md` view.
- **BREAKING** Remove the CLI `roadmap validate`, `roadmap render`, and `roadmap check` commands and their implementation.
- Remove roadmap-specific tests, fixtures, CI checks, documentation, and dependencies that are no longer used.
- Point contributors to GitHub Project #1 for roadmap direction while retaining OpenSpec as the authority for concrete requirements and delivery plans.
- Remove the `roadmap-management` capability because its required behavior is intentionally retired.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `roadmap-management`: Remove all requirements for repository-local roadmap storage, validation, rendering, lifecycle evidence, and verification.

## Impact

- Deletes roadmap artifacts at the repository root and roadmap code from the CLI crate.
- Simplifies the CLI surface and repository verification workflow.
- May remove YAML deserialization dependencies if no remaining code uses them.
- Moves roadmap ownership to https://github.com/users/jwwharrell/projects/1; OpenSpec remains repository-local for scoped changes.
