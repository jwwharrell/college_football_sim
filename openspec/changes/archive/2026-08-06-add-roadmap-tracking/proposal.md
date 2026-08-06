## Why

The dynasty feature sequence currently exists only in conversation, so priorities, dependencies, and completion evidence can drift or disappear. A version-controlled, machine-validated roadmap will preserve product direction while linking strategic milestones to OpenSpec's authoritative capability and change history.

## What Changes

- Add a structured `roadmap.yaml` as the authoritative source for roadmap item identifiers, sequence, status, dependencies, outcomes, scope exclusions, and implementation evidence.
- Add a generated `ROADMAP.md` that presents the same roadmap clearly for contributors without duplicating manually maintained status data.
- Define lifecycle states for `exploring`, `proposed`, `active`, `complete`, and `deferred`, including evidence requirements and permitted transitions.
- Add deterministic roadmap validation for schema correctness, unique stable identifiers, dependency existence and cycles, sequence consistency, lifecycle evidence, and references to active changes, archived changes, and main capability specs.
- Add a render/check workflow so CI can detect when `ROADMAP.md` does not match `roadmap.yaml`.
- Seed the roadmap with the completed statistical game simulation and the recommended dynasty-oriented feature sequence from player rosters through historical continuity.
- Document how roadmap discovery becomes an OpenSpec proposal, implementation, spec sync, archive, and finally a completed roadmap item.

## Capabilities

### New Capabilities

- `roadmap-management`: Structured, version-controlled dynasty roadmap management with validated lifecycle, dependency, evidence, and generated-document behavior.

### Modified Capabilities

None.

## Impact

- The repository gains `roadmap.yaml` and generated `ROADMAP.md` planning artifacts.
- The Rust CLI gains roadmap validation and rendering/check commands; this introduces YAML deserialization support at the CLI boundary.
- CI or the standard verification workflow gains a roadmap validation and generated-file consistency check.
- OpenSpec main specs, active changes, and archived changes remain authoritative for product behavior and delivery evidence; the roadmap links to them rather than duplicating their requirements.
- No simulation, persistence, or gameplay behavior changes.
