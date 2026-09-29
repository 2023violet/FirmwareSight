//! The contract test required by `04_TECH/26_PORTABLE_SCHEMA_POLICY.md` and prompt §30.
//!
//! There is no JSON-Schema validator in the dependency set, and §57 does not authorise adding one,
//! so this file implements the subset of draft/2020-12 that `schemas/diff.schema.json` actually
//! uses: `$ref`, `type`, `required`, `properties`, `additionalProperties`, `items`, `const`, `enum`,
//! `pattern`, `minimum`, `minLength` and `oneOf`. A construct the schema uses but the validator does
//! not implement is reported as an error rather than waved through, so the validator cannot silently
//! become weaker than the schema it checks.
//!
//! The rejecting cases are part of the test, not an afterthought: a contract test that can only ever
//! pass proves nothing (`05_ENGINEERING/02_TEST_STRATEGY.md`).

use std::path::{Path, PathBuf};

use firmwaresight_core::domain::diff::{
    BudgetState, DiffArtifact, DiffBudget, DiffError, DiffMemory, DiffResult, DiffSection,
    DiffSnapshotInput, DiffSymbol, SideEvidence, compare,
};
use firmwaresight_core::domain::identity::Fact;
use firmwaresight_report::diff::{DIFF_SCHEMA_ID, DIFF_SCHEMA_VERSION, DiffResultDto};
use firmwaresight_report::diff_render;
use serde_json::Value;

// --------------------------------------------------------------------------- schema subset

struct Schema<'a> {
    root: &'a Value,
}

#[derive(Debug)]
struct Failure {
    path: String,
    message: String,
}

