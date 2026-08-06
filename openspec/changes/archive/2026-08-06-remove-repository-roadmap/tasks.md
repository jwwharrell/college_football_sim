## 1. Remove Roadmap Artifacts and CLI

- [x] 1.1 Delete `roadmap.yaml`, generated `ROADMAP.md`, and roadmap-only fixtures or snapshots.
- [x] 1.2 Remove the roadmap CLI command hierarchy, dispatch paths, and roadmap implementation module.
- [x] 1.3 Remove roadmap-only dependencies from workspace and crate manifests and refresh the lockfile.

## 2. Update Verification and Documentation

- [x] 2.1 Remove roadmap validation and generated-document checks from CI and repository verification scripts.
- [x] 2.2 Replace repository-roadmap instructions with a concise link to GitHub Project #1 and retain the OpenSpec delivery workflow guidance.
- [x] 2.3 Search current non-archived files for stale roadmap command, YAML, generated Markdown, or capability references and remove them.

## 3. Validate and Finalize

- [x] 3.1 Run Rust formatting, Clippy with warnings denied, and all workspace tests.
- [x] 3.2 Validate all current OpenSpec specs and changes.
- [x] 3.3 Confirm the GitHub Project migration remains complete and record all implementation tasks as finished.
