//! Strict roadmap parsing, validation, and deterministic Markdown rendering.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path};

const SUPPORTED_SCHEMA_VERSION: u8 = 1;
const SOURCE_FILE: &str = "roadmap.yaml";
const GENERATED_FILE: &str = "ROADMAP.md";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Roadmap {
    schema_version: u8,
    items: Vec<RoadmapItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct RoadmapItem {
    id: String,
    sequence: u32,
    title: String,
    theme: String,
    status: Status,
    outcome: String,
    exclusions: Vec<String>,
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(default)]
    evidence: Evidence,
    #[serde(default)]
    deferral_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Exploring,
    Proposed,
    Active,
    Complete,
    Deferred,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Self::Exploring => "exploring",
            Self::Proposed => "proposed",
            Self::Active => "active",
            Self::Complete => "complete",
            Self::Deferred => "deferred",
        }
    }

    fn requires_ready_dependencies(self) -> bool {
        matches!(self, Self::Proposed | Self::Active | Self::Complete)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    active_changes: Vec<String>,
    #[serde(default)]
    archived_changes: Vec<String>,
    #[serde(default)]
    issues: Vec<String>,
}

pub fn validate_at(root: &Path) -> Result<String> {
    let roadmap = load(root)?;
    validate(&roadmap, root)?;
    Ok(format!("roadmap valid: {} items", roadmap.items.len()))
}

pub fn render_at(root: &Path) -> Result<String> {
    let roadmap = load(root)?;
    validate(&roadmap, root)?;
    let target = root.join(GENERATED_FILE);
    fs::write(&target, render(&roadmap)).with_context(|| format!("write {}", target.display()))?;
    Ok(format!(
        "rendered {} from {}",
        target.display(),
        SOURCE_FILE
    ))
}

pub fn check_at(root: &Path) -> Result<String> {
    let roadmap = load(root)?;
    validate(&roadmap, root)?;
    let expected = render(&roadmap);
    let target = root.join(GENERATED_FILE);
    let actual =
        fs::read_to_string(&target).with_context(|| format!("read {}", target.display()))?;
    if actual != expected {
        bail!(
            "{} is stale; regenerate it with `cargo run -p cli -- roadmap render`",
            target.display()
        );
    }
    Ok(format!("roadmap current: {} items", roadmap.items.len()))
}

fn load(root: &Path) -> Result<Roadmap> {
    let source = root.join(SOURCE_FILE);
    let yaml = fs::read_to_string(&source).with_context(|| format!("read {}", source.display()))?;
    parse(&yaml).with_context(|| format!("parse {}", source.display()))
}

fn parse(yaml: &str) -> Result<Roadmap> {
    let roadmap: Roadmap = serde_yaml::from_str(yaml).context("invalid roadmap YAML")?;
    if roadmap.schema_version != SUPPORTED_SCHEMA_VERSION {
        bail!(
            "unsupported schema_version {}; supported versions: [{}]",
            roadmap.schema_version,
            SUPPORTED_SCHEMA_VERSION
        );
    }
    Ok(roadmap)
}

fn validate(roadmap: &Roadmap, root: &Path) -> Result<()> {
    let mut errors = Vec::new();
    let mut ids: BTreeMap<&str, usize> = BTreeMap::new();
    let mut sequences: BTreeMap<u32, &str> = BTreeMap::new();

    for (index, item) in roadmap.items.iter().enumerate() {
        validate_item_shape(item, &mut errors);
        if let Some(previous) = ids.insert(&item.id, index) {
            errors.push(format!(
                "{}: duplicate id also used by item at index {}",
                item.id, previous
            ));
        }
        if let Some(previous) = sequences.insert(item.sequence, &item.id) {
            errors.push(format!(
                "{}: duplicate sequence {} also used by {}",
                item.id, item.sequence, previous
            ));
        }
    }

    for item in &roadmap.items {
        validate_dependencies(item, &ids, roadmap, &mut errors);
        validate_lifecycle(item, &mut errors);
        validate_evidence(item, root, &mut errors);
    }
    validate_cycles(roadmap, &ids, &mut errors);

    errors.sort();
    errors.dedup();
    if !errors.is_empty() {
        bail!("roadmap validation failed:\n- {}", errors.join("\n- "));
    }
    Ok(())
}