impl<'a> Schema<'a> {
    fn from_root(root: &'a Value) -> Schema<'a> {
        Schema { root }
    }

    fn check(&self, schema: &Value, doc: &Value, path: &str, errors: &mut Vec<Failure>) {
        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            let Some(target) = self.resolve(reference) else {
                errors.push(Failure {
                    path: path.to_owned(),
                    message: format!("unresolvable $ref {reference}"),
                });
                return;
            };
            let mut inner = target.clone();
            // Siblings of $ref are meaningful in 2020-12; merge the ones this schema uses.
            if let Some(map) = schema.as_object()
                && let Some(obj) = inner.as_object_mut()
            {
                for (key, value) in map {
                    if key != "$ref" {
                        obj.insert(key.clone(), value.clone());
                    }
                }
            }
            self.check(&inner, doc, path, errors);
            return;
        }

        if let Some(want) = schema.get("const")
            && want != doc
        {
            errors.push(Failure {
                path: path.to_owned(),
                message: format!("must equal {want}"),
            });
        }

        if let Some(values) = schema.get("enum").and_then(Value::as_array)
            && !values.iter().any(|allowed| allowed == doc)
        {
            errors.push(Failure {
                path: path.to_owned(),
                message: format!("must be one of {}", shorten(values)),
            });
        }

        if let Some(types) = schema.get("type") {
            // A single type must hold; a list of types is a union, so one match is enough.
            let names = one_or_many(types);
            if !names.iter().any(|name| type_matches(name, doc)) {
                errors.push(Failure {
                    path: path.to_owned(),
                    message: format!(
                        "expected one of {}, found {}",
                        names.join("|"),
                        json_kind(doc)
                    ),
                });
            }
        }

        if let (Some(pattern), Some(text)) =
            (schema.get("pattern").and_then(Value::as_str), doc.as_str())
            && !self.pattern_holds(pattern, text)
        {
            errors.push(Failure {
                path: path.to_owned(),
                message: format!("{text:?} does not match pattern {pattern}"),
            });
        }

        if let Some(min) = schema.get("minimum").and_then(Value::as_u64)
            && let Some(number) = doc.as_u64()
            && number < min
        {
            errors.push(Failure {
                path: path.to_owned(),
                message: format!("{number} is below minimum {min}"),
            });
        }

        if let Some(min) = schema.get("minLength").and_then(Value::as_u64)
            && let Some(text) = doc.as_str()
            && (text.chars().count() as u64) < min
        {
            errors.push(Failure {
                path: path.to_owned(),
                message: format!("{text:?} is shorter than {min}"),
            });
        }

        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for key in required.iter().filter_map(Value::as_str) {
                if doc.get(key).is_none() {
                    errors.push(Failure {
                        path: path.to_owned(),
                        message: format!("missing required property {key}"),
                    });
                }
            }
        }

        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            if let Some(map) = doc.as_object() {
                for (key, value) in map {
                    match properties.get(key) {
                        Some(sub) => {
                            self.check(sub, value, &format!("{path}.{key}"), errors);
                        }
                        None => {
                            let closed =
                                schema.get("additionalProperties").and_then(Value::as_bool)
                                    == Some(false);
                            if closed {
                                errors.push(Failure {
                                    path: path.to_owned(),
                                    message: format!("property {key} is not declared"),
                                });
                            }
                        }
                    }
                }
            }
        } else if schema.get("additionalProperties").and_then(Value::as_bool) == Some(false)
            && doc.is_object()
        {
            errors.push(Failure {
                path: path.to_owned(),
                message: "additionalProperties:false with no properties is not implemented"
                    .to_owned(),
            });
        }

        if let Some(items) = schema.get("items")
            && let Some(array) = doc.as_array()
        {
            for (index, element) in array.iter().enumerate() {
                self.check(items, element, &format!("{path}[{index}]"), errors);
            }
        }

        if let Some(cases) = schema.get("oneOf").and_then(Value::as_array) {
            let matched = cases
                .iter()
                .filter(|case| {
                    let mut scratch = Vec::new();
                    self.check(case, doc, path, &mut scratch);
                    scratch.is_empty()
                })
                .count();
            if matched != 1 {
                errors.push(Failure {
                    path: path.to_owned(),
                    message: format!("oneOf matched {matched} of {} cases", cases.len()),
                });
            }
        }

        if let Some(cases) = schema.get("allOf").and_then(Value::as_array) {
            for case in cases {
                self.check(case, doc, path, errors);
            }
        }

        // `if` / `then` / `else` carries the absence invariants: an Added row must report the base
        // side as null, and a Removed row the target side. Without these, `null` and `0` would both
        // satisfy the type, and a fabricated zero would validate.
        if let Some(condition) = schema.get("if") {
            let mut scratch = Vec::new();
            self.check(condition, doc, path, &mut scratch);
            let branch = if scratch.is_empty() {
                schema.get("then")
            } else {
                schema.get("else")
            };
            if let Some(branch) = branch {
                self.check(branch, doc, path, errors);
            }
        }

        for keyword in ["anyOf", "not", "format", "patternProperties"] {
            if schema.get(keyword).is_some() {
                errors.push(Failure {
                    path: path.to_owned(),
                    message: format!("validator does not implement keyword {keyword}"),
                });
            }
        }
    }

    fn resolve(&self, reference: &str) -> Option<Value> {
        let rest = reference.strip_prefix("#/")?;
        let mut node = self.root;
        for segment in rest.split('/') {
            let key = segment.replace("~1", "/").replace("~0", "~");
            node = node.get(key)?;
        }
        Some(node.clone())
    }

    /// Only the two patterns the diff schema declares. Anything else fails the test loudly instead
    /// of passing silently, which is how a subset validator stays honest.
    fn pattern_holds(&self, pattern: &str, text: &str) -> bool {
        match pattern {
            "^[0-9a-f]{64}$" => {
                text.len() == 64
                    && text
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            }
            "^0x[0-9a-f]+$" => {
                let Some(digits) = text.strip_prefix("0x") else {
                    return false;
                };
                !digits.is_empty()
                    && digits
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            }
            other => {
                panic!("schema declares an unimplemented pattern: {other}")
            }
        }
    }
}

fn one_or_many(value: &Value) -> Vec<&str> {
    match value {
        Value::String(name) => vec![name.as_str()],
        Value::Array(names) => names.iter().filter_map(Value::as_str).collect(),
        _ => vec![],
    }
}

