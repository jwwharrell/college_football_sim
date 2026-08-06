## REMOVED Requirements

### Requirement: Structured roadmap is authoritative
**Reason**: GitHub Project #1 now owns roadmap state, so repository-local YAML is no longer authoritative.
**Migration**: Manage roadmap items and fields at https://github.com/users/jwwharrell/projects/1.

### Requirement: Roadmap items have stable identity and ordered outcomes
**Reason**: Stable IDs, ordering, and outcomes are now represented by GitHub issues and project fields.
**Migration**: Use the Roadmap ID and Sequence fields and each issue's Outcome section.

### Requirement: Dependencies are valid and acyclic
**Reason**: The custom repository validator is being retired with the YAML roadmap.
**Migration**: Record dependencies in roadmap issue bodies and review them during prioritization.

### Requirement: Lifecycle states require repository evidence
**Reason**: GitHub Project fields now represent lifecycle, while issue bodies link OpenSpec evidence.
**Migration**: Use the Lifecycle field and update the OpenSpec evidence section of each roadmap issue.

### Requirement: Lifecycle transitions are documented
**Reason**: Roadmap lifecycle management has moved to the GitHub Project workflow.
**Migration**: Move project items through the configured Lifecycle and Status fields.

### Requirement: Human roadmap is deterministic and drift-free
**Reason**: GitHub Project #1 is the human-readable roadmap, eliminating the generated Markdown mirror.
**Migration**: View and update roadmap state directly in GitHub Project #1.

### Requirement: Roadmap commands are safe and diagnostic
**Reason**: Repository roadmap validation, rendering, and checking are no longer needed.
**Migration**: Remove use of the `roadmap validate`, `roadmap render`, and `roadmap check` CLI commands.

### Requirement: Initial dynasty roadmap is captured
**Reason**: All eleven initial roadmap entries have been migrated to repository issues in GitHub Project #1.
**Migration**: Use issues #6 through #16 and their project metadata as the migrated roadmap.

### Requirement: Verification detects roadmap drift
**Reason**: There is no generated repository roadmap after migration, so CI drift checks have no target.
**Migration**: Remove roadmap validation and generated-document checks from repository verification.