fn validate_item_shape(item: &RoadmapItem, errors: &mut Vec<String>) {
    if !valid_id(&item.id) {
        errors.push(format!(
            "{}: id must match <UPPERCASE_THEME>-<positive integer>",
            item.id
        ));
    }
    if item.sequence == 0 {
        errors.push(format!("{}: sequence must be positive", item.id));
    }
    for (name, value) in [
        ("title", item.title.as_str()),
        ("theme", item.theme.as_str()),
        ("outcome", item.outcome.as_str()),
    ] {
        if value.trim().is_empty() {
            errors.push(format!("{}: {name} cannot be empty", item.id));
        } else if value.contains(['\r', '\n']) {
            errors.push(format!("{}: {name} must be one line", item.id));
        }
    }
    if item.exclusions.is_empty() {
        errors.push(format!("{}: at least one exclusion is required", item.id));
    }
    for exclusion in &item.exclusions {
        if exclusion.trim().is_empty() || exclusion.contains(['\r', '\n']) {
            errors.push(format!(
                "{}: exclusions must be non-empty single lines",
                item.id
            ));
        }
    }
}

fn validate_dependencies(
    item: &RoadmapItem,
    ids: &BTreeMap<&str, usize>,
    roadmap: &Roadmap,
    errors: &mut Vec<String>,
) {
    let mut seen = BTreeSet::new();
    for dependency in &item.depends_on {
        if !seen.insert(dependency) {
            errors.push(format!("{}: duplicate dependency {dependency}", item.id));
        }
        if dependency == &item.id {
            errors.push(format!("{}: item cannot depend on itself", item.id));
            continue;
        }
        let Some(index) = ids.get(dependency.as_str()) else {
            errors.push(format!(
                "{}: dependency {dependency} does not exist",
                item.id
            ));
            continue;
        };
        let dependency_item = &roadmap.items[*index];
        if item.status.requires_ready_dependencies()
            && matches!(dependency_item.status, Status::Exploring | Status::Deferred)
        {
            errors.push(format!(
                "{}: {} item cannot depend on {} item {}",
                item.id,
                item.status.as_str(),
                dependency_item.status.as_str(),
                dependency
            ));
        }
    }
}

fn validate_lifecycle(item: &RoadmapItem, errors: &mut Vec<String>) {
    match item.status {
        Status::Proposed | Status::Active if item.evidence.active_changes.is_empty() => errors
            .push(format!(
                "{}: {} status requires an active change",
                item.id,
                item.status.as_str()
            )),
        Status::Complete
            if item.evidence.capabilities.is_empty()
                && item.evidence.archived_changes.is_empty() =>
        {
            errors.push(format!(
                "{}: complete status requires a capability or archived change",
                item.id
            ));
        }
        Status::Deferred
            if item
                .deferral_reason
                .as_deref()
                .is_none_or(|reason| reason.trim().is_empty()) =>
        {
            errors.push(format!("{}: deferred status requires a reason", item.id));
        }
        _ => {}
    }
}

fn validate_evidence(item: &RoadmapItem, root: &Path, errors: &mut Vec<String>) {
    for capability in &item.evidence.capabilities {
        validate_reference(item, "capability", capability, errors, || {
            root.join("openspec/specs").join(capability).join("spec.md")
        });
    }
    for change in &item.evidence.active_changes {
        validate_reference(item, "active change", change, errors, || {
            root.join("openspec/changes")
                .join(change)
                .join(".openspec.yaml")
        });
    }
    for change in &item.evidence.archived_changes {
        validate_reference(item, "archived change", change, errors, || {
            root.join("openspec/changes/archive")
                .join(change)
                .join(".openspec.yaml")
        });
    }
    for issue in &item.evidence.issues {
        if !(issue.starts_with("https://") || issue.starts_with("http://"))
            || issue.chars().any(char::is_whitespace)
        {
            errors.push(format!(
                "{}: issue reference is not an HTTP(S) URL: {issue}",
                item.id
            ));
        }
    }
}

