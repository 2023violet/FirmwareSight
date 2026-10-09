//! Deterministic fingerprints: which policy, which run.
//!
//! A run id must answer "would re-running this produce the same answer?", so it is a digest of the
//! Gate input, never a wall clock or a random uuid (prompt §28). Two properties make it usable:
//! the same facts always produce the same id, and any fact that changes the verdict produces a
//! different one. Both are checked against the canonical text `firmwaresight-core` defines, so the
//! fingerprint and the evaluation cannot drift apart.
//!
//! Hashing lives here rather than in Core because Core declares no external dependencies (ADR-0027).

use firmwaresight_core::domain::gate::{GateContext, GatePolicy, canonical_policy_text};
use sha2::{Digest, Sha256};

/// The prefix that marks an id as a Gate run id in storage, portable output and the UI.
pub const RUN_ID_PREFIX: &str = "gate-";

/// The lowercase hex SHA-256 of some bytes.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex_of(&digest.finalize())
}

/// The semantic policy fingerprint. Comments, blank lines and key order are already gone by the time a
/// `GatePolicy` exists, so two configs that say the same thing hash the same (`04_TECH/08:52-56`).
#[must_use]
pub fn policy_sha256(policy: &GatePolicy) -> String {
    sha256_hex(canonical_policy_text(policy).as_bytes())
}

/// The run id: `gate-<sha256 of the canonical Gate input>`.
#[must_use]
pub fn run_id(context: &GateContext) -> String {
    format!(
        "{RUN_ID_PREFIX}{}",
        sha256_hex(context.canonical_input().as_bytes())
    )
}

/// Stream a file's digest without holding the file in memory (`04_TECH/11`, `sha2` rule).
pub fn file_sha256(path: &std::path::Path) -> std::io::Result<String> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut digest = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(hex_of(&digest.finalize()))
}

fn hex_of(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push_str(&format!("{byte:02x}"));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::gate_context;
    use firmwaresight_core::domain::gate::{GateRuleId, UnknownDisposition};

    #[test]
    fn hex_is_lowercase_and_two_characters_per_byte() {
        assert_eq!(hex_of(&[0x00, 0xff, 0x0a]), "00ff0a");
    }

    #[test]
    fn a_known_digest_matches_the_reference_vector() {
        // SHA-256 of the empty input and of `abc`: the two vectors every implementation publishes.
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn the_same_policy_always_hashes_the_same_and_a_change_never_does() {
        let policy = GatePolicy::default();
        assert_eq!(policy_sha256(&policy), policy_sha256(&policy.clone()));
        let mut changed = policy.clone();
        changed.unknown_evidence_review_count = Some(1);
        assert_ne!(policy_sha256(&policy), policy_sha256(&changed));
        let mut disposition = policy;
        disposition.on_unknown.git_clean = UnknownDisposition::Block;
        assert_ne!(policy_sha256(&disposition), policy_sha256(&changed));
    }

    /// PA-1 / T-C1-01. These four constants were measured at the C1 handover head (`114a84e`), before
    /// `attachments` existed, and they are the point of the byte-freeze: a zero-attachment run must
    /// still hash to the id a reviewer already accepted. The digest and the id are the same value
    /// because the id is `gate-` plus this text's digest.
    #[test]
    fn the_zero_attachment_canonical_text_is_byte_frozen() {
        let context = gate_context();
        let text = context.canonical_input();
        assert!(
            text.starts_with("firmwaresight-gate-input/1\n"),
            "the label moved while no attachment was bound: {text}"
        );
        assert!(
            !text.contains("attachments["),
            "an empty attachment set wrote a block, which is how `/1` bytes would drift: {text}"
        );
        assert_eq!(
            text.len(),
            1044,
            "the canonical text changed length: {text}"
        );
        assert_eq!(
            sha256_hex(text.as_bytes()),
            "72d5c108e9f5b362e2f108b112389a04abfa10934207045926e1a02bd8c5da00"
        );
        assert_eq!(
            run_id(&context),
            "gate-72d5c108e9f5b362e2f108b112389a04abfa10934207045926e1a02bd8c5da00"
        );
    }

    #[test]
    fn a_run_id_is_prefixed_and_binds_the_whole_input() {
        let context = gate_context();
        let first = run_id(&context);
        assert!(first.starts_with(RUN_ID_PREFIX), "{first}");
        assert_eq!(first.len(), RUN_ID_PREFIX.len() + 64, "{first}");
        assert_eq!(first, run_id(&context.clone()));
        let mut other = context.clone();
        other.snapshot_id = "snap-2".to_owned();
        assert_ne!(first, run_id(&other));
        let mut policy_change = context.clone();
        policy_change.policy.flash_budget = Some(4_096);
        assert_ne!(first, run_id(&policy_change));
        let mut dirty = context;
        dirty.git.dirty = firmwaresight_core::domain::identity::Fact::known(true);
        assert_ne!(first, run_id(&dirty));
    }

    #[test]
    fn digesting_a_file_streams_it_instead_of_trusting_a_size_claim() {
        let path = std::env::temp_dir().join(format!("firmwaresight-hash-{}", std::process::id()));
        std::fs::write(&path, b"abc").expect("the temporary file was written");
        assert_eq!(
            file_sha256(&path).expect("the file is readable"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn hashing_a_missing_file_is_an_error_not_a_zero_digest() {
        let missing = std::path::Path::new("firmwaresight-hash-target-that-does-not-exist");
        assert!(file_sha256(missing).is_err());
    }

    #[test]
    fn a_finding_id_is_derivable_from_the_fingerprint_without_running_the_gate_twice() {
        let context = gate_context();
        let id = run_id(&context);
        let evaluation = context.evaluate(&id);
        let clean = evaluation
            .finding(GateRuleId::GitClean)
            .expect("every rule answers");
        assert_eq!(clean.id, format!("{id}#{}", GateRuleId::GitClean.as_str()));
        assert_eq!(evaluation.run_id, id);
    }
}
