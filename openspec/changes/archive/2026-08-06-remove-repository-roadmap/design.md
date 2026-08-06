## Context

The repository currently maintains product direction in `roadmap.yaml`, generates `ROADMAP.md`, and ships validation/rendering logic inside the product CLI. The same roadmap has now been migrated to GitHub Project #1 as repository issues with stable IDs, sequence, theme, lifecycle, dependencies, exclusions, outcomes, and OpenSpec evidence.

## Goals / Non-Goals

**Goals:**

- Establish GitHub Project #1 as the sole roadmap and prioritization surface.
- Remove all custom roadmap runtime code, generated artifacts, tests, CI hooks, and unused dependencies.
- Preserve OpenSpec for concrete proposal, design, specification, task, and archive history.
- Leave the product CLI focused on simulator behavior and developer harness commands.

**Non-Goals:**

- Delete or reorganize pre-existing items in GitHub Project #1.
- Automate synchronization between GitHub Projects and OpenSpec.
- Remove historical archived OpenSpec artifacts that explain the former roadmap implementation.
- Redesign unrelated CLI behavior.

## Decisions

### Use GitHub issues as canonical project items

Each migrated roadmap entry is a repository issue added to Project #1. Issues retain discussion and durable URLs, while project fields provide Roadmap ID, Sequence, Theme, Lifecycle, and Status. Draft items were rejected because they are less discoverable from repository history and do not provide normal issue linking.

### Remove rather than deprecate the CLI surface

The roadmap commands exist only to maintain the repository-local representation. Keeping compatibility shims would retain dependencies and ambiguity after ownership moves to GitHub, so the commands and module are deleted together.

### Preserve historical OpenSpec artifacts

The archived `add-roadmap-tracking` change remains as immutable design history. The synchronized main capability is removed through this change's delta spec so the active specification no longer claims retired behavior.

### Keep only a concise contributor pointer

README documentation will link to GitHub Project #1 and state that OpenSpec governs concrete delivery changes. Detailed lifecycle validation rules are removed because GitHub Project configuration now owns those fields.

## Risks / Trade-offs

- **GitHub roadmap state is unavailable offline** → OpenSpec artifacts and implementation remain local; only portfolio planning requires GitHub.
- **Project fields are not CI-validated** → Prefer the lower maintenance cost and native project UX; review roadmap metadata in GitHub.
- **Old roadmap issues can drift from OpenSpec delivery state** → Link active changes and archived evidence in issue bodies during normal planning review.
- **Historical docs mention removed commands** → Preserve archived change history intentionally, while removing commands from current documentation and specs.

## Migration Plan

1. Verify all eleven roadmap issues and metadata in GitHub Project #1.
2. Remove root roadmap artifacts, CLI module and commands, tests/fixtures, CI hooks, and documentation.
3. Remove roadmap-only dependencies and regenerate the lockfile through Cargo.
4. Validate formatting, linting, tests, and OpenSpec artifacts.
5. Sync removal of the `roadmap-management` main spec and archive this change.

Rollback would restore current files from version control and treat Project #1 as a mirror again.

## Open Questions

None.