fn validate_reference<F: FnOnce() -> std::path::PathBuf>(
    item: &RoadmapItem,
    kind: &str,
    reference: &str,
    errors: &mut Vec<String>,
    path: F,
) {
    if !safe_component(reference) {
        errors.push(format!("{}: unsafe {kind} reference: {reference}", item.id));
    } else {
        let expected = path();
        if !expected.is_file() {
            errors.push(format!(
                "{}: {kind} reference does not exist: {}",
                item.id,
                expected.display()
            ));
        }
    }
}

fn validate_cycles(roadmap: &Roadmap, ids: &BTreeMap<&str, usize>, errors: &mut Vec<String>) {
    fn visit(
        index: usize,
        roadmap: &Roadmap,
        ids: &BTreeMap<&str, usize>,
        state: &mut [u8],
        stack: &mut Vec<usize>,
        errors: &mut Vec<String>,
    ) {
        state[index] = 1;
        stack.push(index);
        let mut dependencies = roadmap.items[index].depends_on.iter().collect::<Vec<_>>();
        dependencies.sort();
        for dependency in dependencies {
            let Some(next) = ids.get(dependency.as_str()).copied() else {
                continue;
            };
            if state[next] == 0 {
                visit(next, roadmap, ids, state, stack, errors);
            } else if state[next] == 1 {
                let start = stack.iter().position(|entry| *entry == next).unwrap_or(0);
                let mut cycle = stack[start..]
                    .iter()
                    .map(|entry| roadmap.items[*entry].id.as_str())
                    .collect::<Vec<_>>();
                cycle.push(roadmap.items[next].id.as_str());
                errors.push(format!("dependency cycle: {}", cycle.join(" -> ")));
            }
        }
        stack.pop();
        state[index] = 2;
    }

    let mut state = vec![0_u8; roadmap.items.len()];
    let mut order = (0..roadmap.items.len()).collect::<Vec<_>>();
    order.sort_by_key(|index| &roadmap.items[*index].id);
    for index in order {
        if state[index] == 0 {
            visit(index, roadmap, ids, &mut state, &mut Vec::new(), errors);
        }
    }
}

fn render(roadmap: &Roadmap) -> String {
    let mut items = roadmap.items.iter().collect::<Vec<_>>();
    items.sort_by_key(|item| item.sequence);
    let mut output = String::new();
    output.push_str("<!-- GENERATED from roadmap.yaml. Do not edit ROADMAP.md directly. -->\n\n");
    output.push_str("# College Football Simulator Roadmap\n\n");
    output.push_str("This roadmap records product direction, not delivery dates or release commitments. `roadmap.yaml` is authoritative; OpenSpec remains authoritative for detailed requirements and implementation evidence.\n\n");
    output.push_str("## Lifecycle\n\n");
    output.push_str("`exploring` → `proposed` → `active` → `complete`. Any non-complete item can become `deferred`; deferred work returns through `exploring`.\n\n");
    output.push_str("| Order | ID | Feature | Theme | Status | Dependencies |\n");
    output.push_str("|---:|---|---|---|---|---|\n");
    for item in &items {
        let dependencies = if item.depends_on.is_empty() {
            "—".into()
        } else {
            item.depends_on.join(", ")
        };
        writeln!(
            output,
            "| {} | `{}` | {} | {} | `{}` | {} |",
            item.sequence,
            item.id,
            escape_table(&item.title),
            escape_table(&item.theme),
            item.status.as_str(),
            dependencies
        )
        .expect("writing to String cannot fail");
    }
    for item in items {
        writeln!(output, "\n## {} — {}\n", item.id, item.title)
            .expect("writing to String cannot fail");
        writeln!(output, "- **Status:** `{}`", item.status.as_str())
            .expect("writing to String cannot fail");
        writeln!(output, "- **Theme:** {}", item.theme).expect("writing to String cannot fail");
        writeln!(
            output,
            "- **Dependencies:** {}",
            if item.depends_on.is_empty() {
                "None".into()
            } else {
                item.depends_on.join(", ")
            }
        )
        .expect("writing to String cannot fail");
        writeln!(output, "- **Outcome:** {}", item.outcome).expect("writing to String cannot fail");
        output.push_str("- **Excludes:**\n");
        for exclusion in &item.exclusions {
            writeln!(output, "  - {exclusion}").expect("writing to String cannot fail");
        }
        if let Some(reason) = item.deferral_reason.as_deref() {
            writeln!(output, "- **Deferral reason:** {reason}")
                .expect("writing to String cannot fail");
        }
        render_evidence(&mut output, item);
    }
    output
}

