## 1. Structured Roadmap Model

- [x] 1.1 Add the CLI-local roadmap module and workspace-managed YAML deserialization dependency with strict, versioned schema types that deny unknown fields.
- [x] 1.2 Define typed roadmap items, lifecycle statuses, evidence references, dependencies, outcomes, exclusions, and optional deferral reasons.
- [x] 1.3 Add fixture-based tests for valid YAML, unsupported versions, missing required fields, unknown fields, and ambiguous or malformed scalar input.

## 2. Semantic and Dependency Validation

- [x] 2.1 Implement deterministic aggregated validation errors for stable identifier format, non-empty content, positive unique sequences, and unique identifiers.
- [x] 2.2 Implement dependency existence, self-reference, cycle-path detection, and dependency-status compatibility checks.
- [x] 2.3 Validate lifecycle-specific evidence rules for exploring, proposed, active, complete, and deferred items.
- [x] 2.4 Add unit tests covering every semantic rule, multiple simultaneous errors, deterministic error ordering, direct cycles, and transitive cycles.

## 3. Repository Evidence Validation

- [x] 3.1 Implement safe single-component reference validation that rejects absolute paths, separators, traversal, and `.` or `..` names.
- [x] 3.2 Resolve capability, active-change, and archived-change evidence against the documented OpenSpec repository paths without shell or network access.
- [x] 3.3 Add temporary-repository tests for valid and missing capability specs, active changes, archived changes, unsafe references, and optional issue URL syntax.

## 4. Deterministic Markdown Generation

- [x] 4.1 Implement deterministic roadmap rendering with a generated-file warning, lifecycle legend, ordered summary table, and item outcome, exclusion, dependency, and evidence sections.
- [x] 4.2 Implement write mode restricted to `ROADMAP.md` and no-write check mode that compares canonical bytes and explains how to regenerate stale output.
- [x] 4.3 Add rendering snapshot tests, ordering tests, link-generation tests, final-newline verification, stale-output detection, and assertions that check mode performs no writes.

## 5. Initial Dynasty Roadmap

- [x] 5.1 Create `roadmap.yaml` with `SIM-01` complete and durable references to the possession simulation capabilities and archived change.
- [x] 5.2 Add the ten ordered exploratory roadmap outcomes for rosters, depth charts, weekly seasons, player state, recruiting, staff/tactics, offseason movement, program management, postseason, and history.
- [x] 5.3 Declare hard dependencies and meaningful exclusions for every initial item, then validate that the graph is acyclic and lifecycle-compatible.
- [x] 5.4 Generate and review the initial `ROADMAP.md`, confirming all eleven items and lifecycle definitions appear in canonical order.

## 6. CLI and Verification Integration

- [x] 6.1 Add CLI commands for roadmap validation, Markdown rendering, and generated-document checking with repository-root and path handling that works from the project CLI.
- [x] 6.2 Return concise success summaries and non-zero diagnostic failures without modifying files from validation or check commands.
- [x] 6.3 Add CLI parsing and end-to-end fixture tests for successful validation/render/check, invalid roadmap data, missing evidence, and stale generated output.
- [x] 6.4 Add roadmap validation and check commands to the repository's standard CI or verification workflow.

## 7. Documentation and Final Verification

- [x] 7.1 Document roadmap ownership, lifecycle transitions, evidence rules, editing/regeneration commands, and the OpenSpec discovery-to-completion workflow.
- [x] 7.2 Document that exploring items are directional candidates without dates or release commitments and that OpenSpec remains authoritative for requirements and delivery.
- [x] 7.3 Run roadmap validation/check, rustfmt, Clippy with warnings denied, all workspace tests, OpenSpec validation, and the existing canonical simulation calibration suite.
