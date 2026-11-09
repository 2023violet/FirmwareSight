//! ADR-0039 D2 on real builds: the composition tree is conserved (every known parent equals the sum of
//! its children), never shows Unknown as zero, and its Flash and RAM roots are the footprint's totals.
//!
//! The committed fixtures always run. Real third-party builds that are not committed (the Keil and IAR
//! samples of `evidence/D4-REAL` and `evidence/D4-IAR`) run when `FWSIGHT_COMPOSITION_SAMPLES` names a
//! folder of `<name>.{out,axf,elf}` + `<name>.map` pairs.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{AnalysisRequest, analyze};
use firmwaresight_core::domain::composition::{CompositionNode, NodeKind, Side, build};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Returns how many nodes the tree holds, after checking every invariant on the way.
fn check(node: &CompositionNode, path: &str) -> usize {
    let mut nodes = 1;
    if node.bytes.is_none() {
        assert!(
            node.reason.is_some() || node.kind == NodeKind::Root,
            "{path}: an Unknown node states why"
        );
    }
    if let Some(parent) = node.bytes
        && !node.children.is_empty()
    {
        let known: Option<u64> = node
            .children
            .iter()
            .try_fold(0_u64, |acc, c| c.bytes.map(|b| acc + b));
        if let Some(sum) = known {
            assert!(
                sum == parent || node.reason.is_some(),
                "{path}/{}: children {sum} vs parent {parent}",
                node.name
            );
        }
    }
    for child in &node.children {
        nodes += check(child, &format!("{path}/{}", child.name));
    }
    nodes
}

fn conserved(elf: &Path, map: &Path) -> (usize, usize) {
    let analysis = analyze(&AnalysisRequest::new(elf).with_map(map)).expect("analyzes");
    let snapshot = &analysis.snapshot;
    let memory = snapshot.memory().expect("a footprint");
    let mut counts = (0, 0);
    for (side, total, slot) in [
        (Side::Nonvolatile, memory.nonvolatile.bytes(), &mut counts.0),
        (Side::Runtime, memory.runtime_ram.bytes(), &mut counts.1),
    ] {
        let tree = build(side, memory, snapshot.objects(), snapshot.symbols());
        assert_eq!(
            tree.bytes,
            total,
            "{}: the root is the footprint total",
            elf.display()
        );
        *slot = check(&tree, &elf.display().to_string());
    }
    counts
}

#[test]
fn committed_gnu_and_armlink_fixtures_are_conserved() {
    let r = root();
    for (elf, map) in [
        (
            "fixtures/elf/p2-diff/target/firmware.elf",
            "fixtures/elf/p2-diff/target/firmware.map",
        ),
        (
            "fixtures/elf/d4-armlink/target/firmware.axf",
            "fixtures/elf/d4-armlink/target/firmware.map",
        ),
        (
            "fixtures/elf/m2-objects/target/firmware.elf",
            "fixtures/elf/m2-objects/target/firmware.map",
        ),
    ] {
        let (flash, ram) = conserved(&r.join(elf), &r.join(map));
        assert!(
            flash > 3 && ram > 1,
            "{elf}: a nested tree, not a flat list ({flash}, {ram} nodes)"
        );
    }
}

#[test]
fn real_samples_named_by_the_environment_are_conserved() {
    let Ok(dir) = std::env::var("FWSIGHT_COMPOSITION_SAMPLES") else {
        return;
    };
    let mut seen = 0;
    for entry in std::fs::read_dir(&dir).expect("the folder reads") {
        let path = entry.expect("an entry").path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !matches!(ext, "out" | "axf" | "elf") {
            continue;
        }
        let map = path.with_extension("map");
        if map.exists() {
            let (flash, ram) = conserved(&path, &map);
            println!("{}: {flash} Flash nodes, {ram} RAM nodes", path.display());
            seen += 1;
        }
    }
    assert!(seen > 0, "no sample pairs in {dir}");
}