fn render_evidence(output: &mut String, item: &RoadmapItem) {
    let mut evidence = Vec::new();
    evidence.extend(
        item.evidence
            .capabilities
            .iter()
            .map(|name| format!("[capability `{name}`](openspec/specs/{name}/spec.md)")),
    );
    evidence.extend(
        item.evidence
            .active_changes
            .iter()
            .map(|name| format!("[active change `{name}`](openspec/changes/{name}/)")),
    );
    evidence.extend(
        item.evidence
            .archived_changes
            .iter()
            .map(|name| format!("[archived change `{name}`](openspec/changes/archive/{name}/)")),
    );
    evidence.extend(
        item.evidence
            .issues
            .iter()
            .map(|url| format!("[issue]({url})")),
    );
    writeln!(
        output,
        "- **Evidence:** {}",
        if evidence.is_empty() {
            "None yet".into()
        } else {
            evidence.join(", ")
        }
    )
    .expect("writing to String cannot fail");
}

fn valid_id(id: &str) -> bool {
    let Some((theme, number)) = id.rsplit_once('-') else {
        return false;
    };
    !theme.is_empty()
        && theme.chars().all(|value| value.is_ascii_uppercase())
        && !number.is_empty()
        && number.chars().all(|value| value.is_ascii_digit())
        && number.parse::<u32>().is_ok_and(|value| value > 0)
}

