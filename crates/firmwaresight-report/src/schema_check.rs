//! The dependency-free JSON-Schema subset FirmwareSight's portable contracts are tested with.
//!
//! There is no schema validator in the dependency set, and none is authorized: `04_TECH/11` admits no
//! third-party validation crate, and a validator pulled in to check four small documents would be a
//! larger unreviewed surface than the documents it checks. So this module implements exactly the
//! keywords the published schemas use — `$ref`, `type`, `required`, `properties`,
//! `additionalProperties`, `items`, `contains`, `minItems`, `const`, `enum`, `pattern`, `minimum`,
//! `minLength`, `format`, `oneOf`, `allOf`, `if`/`then`/`else` — and reports a construct it does not
//! implement as a *failure* rather than skipping it. A subset validator that quietly ignores a keyword
//! is worse than none, because the contract then reads as checked when it is not.
//!
//! Lives in the library rather than in one `tests/` file because three crates test against it
//! (`firmwaresight-report` for diff, gate-results and accepted-reviews; `firmwaresight-project` for
//! `firmwaresight.toml`) and integration tests cannot share source across crate boundaries. Prompt §55
//! asks for this reuse explicitly instead of three copies. It reads nothing: a caller hands in the
//! schema document it loaded, so this module stays free of the filesystem like the rest of the crate.

use serde_json::Value;

/// A schema document and the checks it declares.
#[derive(Debug)]
pub struct Schema<'a> {
    root: &'a Value,
}

/// One rejection, located in the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl<'a> Schema<'a> {
    #[must_use]
    pub fn from_root(root: &'a Value) -> Schema<'a> {
        Schema { root }
    }

    /// Check one document against one schema node, appending every rejection found.
    pub fn check(&self, schema: &Value, doc: &Value, path: &str, errors: &mut Vec<Failure>) {
        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            let Some(target) = self.resolve(reference) else {
                errors.push(Failure {
                    path: path.to_owned(),
                    message: format!("unresolvable $ref {reference}"),
                });
                return;
            };
            let mut inner = target.clone();
            // Siblings of $ref are meaningful in 2020-12; merge the ones these schemas use.
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

        if let Some(min) = schema.get("minItems").and_then(Value::as_u64)
            && let Some(array) = doc.as_array()
            && (array.len() as u64) < min
        {
            errors.push(Failure {
                path: path.to_owned(),
                message: format!("{} item(s) is fewer than {min}", array.len()),
            });
        }

        // `contains` is how `[artifacts] required` says "elf always ships": at least one element must
        // satisfy the subschema, and an empty or unrelated list is a rejection rather than a pass.
        if let Some(wanted) = schema.get("contains")
            && let Some(array) = doc.as_array()
        {
            let matched = array.iter().any(|element| {
                let mut scratch = Vec::new();
                self.check(wanted, element, path, &mut scratch);
                scratch.is_empty()
            });
            if !matched {
                errors.push(Failure {
                    path: path.to_owned(),
                    message: format!("no item satisfies contains: {wanted}"),
                });
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

        // `format` is an annotation in 2020-12, but these schemas declare it as a promise about stored
        // timestamps, so it is asserted. An unimplemented format fails loudly rather than passing.
        if let Some(format) = schema.get("format").and_then(Value::as_str)
            && !self.format_holds(format, doc)
        {
            errors.push(Failure {
                path: path.to_owned(),
                message: format!("does not satisfy format {format:?}"),
            });
        }

        for keyword in ["anyOf", "not", "patternProperties"] {
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

    /// Only the patterns these schemas declare. Anything else panics instead of passing silently, so a
    /// new pattern in a published schema forces a change here rather than an unchecked contract.
    fn pattern_holds(&self, pattern: &str, text: &str) -> bool {
        let hex = |text: &str| {
            text.bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        };
        match pattern {
            "^[0-9a-f]{64}$" => text.len() == 64 && hex(text),
            "^0x[0-9a-f]+$" => {
                let Some(digits) = text.strip_prefix("0x") else {
                    return false;
                };
                !digits.is_empty() && hex(digits)
            }
            // A Git object id is either full SHA-1 or SHA-256, never an abbreviated prefix.
            "^([0-9a-f]{40}|[0-9a-f]{64})$" => (text.len() == 40 || text.len() == 64) && hex(text),
            other => {
                panic!("schema declares an unimplemented pattern: {other}");
            }
        }
    }

    fn format_holds(&self, format: &str, doc: &Value) -> bool {
        match format {
            // The UTC shape `gate_runs.created_at` and `accepted_reviews.accepted_at` are written in,
            // and the only format any published schema declares.
            "date-time" => match doc.as_str() {
                Some(text) => {
                    let bytes = text.as_bytes();
                    bytes.len() == 20
                        && bytes.iter().enumerate().all(|(index, byte)| match index {
                            4 | 7 => *byte == b'-',
                            10 => *byte == b'T',
                            13 | 16 => *byte == b':',
                            19 => *byte == b'Z',
                            _ => byte.is_ascii_digit(),
                        })
                }
                // A non-string is this keyword's business, and `type` already reported it.
                None => true,
            },
            other => {
                panic!("schema declares an unimplemented format: {other}");
            }
        }
    }
}

/// Validate one document against one whole schema.
#[must_use]
pub fn validate(schema: &Value, doc: &Value) -> Vec<Failure> {
    let mut errors = Vec::new();
    Schema::from_root(schema).check(schema, doc, "$", &mut errors);
    errors
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
