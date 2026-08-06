## Context

The implemented simulation is now represented by main OpenSpec capability specs and an archived change. The longer dynasty feature sequence exists only in conversation, with no durable record of ordering, dependencies, status, or completion evidence. OpenSpec correctly captures current behavior and concrete changes, but active-change lists are intentionally not a speculative product backlog.

The roadmap needs to remain pleasant to read while also being reliable enough for automation. Maintaining the same status information independently in YAML and Markdown would create immediate drift risk, so the design distinguishes a structured source from a generated view.

## Goals / Non-Goals

**Goals:**

- Preserve the dynasty product direction in version control using stable identifiers and explicit outcomes.
- Validate dependencies, lifecycle requirements, and links to actual OpenSpec state.
- Generate a contributor-friendly roadmap without duplicating manual status updates.
- Make roadmap drift a deterministic test or CI failure.
- Seed the initial roadmap without prematurely committing exploratory features to detailed requirements.

**Non-Goals:**

- Implement any dynasty gameplay feature listed in the roadmap.
- Replace OpenSpec proposals, specs, designs, tasks, or archives.
- Replace GitHub issues as optional collaboration and discussion records.
- Estimate dates, effort, staffing, or release commitments.
- Automatically change roadmap status when filesystem state changes.
- Enforce historical state transitions by inspecting Git history in the first version.

## Decisions

### Use YAML as the sole editable roadmap source

The root `roadmap.yaml` will use a versioned schema similar to:

```yaml
schema_version: 1
items:
  - id: SIM-01
    sequence: 1
    title: Statistical game simulation
    theme: simulation
    status: complete
    outcome: Teams produce deterministic, calibrated possession-level results.
    exclusions:
      - Player-level simulation
    depends_on: []
    evidence:
      capabilities:
        - possession-game-simulation
        - simulation-calibration
      active_changes: []
      archived_changes:
        - 2026-08-05-add-possession-game-simulation
      issues: []
```

YAML is chosen over JSON for contributor editing while retaining typed deserialization and deterministic ordering. Markdown front matter was rejected because it mixes presentation and authoritative data. Maintaining both YAML and hand-edited Markdown was rejected because no validator can reliably infer which conflicting status is intended.

### Generate `ROADMAP.md` from the structured model

The renderer will sort items by `sequence` and produce fixed headings, a lifecycle legend, an ordered summary table, and per-item outcome, exclusions, dependencies, and evidence. It will use repository-relative Markdown links for evidence that exists locally. Rendering the same valid model must always produce identical UTF-8 bytes with a final newline.

`roadmap render` writes `ROADMAP.md`; `roadmap check` renders in memory and compares bytes without writing. A templating dependency is unnecessary for the stable initial format; explicit Rust formatting keeps behavior reviewable.

### Keep validation in the CLI boundary

Roadmap parsing touches repository files and is therefore not pure simulation-domain behavior. A dedicated CLI-side roadmap module will own schema structs, validation, repository reference checks, and Markdown rendering. `serde_yaml` will deserialize `roadmap.yaml`; existing `anyhow` will provide command-boundary context.

A standalone script was rejected because the repository already standardizes on Rust and Clippy/test verification. Placing roadmap types in `sim_core` was rejected because product planning metadata is not part of the football domain.

### Validate repository evidence by explicit paths

The validator will receive a repository root and resolve evidence without shelling out:

```text
capability          openspec/specs/<name>/spec.md
active change       openspec/changes/<name>/.openspec.yaml
archived change     openspec/changes/archive/<name>/.openspec.yaml
```

Names must be safe single path components: no separators, `.`/`..`, absolute paths, or traversal. Symlink canonicalization is not required because validation only checks expected in-repository file locations and does not read referenced content.

Issue references will be optional URL strings in v1 and will be syntax-validated without network access. Network validation was rejected because it makes local and CI results depend on authentication and external availability.

### Aggregate deterministic validation errors

Validation will run independent checks where possible and return a sorted list of structured errors rather than stopping at the first problem. Checks include:

1. Supported schema version and deserialization.
2. Required content, identifier form, unique identifiers, and unique positive sequence.
3. Dependency existence, self-reference, cycles, and lifecycle compatibility.
4. Status-specific evidence and safe evidence names.
5. Expected repository evidence files.

This provides useful contributor feedback in one run. YAML syntax errors necessarily stop semantic validation because no typed model exists.

### Use explicit lifecycle evidence without automatic status mutation

Statuses mean:

- `exploring`: Directional candidate; no delivery evidence required.
- `proposed`: An active OpenSpec change exists and planning may be underway.
- `active`: The referenced OpenSpec change is being implemented.
- `complete`: A main capability spec or archived change provides durable evidence.
- `deferred`: Work is intentionally set aside with a reason.

The normal lifecycle is `exploring → proposed → active → complete`. Any non-complete item can become deferred; deferred can return to exploring. The validator checks whether current status is supported, but it will not inspect Git history to prove how the item arrived there. Automatic status changes were rejected because file presence cannot encode product intent.

### Seed outcomes, not detailed future requirements

The initial items will describe completion outcomes and explicit exclusions at a level suitable for strategic sequencing. All remaining dynasty items begin as `exploring`; only the already delivered simulation begins `complete`. Detailed requirements will be created through future OpenSpec exploration and proposals.

Dependencies will represent hard sequencing constraints, not every conceptual relationship. This keeps the graph useful and avoids creating one long artificial chain.

### Add roadmap checks to standard verification

CLI tests will cover parsing, every validation class, deterministic ordering, cycles, evidence checks in temporary repository fixtures, rendering snapshots, no-write check behavior, and stale output detection. The repository's CI workflow or documented verification command will run both `roadmap validate` and `roadmap check` after tests.

## Risks / Trade-offs

- **Generated Markdown can be edited accidentally** → Put a generated-file warning at the top and fail `roadmap check` when it drifts.
- **Roadmap statuses may still become stale despite valid references** → Require review ownership and keep statuses explicit; validity proves evidence, not ongoing product intent.
- **Filesystem evidence rules couple validation to OpenSpec layout** → Centralize path mapping and cover it with tests so a future OpenSpec layout change has one update point.
- **Exploratory roadmap items can be mistaken for promises** → Define the lifecycle prominently and exclude dates or release commitments.
- **YAML permits surprising scalar coercions** → Deserialize into strict typed fields, deny unknown fields, and quote values in generated examples where ambiguity exists.
- **Adding `serde_yaml` increases dependencies for planning tooling** → Keep it confined to the CLI crate and pin it through the workspace lockfile.
- **Sequence numbers require edits when reprioritizing** → Treat them as ordering values rather than identity; identifiers remain stable.

## Migration Plan

1. Add strict roadmap schema types and semantic validation in the CLI crate.
2. Add repository evidence validation and deterministic Markdown rendering/checking.
3. Seed `roadmap.yaml` with the eleven known outcomes and generate `ROADMAP.md`.
4. Add CLI commands, tests, documentation, and automated verification hooks.
5. Validate the initial complete item against the existing simulation main specs and archive.

Rollback removes the roadmap files, CLI module/commands, YAML dependency, and verification hook. It does not affect simulation behavior or OpenSpec history.

## Open Questions

- Should GitHub issue references become mandatory once a roadmap item moves from exploring to proposed, or remain optional because the OpenSpec change already provides execution state?
- Should a later version add target releases or milestone groupings after the dynasty loop becomes playable?