fn safe_component(value: &str) -> bool {
    if value.is_empty() || matches!(value, "." | "..") {
        return false;
    }
    let mut components = Path::new(value).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

fn escape_table(value: &str) -> String {
    value.replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const VALID: &str = r#"
schema_version: 1
items:
  - id: SIM-01
    sequence: 1
    title: Simulation
    theme: simulation
    status: complete
    outcome: Games are simulated.
    exclusions: [Players]
    evidence:
      capabilities: [game-simulation]
  - id: TEAM-01
    sequence: 2
    title: Teams
    theme: dynasty
    status: exploring
    outcome: Teams have rosters.
    exclusions: [Recruiting]
    depends_on: [SIM-01]
"#;

    fn repository() -> TempDir {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir_all(directory.path().join("openspec/specs/game-simulation")).unwrap();
        fs::write(
            directory
                .path()
                .join("openspec/specs/game-simulation/spec.md"),
            "# spec\n",
        )
        .unwrap();
        fs::write(directory.path().join(SOURCE_FILE), VALID).unwrap();
        directory
    }

    #[test]
    fn strict_schema_parses_and_rejects_bad_documents() {
        assert_eq!(parse(VALID).unwrap().items.len(), 2);
        assert!(
            parse(&VALID.replace("schema_version: 1", "schema_version: 2"))
                .unwrap_err()
                .to_string()
                .contains("supported versions")
        );
        assert!(parse(&VALID.replace("items:", "unknown: true\nitems:")).is_err());
        assert!(parse(&VALID.replace("    title: Simulation\n", "")).is_err());
        assert!(parse("schema_version: one\nitems: []").is_err());
    }

    #[test]
    fn semantic_errors_are_aggregated_and_sorted() {
        let root = repository();
        let mut roadmap = parse(VALID).unwrap();
        roadmap.items[0].id = "bad".into();
        roadmap.items[0].sequence = 0;
        roadmap.items[0].title.clear();
        roadmap.items[0].exclusions.clear();
        roadmap.items[1].id = "bad".into();
        roadmap.items[1].sequence = 0;
        let error = validate(&roadmap, root.path()).unwrap_err().to_string();
        assert!(error.contains("duplicate id"));
        assert!(error.contains("duplicate sequence"));
        assert!(error.contains("id must match"));
        assert!(error.contains("title cannot be empty"));
        let lines = error.lines().skip(1).collect::<Vec<_>>();
        let mut sorted = lines.clone();
        sorted.sort();
        assert_eq!(lines, sorted);
    }

    #[test]
    fn dependency_errors_include_missing_status_and_cycles() {
        let root = repository();
        let mut roadmap = parse(VALID).unwrap();
        roadmap.items[0].status = Status::Active;
        roadmap.items[0].evidence.active_changes = vec!["missing-change".into()];
        roadmap.items[0].depends_on = vec!["TEAM-01".into(), "MISSING-01".into()];
        roadmap.items[1].depends_on = vec!["SIM-01".into()];
        let error = validate(&roadmap, root.path()).unwrap_err().to_string();
        assert!(error.contains("does not exist"));
        assert!(error.contains("active item cannot depend on exploring"));
        assert!(error.contains("dependency cycle"));
    }

    #[test]
    fn lifecycle_and_reference_rules_are_enforced() {
        let root = repository();
        let mut roadmap = parse(VALID).unwrap();
        roadmap.items[0].status = Status::Deferred;
        roadmap.items[0].evidence.capabilities = vec!["../escape".into(), "missing".into()];
        roadmap.items[0].evidence.issues = vec!["not a url".into()];
        let error = validate(&roadmap, root.path()).unwrap_err().to_string();
        assert!(error.contains("deferred status requires a reason"));
        assert!(error.contains("unsafe capability reference"));
        assert!(error.contains("reference does not exist"));
        assert!(error.contains("not an HTTP(S) URL"));
    }

    #[test]
    fn active_and_archived_evidence_resolve() {
        let root = repository();
        fs::create_dir_all(root.path().join("openspec/changes/work")).unwrap();
        fs::write(
            root.path().join("openspec/changes/work/.openspec.yaml"),
            "schema: spec-driven\n",
        )
        .unwrap();
        fs::create_dir_all(root.path().join("openspec/changes/archive/done")).unwrap();
        fs::write(
            root.path()
                .join("openspec/changes/archive/done/.openspec.yaml"),
            "schema: spec-driven\n",
        )
        .unwrap();
        let mut roadmap = parse(VALID).unwrap();
        roadmap.items[0].evidence.capabilities.clear();
        roadmap.items[0].evidence.archived_changes = vec!["done".into()];
        roadmap.items[1].status = Status::Proposed;
        roadmap.items[1].evidence.active_changes = vec!["work".into()];
        validate(&roadmap, root.path()).unwrap();
    }

    #[test]
    fn rendering_is_ordered_linked_and_has_final_newline() {
        let mut roadmap = parse(VALID).unwrap();
        roadmap.items.reverse();
        let markdown = render(&roadmap);
        assert!(markdown.starts_with("<!-- GENERATED"));
        assert!(markdown.ends_with('\n'));
        assert!(
            markdown.find("SIM-01 — Simulation").unwrap()
                < markdown.find("TEAM-01 — Teams").unwrap()
        );
        assert!(markdown.contains("openspec/specs/game-simulation/spec.md"));
    }

    #[test]
    fn render_and_check_modes_detect_drift_without_check_writes() {
        let root = repository();
        render_at(root.path()).unwrap();
        check_at(root.path()).unwrap();
        fs::write(root.path().join(GENERATED_FILE), "stale\n").unwrap();
        let before = fs::read(root.path().join(GENERATED_FILE)).unwrap();
        assert!(check_at(root.path()).is_err());
        let after = fs::read(root.path().join(GENERATED_FILE)).unwrap();
        assert_eq!(before, after);
    }
}