fn type_matches(name: &str, doc: &Value) -> bool {
    match name {
        "object" => doc.is_object(),
        "array" => doc.is_array(),
        "string" => doc.is_string(),
        "boolean" => doc.is_boolean(),
        "null" => doc.is_null(),
        "integer" => doc.is_i64() || doc.is_u64(),
        "number" => doc.is_f64() || doc.is_i64() || doc.is_u64(),
        other => panic!("schema declares an unimplemented type: {other}"),
    }
}

fn json_kind(doc: &Value) -> &'static str {
    match doc {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn shorten(values: &[Value]) -> String {
    values
        .iter()
        .map(|value| value.as_str().unwrap_or("?").to_owned())
        .collect::<Vec<_>>()
        .join(", ")
}

fn diff_schema() -> Value {
    let path = repo_root().join("schemas").join("diff.schema.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("diff.schema.json must parse: {error}"))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|parent| parent.parent())
        .expect("crates/firmwaresight-report lives at <root>/crates/<name>")
        .to_path_buf()
}

fn validate(doc: &Value) -> Vec<Failure> {
    let schema = diff_schema();
    let mut errors = Vec::new();
    Schema::from_root(&schema).check(&schema, doc, "$", &mut errors);
    errors
}

// --------------------------------------------------------------------------- fixtures

fn known(name: &str) -> Fact<String> {
    Fact::known(name.to_owned())
}

fn exact(bytes: u64) -> DiffBudget {
    DiffBudget {
        state: BudgetState::Exact,
        bytes: Some(bytes),
    }
}

fn partial(bytes: u64) -> DiffBudget {
    DiffBudget {
        state: BudgetState::Partial,
        bytes: Some(bytes),
    }
}

fn map() -> SideEvidence {
    SideEvidence {
        map_backed: true,
        layout_source: "map".to_owned(),
        weakest_basis: Some("MapRegionAndElfLoad".to_owned()),
    }
}

fn elf_only() -> SideEvidence {
    SideEvidence {
        map_backed: false,
        layout_source: "none".to_owned(),
        weakest_basis: Some("ElfAddressAndFlags".to_owned()),
    }
}

fn section(
    index: i64,
    name: Fact<String>,
    file_size: u64,
    memory_size: Fact<u64>,
    region: Fact<String>,
) -> DiffSection {
    DiffSection {
        index,
        name,
        role: "Code".to_owned(),
        alloc: true,
        write: false,
        execute: true,
        virtual_address: Fact::known(0x0800_0000 + (index as u64) * 0x100),
        load_address: Fact::known(0x0800_0000 + (index as u64) * 0x100),
        file_offset: Some(0x40 + (index as u64) * 0x100),
        file_size,
        memory_size,
        region,
    }
}

fn symbol(ordinal: i64, name: &str, size: Fact<u64>, address: Option<u64>) -> DiffSymbol {
    DiffSymbol {
        ordinal,
        name: known(name),
        address,
        size,
        kind: "Function".to_owned(),
        binding: "Global".to_owned(),
        section_ref: "Index(1)".to_owned(),
    }
}

fn artifact(tag: &str) -> DiffArtifact {
    DiffArtifact {
        file_name: format!("{tag}.elf"),
        sha256: format!("{:064x}", tag.len()),
        byte_size: 8192,
    }
}

fn input(
    id: &str,
    nonvolatile: DiffBudget,
    runtime: DiffBudget,
    evidence: SideEvidence,
    sections: Vec<DiffSection>,
    symbols: Vec<DiffSymbol>,
) -> DiffSnapshotInput {
    DiffSnapshotInput {
        snapshot_id: id.to_owned(),
        artifact: artifact(id),
        memory: Some(DiffMemory {
            nonvolatile,
            runtime_ram: runtime,
            excluded_metadata_bytes: 16,
            evidence,
        }),
        sections,
        symbols,
    }
}

/// A comparison that exercises every shape the schema declares: an addition, a removal, a changed
/// row, an unknown fact, a partial budget, and a name that repeats on one side.
fn rich_diff() -> DiffResult {
    let base = input(
        "snap-base",
        partial(900),
        exact(400),
        elf_only(),
        vec![
            section(0, known(".text"), 600, Fact::known(600), known("ROM")),
            section(1, known(".rodata"), 200, Fact::known(200), known("ROM")),
            section(
                2,
                known(".data"),
                64,
                Fact::unknown("sh_size absent for this row"),
                known("RAM"),
            ),
            section(3, known(".calib"), 128, Fact::known(128), known("ROM")),
        ],
        vec![
            symbol(0, "mqtt_task", Fact::known(64), Some(0x0800_0010)),
            symbol(1, "tls_handshake", Fact::known(256), Some(0x0800_0040)),
            symbol(2, "calib_apply", Fact::known(48), Some(0x0800_0080)),
        ],
    );
    let target = input(
        "snap-target",
        exact(1000),
        exact(460),
        map(),
        vec![
            section(0, known(".text"), 700, Fact::known(700), known("ROM")),
            section(1, known(".rodata"), 200, Fact::known(200), known("ROM")),
            section(
                2,
                known(".data"),
                64,
                Fact::known(64),
                Fact::unknown("no region recorded"),
            ),
            section(
                3,
                Fact::unknown("name table entry missing"),
                32,
                Fact::known(32),
                known("RAM"),
            ),
            section(4, known(".ota"), 512, Fact::known(512), known("RAM")),
        ],
        vec![
            symbol(0, "mqtt_task", Fact::known(128), Some(0x0800_0010)),
            symbol(1, "tls_handshake", Fact::known(256), Some(0x0800_0090)),
            symbol(
                2,
                "packet_router",
                Fact::unknown("STT_OBJECT entry carried sh_size 0"),
                Some(0x0800_00c0),
            ),
        ],
    );

    compare(&base, &target).expect("the fixture pair is comparable")
}

fn document(result: &DiffResult) -> Value {
    let dto = DiffResultDto::from_diff(result);
    let text = diff_render::render_json(&dto);
    serde_json::from_str(&text).expect("the renderer emits JSON")
}

// --------------------------------------------------------------------------- cases

#[test]
fn the_portable_diff_validates_against_the_published_schema() {
    let doc = document(&rich_diff());
    let errors = validate(&doc);
    assert!(
        errors.is_empty(),
        "diff.schema.json rejects its own output: {:?}",
        errors
            .iter()
            .map(|f| format!("{}: {}", f.path, f.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn schema_identity_is_the_stable_urn_and_major_one() {
    let doc = document(&rich_diff());
    assert_eq!(doc["schema"], DIFF_SCHEMA_ID);
    assert_eq!(doc["schemaVersion"], DIFF_SCHEMA_VERSION);
    assert_eq!(DIFF_SCHEMA_VERSION, 1);
    let schema = diff_schema();
    assert_eq!(schema["$id"], DIFF_SCHEMA_ID);
    assert_eq!(schema["properties"]["schemaVersion"]["const"], 1);
    assert_eq!(schema["additionalProperties"], false);
}

#[test]
fn an_added_row_is_absence_on_the_base_side_and_never_a_zero() {
    let doc = document(&rich_diff());
    let added = doc["sectionChanges"]
        .as_array()
        .expect("sections")
        .iter()
        .find(|row| row["key"] == ".ota")
        .expect("the target-only section is listed");

    assert_eq!(added["change"], "Added");
    assert_eq!(
        added["base"],
        Value::Null,
        "an addition is not a change from zero"
    );
    assert_eq!(added["fileSize"]["base"], Value::Null);
    assert_eq!(added["fileSize"]["delta"], Value::Null);
    assert_eq!(added["fileSize"]["target"], 512);
    assert!(
        added["fileSize"]["reason"]
            .as_str()
            .expect("a reason accompanies the missing delta")
            .contains("addition"),
    );
    assert!(added["target"].is_object());
}

#[test]
fn a_removed_row_is_absence_on_the_target_side_and_never_a_zero() {
    let doc = document(&rich_diff());
    let removed = doc["sectionChanges"]
        .as_array()
        .expect("sections")
        .iter()
        .find(|row| row["key"] == ".calib")
        .expect("the base-only section is listed");

    assert_eq!(removed["change"], "Removed");
    assert_eq!(removed["target"], Value::Null);
    assert_eq!(removed["fileSize"]["base"], 128);
    assert_eq!(removed["fileSize"]["target"], Value::Null);
    assert_eq!(removed["fileSize"]["delta"], Value::Null);
}

#[test]
fn deltas_are_signed_and_read_target_minus_base() {
    let doc = document(&rich_diff());
    let text = doc["sectionChanges"]
        .as_array()
        .expect("sections")
        .iter()
        .find(|row| row["key"] == ".text")
        .expect(".text is present on both sides");

    assert_eq!(text["change"], "Changed");
    assert_eq!(text["fileSize"]["base"], 600);
    assert_eq!(text["fileSize"]["target"], 700);
    assert_eq!(text["fileSize"]["delta"], 100);

    let shrunk = doc["symbolChanges"]
        .as_array()
        .expect("symbols")
        .iter()
        .find(|row| row["name"] == "calib_apply")
        .expect("the base-only symbol");
    assert_eq!(shrunk["change"], "Removed");

    let memory = &doc["memory"];
    assert_eq!(memory["nonvolatile"]["base"], 900);
    assert_eq!(memory["nonvolatile"]["target"], 1000);
    assert_eq!(memory["nonvolatile"]["delta"], 100);
    assert_eq!(
        memory["comparability"], "partial",
        "a floor against an exact total is not an exact comparison"
    );
    assert_eq!(memory["runtimeRam"]["delta"], 60);
    assert_eq!(memory["runtimeRam"]["comparability"], "exact");
}

#[test]
fn each_side_keeps_its_own_state_and_evidence() {
    let doc = document(&rich_diff());
    assert_eq!(doc["base"]["memory"]["nonvolatile"]["state"], "partial");
    assert_eq!(
        doc["base"]["memory"]["nonvolatile"]["classification"],
        "derived"
    );
    assert_eq!(doc["target"]["memory"]["nonvolatile"]["state"], "exact");
    assert_eq!(
        doc["target"]["memory"]["nonvolatile"]["classification"],
        "observed"
    );
    assert_eq!(doc["base"]["memory"]["evidence"]["mapBacked"], false);
    assert_eq!(doc["target"]["memory"]["evidence"]["mapBacked"], true);
    assert_eq!(doc["base"]["memory"]["footprintRowPresent"], true);
    assert_eq!(doc["base"]["memory"]["excludedMetadataBytes"], 16);
    assert_eq!(doc["base"]["artifact"]["fileName"], "snap-base.elf");
}

#[test]
fn a_side_with_no_footprint_row_stays_absent_rather_than_zero() {
    let mut base = input(
        "snap-base",
        exact(100),
        exact(20),
        elf_only(),
        vec![section(
            0,
            known(".text"),
            100,
            Fact::known(100),
            known("ROM"),
        )],
        vec![],
    );
    base.memory = None;
    let target = input(
        "snap-target",
        exact(100),
        exact(20),
        elf_only(),
        vec![section(
            0,
            known(".text"),
            100,
            Fact::known(100),
            known("ROM"),
        )],
        vec![],
    );

    let doc = document(&compare(&base, &target).expect("comparable"));
    assert_eq!(doc["base"]["memory"]["footprintRowPresent"], false);
    assert_eq!(doc["base"]["memory"]["nonvolatile"]["bytes"], Value::Null);
    assert_eq!(doc["base"]["memory"]["nonvolatile"]["state"], "unknown");
    assert_eq!(doc["base"]["memory"]["excludedMetadataBytes"], Value::Null);
    assert_eq!(doc["base"]["memory"]["evidence"]["layoutSource"], "absent");
    assert_eq!(doc["memory"]["nonvolatile"]["delta"], Value::Null);
    assert!(validate(&doc).is_empty());
}

#[test]
fn unknown_facts_survive_as_value_plus_reason_pairs() {
    let doc = document(&rich_diff());
    let data = doc["sectionChanges"]
        .as_array()
        .expect("sections")
        .iter()
        .find(|row| row["key"] == ".data")
        .expect("the row with an unknown base size");

    // Base size unknown, target size known: that is a field that cannot be decided, not a change.
    assert_eq!(data["memorySize"]["base"], Value::Null);
    assert_eq!(data["memorySize"]["target"], 64);
    let base_row = &data["base"];
    assert_eq!(base_row["memorySize"], Value::Null);
    assert_eq!(
        base_row["memorySizeUnknownReason"],
        "sh_size absent for this row"
    );
    let target_row = &data["target"];
    assert_eq!(target_row["region"], Value::Null);
    assert_eq!(target_row["regionUnknownReason"], "no region recorded");
    assert!(
        !data["indeterminateFields"]
            .as_array()
            .expect("indeterminate fields")
            .is_empty()
    );
    assert!(validate(&doc).is_empty());
}

#[test]
fn an_unnamed_row_and_a_repeated_name_are_reported_without_inventing_a_pair() {
    let doc = document(&rich_diff());
    let rows = doc["sectionChanges"].as_array().expect("sections");

    let unnamed = rows
        .iter()
        .find(|row| row["nameKnown"] == false)
        .expect("the target row with no name is listed");
    assert_eq!(unnamed["base"], Value::Null);
    assert_eq!(
        unnamed["ambiguous"], true,
        "a row with no name cannot be paired"
    );

    let ambiguous_count = rows.iter().filter(|row| row["ambiguous"] == true).count();
    assert!(ambiguous_count >= 1);
    assert!(
        doc["counts"]["sectionsAmbiguous"].as_u64().expect("count") >= 1,
        "the count must agree with the rows"
    );
    let codes = doc["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .map(|warning| warning["code"].as_str().expect("code"))
        .collect::<Vec<_>>();
    assert!(
        codes.contains(&"SECTION-AMBIGUOUS"),
        "ambiguous rows must be warned about: {codes:?}"
    );
}

#[test]
fn object_attribution_is_reported_as_unavailable_with_its_reason() {
    let doc = document(&rich_diff());
    assert_eq!(doc["objectChanges"]["available"], false);
    assert!(
        doc["objectChanges"]["reason"]
            .as_str()
            .expect("reason")
            .contains("object-file"),
        "the reason must name the missing evidence"
    );
}

#[test]
fn no_absolute_host_path_reaches_the_portable_document() {
    let text = diff_render::render_json(&DiffResultDto::from_diff(&rich_diff()));
    for marker in ["C:\\", "D:\\", "/home/", "/Users/", "/tmp/", "\\\\"] {
        assert!(
            !text.contains(marker),
            "host path fragment {marker} leaked into the export"
        );
    }
    let doc: Value = serde_json::from_str(&text).expect("json");
    let name = doc["base"]["artifact"]["fileName"].as_str().expect("name");
    assert!(
        !name.contains('/') && !name.contains('\\'),
        "{name} is not a bare file name"
    );
}

#[test]
fn rendering_the_same_diff_twice_produces_identical_bytes() {
    // Two runs of the same comparison must be byte-identical, including key order and array order.
    let first = diff_render::render_json(&DiffResultDto::from_diff(&rich_diff()));
    let second = diff_render::render_json(&DiffResultDto::from_diff(&rich_diff()));
    assert_eq!(first, second);
    assert!(first.ends_with('\n'));
    assert_eq!(
        first.matches('\n').count(),
        1,
        "exactly one compact document"
    );
}

#[test]
fn the_validator_rejects_documents_that_break_the_contract() {
    let good = document(&rich_diff());

    // Each mutation is one way a future change could silently widen or weaken the contract.
    let mut extra_key = good.clone();
    extra_key["unexpected"] = serde_json::json!(1);
    assert!(
        !validate(&extra_key).is_empty(),
        "additionalProperties:false must bite"
    );

    let mut dropped = good.clone();
    dropped.as_object_mut().expect("object").remove("base");
    assert!(!validate(&dropped).is_empty(), "base is required");

    let mut wrong_major = good.clone();
    wrong_major["schemaVersion"] = serde_json::json!(2);
    assert!(
        !validate(&wrong_major).is_empty(),
        "a v2 document must not validate against v1"
    );

    let mut zero_for_absence = good.clone();
    let rows = zero_for_absence["sectionChanges"]
        .as_array_mut()
        .expect("rows");
    let added = rows
        .iter_mut()
        .find(|row| row["key"] == ".ota")
        .expect("the added row");
    added["fileSize"]["base"] = serde_json::json!(0);
    assert!(
        !validate(&zero_for_absence).is_empty(),
        "a fabricated zero must not be accepted where absence is required"
    );

    let mut bad_hash = good.clone();
    bad_hash["base"]["artifact"]["sha256"] = serde_json::json!("not-a-hash");
    assert!(
        !validate(&bad_hash).is_empty(),
        "the sha256 pattern must hold"
    );

    let mut bad_address = good.clone();
    bad_address["base"]["memory"]["footprintRowPresent"] = serde_json::json!("true");
    assert!(!validate(&bad_address).is_empty(), "types must be checked");

    let mut unknown_code = good.clone();
    unknown_code["warnings"][0]["message"] = serde_json::json!(null);
    assert!(
        !validate(&unknown_code).is_empty(),
        "a warning message stays required"
    );
}

#[test]
fn a_clean_identical_comparison_still_validates_and_says_so() {
    let side = input(
        "snap-a",
        exact(100),
        exact(20),
        map(),
        vec![section(
            0,
            known(".text"),
            100,
            Fact::known(100),
            known("ROM"),
        )],
        vec![symbol(0, "main", Fact::known(10), Some(0x0800_0000))],
    );
    let other = input(
        "snap-b",
        exact(100),
        exact(20),
        map(),
        vec![section(
            0,
            known(".text"),
            100,
            Fact::known(100),
            known("ROM"),
        )],
        vec![symbol(0, "main", Fact::known(10), Some(0x0800_0000))],
    );
    let result = compare(&side, &other).expect("comparable");
    let doc = document(&result);

    assert!(doc["sectionChanges"].as_array().expect("rows").is_empty());
    assert!(doc["symbolChanges"].as_array().expect("rows").is_empty());
    assert_eq!(doc["unchanged"]["sections"], 1);
    assert_eq!(doc["unchanged"]["symbols"], 1);
    assert!(validate(&doc).is_empty());
    let html = diff_render::render_html(&DiffResultDto::from_diff(&result));
    assert!(html.contains("No section row differs"), "{html}");
}

#[test]
fn the_human_default_names_old_new_and_delta_and_carries_the_warnings() {
    let text = diff_render::render_human(&rich_diff());
    assert!(text.contains("Base    snap-base.elf"));
    assert!(text.contains("Target  snap-target.elf"));
    assert!(text.contains("delta +100"));
    assert!(text.contains("[partial]"));
    assert!(text.contains("Object / module attribution"));
    assert!(text.contains("MAP-EVIDENCE"));
    assert!(text.contains("SECTION-AMBIGUOUS"));
    assert!(
        !text.contains("snap-base\\"),
        "the human line must not print a path"
    );
}

#[test]
fn comparing_a_snapshot_with_itself_is_refused_before_any_document_exists() {
    let side = input("snap-a", exact(1), exact(1), map(), vec![], vec![]);
    let error = compare(&side, &side).expect_err("the same snapshot cannot be its own base");
    assert_eq!(error.code(), "ERR-DIFF-5001");
    assert!(matches!(error, DiffError::SameSnapshot { .. }));
}

#[test]
fn html_export_colours_come_from_the_frozen_token_set() {
    // AGENTS.md 11: numbers come from tokens, and a standalone export has no tokens.css to read.
    // So every hex colour the renderer emits must exist in assets/design-tokens.json, and a token
    // that disappears from the frozen set must not keep being used here.
    let tokens = std::fs::read_to_string(repo_root().join("assets").join("design-tokens.json"))
        .expect("the frozen token file is readable");
    let mut known: Vec<String> = tokens
        .split('"')
        .filter(|piece| {
            piece.starts_with('#')
                && matches!(piece.len(), 7 | 9)
                && piece[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
        })
        .map(|piece| piece.to_uppercase())
        .collect();
    known.dedup();

    let html = diff_render::render_html(&DiffResultDto::from_diff(&rich_diff()));
    let used: Vec<String> = html
        .split([' ', ';', ',', '\n', ':'])
        .filter(|piece| piece.starts_with('#') && matches!(piece.len(), 7 | 9))
        .map(|piece| piece.to_uppercase())
        .collect();
    assert!(
        !used.is_empty(),
        "the export is expected to carry token colours"
    );
    for colour in &used {
        assert!(
            known.contains(colour),
            "{colour} is not a value in assets/design-tokens.json"
        );
    }
}

#[test]
fn the_html_is_self_contained_deterministic_and_escape_checked() {
    let dto = DiffResultDto::from_diff(&rich_diff());
    let html = diff_render::render_html(&dto);
    assert_eq!(
        html,
        diff_render::render_html(&dto),
        "the export is byte-stable"
    );

    for forbidden in [
        "<script", "http://", "https://", "cdn", "@import", "<link", "<img", "fetch(",
    ] {
        assert!(
            !html.to_ascii_lowercase().contains(forbidden),
            "the export references {forbidden}"
        );
    }

    assert!(
        html.contains("FirmwareSight 0.1.0"),
        "the version must be visible"
    );
    assert!(html.contains("urn:firmwaresight:schema:diff:1"));
    assert!(html.contains("Added") && html.contains("Removed") && html.contains("Changed"));
    assert!(html.contains("Base (old)") && html.contains("Target (new)"));
    assert!(
        html.contains("MAP-EVIDENCE"),
        "the degradation stays visible"
    );
    assert!(
        html.contains("delta = target") || html.contains("Delta = target"),
        "the delta direction is stated in the export"
    );
}

#[test]
fn a_name_that_carries_markup_cannot_inject_into_the_export() {
    let hostile = "<script>alert(\"x\")</script>";
    let base = input(
        "snap-a",
        exact(10),
        exact(1),
        elf_only(),
        vec![section(
            0,
            known(hostile),
            10,
            Fact::known(10),
            known("ROM"),
        )],
        vec![],
    );
    let target = input(
        "snap-b",
        exact(10),
        exact(1),
        elf_only(),
        vec![section(
            0,
            known(hostile),
            20,
            Fact::unknown("reason \"with quotes\""),
            known("RAM"),
        )],
        vec![],
    );
    let result = compare(&base, &target).expect("comparable");
    let dto = DiffResultDto::from_diff(&result);
    let html = diff_render::render_html(&dto);

    assert!(
        !html.contains("<script>alert("),
        "an artifact-derived name must not become markup: {html}"
    );
    assert!(html.contains("&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;"));

    // JSON is not an HTML context: the name stays exactly as recorded, escaping and unescaped.
    let doc = document(&result);
    assert_eq!(doc["sectionChanges"][0]["key"], hostile);
    assert_eq!(
        doc["sectionChanges"][0]["target"]["memorySizeUnknownReason"],
        "reason \"with quotes\""
    );
    assert!(validate(&doc).is_empty());
    // The same name also travels inside a warning message, so that has to be escaped too.
    let warned = {
        let mut duplicated = rich_diff();
        duplicated
            .warnings
            .push(firmwaresight_core::domain::diff::DiffWarning {
                code: "TEST-ONLY".to_owned(),
                message: hostile.to_owned(),
            });
        diff_render::render_html(&DiffResultDto::from_diff(&duplicated))
    };
    assert!(!warned.contains("<script>alert("));
}
