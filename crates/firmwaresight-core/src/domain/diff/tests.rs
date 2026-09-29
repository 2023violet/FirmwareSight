//! Diff semantics, each test named for the claim it protects.

use super::*;
use crate::domain::identity::Fact;

fn known(name: &str) -> Fact<String> {
    Fact::known(name.to_owned())
}

fn unknown(reason: &str) -> Fact<String> {
    Fact::unknown(reason)
}

fn size(bytes: u64) -> Fact<u64> {
    Fact::known(bytes)
}

fn no_size(reason: &str) -> Fact<u64> {
    Fact::unknown(reason)
}

fn addr(value: u64) -> Fact<u64> {
    Fact::known(value)
}

fn no_addr(reason: &str) -> Fact<u64> {
    Fact::unknown(reason)
}

fn section(index: i64, name: Fact<String>, file_size: u64, memory_size: Fact<u64>) -> DiffSection {
    DiffSection {
        index,
        name,
        role: "Code".to_owned(),
        alloc: true,
        write: false,
        execute: true,
        virtual_address: addr(0x0800_0000 + (index as u64) * 0x100),
        load_address: addr(0x0800_0000 + (index as u64) * 0x100),
        file_offset: Some(0x40 + (index as u64) * 0x100),
        file_size,
        memory_size,
        region: known("ROM"),
    }
}

fn symbol(
    ordinal: i64,
    name: Fact<String>,
    address: Option<u64>,
    size: Fact<u64>,
    section_ref: &str,
) -> DiffSymbol {
    DiffSymbol {
        ordinal,
        name,
        address,
        size,
        kind: "Function".to_owned(),
        binding: "Global".to_owned(),
        section_ref: section_ref.to_owned(),
    }
}

fn budget(exact: u64) -> DiffBudget {
    DiffBudget {
        state: BudgetState::Exact,
        bytes: Some(exact),
    }
}

fn partial(bytes: u64) -> DiffBudget {
    DiffBudget {
        state: BudgetState::Partial,
        bytes: Some(bytes),
    }
}

fn unknown_budget() -> DiffBudget {
    DiffBudget::unknown_total()
}

fn map_backed() -> SideEvidence {
    SideEvidence {
        map_backed: true,
        layout_source: "MapMemoryConfiguration".to_owned(),
        weakest_basis: Some("MapRegionAndElfLoad".to_owned()),
    }
}

fn elf_only() -> SideEvidence {
    SideEvidence {
        map_backed: false,
        layout_source: "None".to_owned(),
        weakest_basis: Some("ElfAddressAndFlags".to_owned()),
    }
}

fn snapshot(
    id: &str,
    nonvolatile: DiffBudget,
    runtime: DiffBudget,
    evidence: SideEvidence,
    sections: Vec<DiffSection>,
    symbols: Vec<DiffSymbol>,
) -> DiffSnapshotInput {
    DiffSnapshotInput {
        snapshot_id: id.to_owned(),
        artifact: DiffArtifact {
            file_name: format!("{}.elf", id.rsplit('-').next().unwrap_or(id)),
            sha256: format!("{:064x}", id.len()),
            byte_size: 4096,
        },
        memory: Some(DiffMemory {
            nonvolatile,
            runtime_ram: runtime,
            excluded_metadata_bytes: 0,
            evidence,
        }),
        sections,
        symbols,
    }
}

fn simple_side(id: &str) -> DiffSnapshotInput {
    snapshot(
        id,
        budget(1000),
        budget(400),
        map_backed(),
        vec![
            section(0, known(".text"), 60, size(60)),
            section(1, known(".rodata"), 32, size(32)),
            section(2, known(".data"), 16, size(16)),
            section(3, known(".bss"), 0, size(64)),
        ],
        vec![
            symbol(
                0,
                known("mqtt_task"),
                Some(0x0800_0010),
                size(40),
                "Index(1)",
            ),
            symbol(
                1,
                known("sensor_fifo"),
                Some(0x0800_0050),
                size(20),
                "Index(1)",
            ),
            symbol(
                2,
                known("g_threshold"),
                Some(0x2000_0000),
                size(4),
                "Index(3)",
            ),
        ],
    )
}

fn find_section(diff: &DiffResult, name: &str) -> SectionChange {
    diff.section_changes
        .iter()
        .find(|row| row.key == name)
        .unwrap_or_else(|| panic!("no section row named {name} in {:?}", diff.section_changes))
        .clone()
}

fn find_symbol(diff: &DiffResult, name: &str) -> SymbolChange {
    diff.symbol_changes
        .iter()
        .find(|row| row.name == name)
        .unwrap_or_else(|| panic!("no symbol row named {name} in {:?}", diff.symbol_changes))
        .clone()
}

#[test]
fn comparing_a_snapshot_with_itself_is_refused_rather_than_answered_with_empty_tables() {
    let base = simple_side("snap-a");
    let same = base.clone();

    let err = compare(&base, &same).expect_err("identical ids must be refused");
    assert_eq!(
        err,
        DiffError::SameSnapshot {
            snapshot_id: "snap-a".to_owned()
        }
    );
    assert_eq!(err.code(), "ERR-DIFF-5001");
}

#[test]
fn two_builds_with_the_same_stored_facts_produce_no_changes_and_zero_deltas() {
    let base = simple_side("snap-a");
    let target = simple_side("snap-b");

    let diff = compare(&base, &target).expect("comparable");
    assert!(diff.section_changes.is_empty());
    assert!(diff.symbol_changes.is_empty());
    assert_eq!(diff.counts, ChangeCounts::default());
    assert_eq!(diff.unchanged.sections, 4);
    assert_eq!(diff.unchanged.symbols, 3);
    assert_eq!(diff.memory.nonvolatile.delta, Some(0));
    assert_eq!(diff.memory.runtime_ram.delta, Some(0));
    assert_eq!(diff.memory.comparability, Comparability::Exact);
    assert!(
        diff.warnings
            .iter()
            .any(|warning| warning.code == "NO-CHANGES"),
        "the reader is told nothing changed, rather than shown an unexplained blank"
    );
}

#[test]
fn a_bigger_target_yields_a_positive_delta_because_delta_is_target_minus_base() {
    let base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    target.sections[0].file_size = 100;
    target.sections[0].memory_size = size(96);

    let diff = compare(&base, &target).expect("comparable");
    let text = find_section(&diff, ".text");
    assert_eq!(text.change, ChangeKind::Changed);
    assert_eq!(text.file_size.delta, Some(40));
    assert_eq!(text.memory_size.delta, Some(36));
    assert!(text.differing_fields.contains(&"file_size".to_owned()));
}

#[test]
fn a_smaller_target_yields_a_negative_delta() {
    let mut base = simple_side("snap-a");
    base.sections[0].file_size = 100;
    let target = simple_side("snap-b");

    let diff = compare(&base, &target).expect("comparable");
    assert_eq!(find_section(&diff, ".text").file_size.delta, Some(-40));
}

#[test]
fn reversing_the_comparison_inverts_added_and_removed_and_flips_every_sign() {
    let mut base = simple_side("snap-a");
    base.sections.remove(1); // .rodata exists only in the target
    base.sections[0].file_size = 92; // .text shrank in the target
    let target = simple_side("snap-b");

    let forward = compare(&base, &target).expect("forward");
    let reverse = compare(&target, &base).expect("reverse");

    let text_forward = find_section(&forward, ".text");
    let text_reverse = find_section(&reverse, ".text");
    assert_eq!(text_forward.file_size.delta, Some(-32));
    assert_eq!(text_reverse.file_size.delta, Some(32));

    let rodata_forward = find_section(&forward, ".rodata");
    let rodata_reverse = find_section(&reverse, ".rodata");
    assert_eq!(rodata_forward.change, ChangeKind::Added);
    assert_eq!(rodata_reverse.change, ChangeKind::Removed);
    assert_eq!(
        forward.counts.sections_added,
        reverse.counts.sections_removed
    );
    assert_eq!(
        forward.counts.sections_removed,
        reverse.counts.sections_added
    );
}

#[test]
fn an_added_section_is_not_reported_as_a_change_from_zero() {
    let base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    target
        .sections
        .push(section(9, known(".ota"), 64, size(64)));

    let diff = compare(&base, &target).expect("comparable");
    let ota = find_section(&diff, ".ota");
    assert_eq!(ota.change, ChangeKind::Added);
    assert_eq!(ota.base, None);
    assert_eq!(ota.file_size.base, None, "absence, not zero");
    assert_eq!(ota.file_size.target, Some(64));
    assert_eq!(
        ota.file_size.delta, None,
        "an addition carries no delta, because there was nothing to grow from"
    );
    assert!(ota.file_size.reason.is_some());
}

#[test]
fn a_removed_section_is_not_reported_as_a_change_to_zero() {
    let mut base = simple_side("snap-a");
    base.sections
        .push(section(9, known(".calib"), 128, size(128)));
    let target = simple_side("snap-b");

    let diff = compare(&base, &target).expect("comparable");
    let calib = find_section(&diff, ".calib");
    assert_eq!(calib.change, ChangeKind::Removed);
    assert_eq!(calib.target, None);
    assert_eq!(calib.file_size.base, Some(128));
    assert_eq!(calib.file_size.target, None);
    assert_eq!(calib.file_size.delta, None);
}

#[test]
fn a_symbol_with_no_size_on_one_side_yields_no_delta_rather_than_a_zero_one() {
    let base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    target.symbols[1].size = no_size("STT_OBJECT entry carried sh_size 0");

    let diff = compare(&base, &target).expect("comparable");
    let sensor = find_symbol(&diff, "sensor_fifo");
    assert_eq!(sensor.size.delta, None);
    assert_eq!(sensor.size.base, Some(20));
    assert_eq!(sensor.size.target, None);
    assert_eq!(sensor.size.comparability, Comparability::Unknown);
    assert!(
        sensor.indeterminate_fields.contains(&"size".to_owned()),
        "known against unknown is recorded as missing evidence, not as a size change"
    );
    assert!(
        !sensor.differing_fields.contains(&"size".to_owned()),
        "the diff cannot claim the size changed when one side never had a number"
    );
}

#[test]
fn exact_against_exact_memory_is_the_only_pair_that_earns_an_exact_delta() {
    let base = snapshot(
        "snap-a",
        budget(1000),
        budget(400),
        map_backed(),
        vec![],
        vec![],
    );
    let target = snapshot(
        "snap-b",
        budget(1200),
        budget(420),
        map_backed(),
        vec![],
        vec![],
    );

    let diff = compare(&base, &target).expect("comparable");
    assert_eq!(diff.memory.nonvolatile.delta, Some(200));
    assert_eq!(diff.memory.nonvolatile.comparability, Comparability::Exact);
    assert_eq!(diff.memory.comparability, Comparability::Exact);
    assert_eq!(diff.memory.evidence_warning, None);
}

#[test]
fn a_partially_accounted_side_caps_the_delta_to_partial_and_says_so() {
    let base = snapshot(
        "snap-a",
        budget(1000),
        budget(400),
        map_backed(),
        vec![],
        vec![],
    );
    let target = snapshot(
        "snap-b",
        partial(1200),
        budget(400),
        elf_only(),
        vec![],
        vec![],
    );

    let diff = compare(&base, &target).expect("comparable");
    assert_eq!(diff.memory.nonvolatile.delta, Some(200));
    assert_eq!(
        diff.memory.nonvolatile.comparability,
        Comparability::Partial
    );
    assert!(
        diff.memory
            .nonvolatile
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("floor")),
        "partial is never presented with the confidence of exact: {:?}",
        diff.memory.nonvolatile.reason
    );
    assert_eq!(diff.memory.comparability, Comparability::Partial);
}

#[test]
fn an_unknown_budget_produces_no_delta_at_all_rather_than_a_number_minus_zero() {
    let base = snapshot(
        "snap-a",
        budget(1000),
        budget(400),
        map_backed(),
        vec![],
        vec![],
    );
    let target = snapshot(
        "snap-b",
        unknown_budget(),
        budget(400),
        elf_only(),
        vec![],
        vec![],
    );

    let diff = compare(&base, &target).expect("comparable");
    assert_eq!(diff.memory.nonvolatile.base, Some(1000));
    assert_eq!(diff.memory.nonvolatile.target, None);
    assert_eq!(diff.memory.nonvolatile.delta, None);
    assert_eq!(
        diff.memory.nonvolatile.comparability,
        Comparability::Unknown
    );
    assert_eq!(diff.memory.comparability, Comparability::Unknown);
}

#[test]
fn a_side_with_no_stored_footprint_row_reads_as_unknown_not_as_an_empty_budget() {
    let mut base = simple_side("snap-a");
    base.memory = None;
    let target = simple_side("snap-b");

    let diff = compare(&base, &target).expect("comparable");
    assert_eq!(diff.memory.nonvolatile.base, None);
    assert_eq!(diff.memory.nonvolatile.delta, None);
    assert_eq!(diff.memory.base.evidence.layout_source, "absent");
    assert!(!diff.memory.base.footprint_row_present);
    assert!(diff.memory.target.footprint_row_present);
    assert_eq!(diff.memory.base.excluded_metadata_bytes, None);
    assert_eq!(diff.memory.target.excluded_metadata_bytes, Some(0));
}

#[test]
fn each_side_keeps_its_own_budget_state_so_a_floor_stays_visible() {
    // §14: a partial 1000 against an exact 1000 must not read as exact against exact, and the
    // reader has to be able to see which side is the floor.
    let base = snapshot(
        "snap-a",
        partial(1000),
        budget(400),
        elf_only(),
        vec![],
        vec![],
    );
    let target = snapshot(
        "snap-b",
        budget(1000),
        budget(400),
        map_backed(),
        vec![],
        vec![],
    );

    let diff = compare(&base, &target).expect("comparable");
    assert_eq!(diff.memory.base.nonvolatile.state, BudgetState::Partial);
    assert_eq!(diff.memory.target.nonvolatile.state, BudgetState::Exact);
    assert!(!diff.memory.base.evidence.map_backed);
    assert!(diff.memory.target.evidence.map_backed);
    assert_eq!(diff.memory.nonvolatile.delta, Some(0));
    assert_eq!(
        diff.memory.comparability,
        Comparability::Partial,
        "the weaker side caps the pair"
    );
}

#[test]
fn a_section_name_that_appears_once_on_each_side_is_matched_by_name() {
    let base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    // Same rows, different storage order: a row position must not read as a change.
    target.sections.reverse();

    let diff = compare(&base, &target).expect("comparable");
    assert!(
        diff.section_changes.is_empty(),
        "row position is not a field a diff may claim changed: {:?}",
        diff.section_changes
    );
    assert_eq!(diff.unchanged.sections, 4);
}

#[test]
fn a_duplicated_section_name_is_left_unpaired_and_counted_as_ambiguous() {
    let mut base = simple_side("snap-a");
    base.sections
        .push(section(80, known(".text"), 60, size(60)));
    let target = simple_side("snap-b");

    let diff = compare(&base, &target).expect("comparable");
    let text_rows: Vec<_> = diff
        .section_changes
        .iter()
        .filter(|row| row.key == ".text")
        .collect();
    assert_eq!(
        text_rows.len(),
        3,
        "one duplicated base row is unpaired, and the single target row is unpaired too"
    );
    assert!(text_rows.iter().all(|row| row.ambiguous));
    assert_eq!(diff.counts.sections_ambiguous, 3);
    assert!(
        diff.warnings
            .iter()
            .any(|warning| warning.code == "SECTION-AMBIGUOUS"),
        "ambiguity is reported, not hidden: {:?}",
        diff.warnings
    );
}

#[test]
fn a_section_that_only_differs_by_an_unknowable_field_is_not_claimed_as_changed() {
    let mut base = simple_side("snap-a");
    base.sections[0].load_address = no_addr("no PT_LOAD covers this section");
    let target = simple_side("snap-b");

    let diff = compare(&base, &target).expect("comparable");
    let text = find_section(&diff, ".text");
    assert!(!text.differing_fields.contains(&"load_address".to_owned()));
    assert!(
        text.indeterminate_fields
            .contains(&"load_address".to_owned())
    );
}

#[test]
fn a_section_whose_role_moved_is_changed_even_when_every_size_matches() {
    let base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    target.sections[1].role = "ReadOnlyData".to_owned();

    let diff = compare(&base, &target).expect("comparable");
    let rodata = find_section(&diff, ".rodata");
    assert_eq!(rodata.change, ChangeKind::Changed);
    assert!(rodata.differing_fields.contains(&"role".to_owned()));
    assert_eq!(rodata.file_size.delta, Some(0));
}

#[test]
fn a_section_name_known_on_only_one_side_is_added_or_removed() {
    let base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    // The target dropped .bss and gained a section the base never had.
    target
        .sections
        .retain(|s| s.name.value() != Some(&".bss".to_owned()));
    target
        .sections
        .push(section(40, known(".noinit"), 0, size(32)));

    let diff = compare(&base, &target).expect("comparable");
    assert_eq!(find_section(&diff, ".bss").change, ChangeKind::Removed);
    assert_eq!(find_section(&diff, ".noinit").change, ChangeKind::Added);
    assert_eq!(diff.counts.sections_removed, 1);
    assert_eq!(diff.counts.sections_added, 1);
    assert!(
        !find_section(&diff, ".bss").ambiguous && !find_section(&diff, ".noinit").ambiguous,
        "a name that occurs once is not a repeat, whichever side it occurs on"
    );
    assert_eq!(diff.counts.sections_ambiguous, 0);
    assert!(
        diff.warnings
            .iter()
            .all(|warning| warning.code != "SECTION-AMBIGUOUS"),
        "a clean removal must not raise an ambiguity warning: {:?}",
        diff.warnings
    );
}

#[test]
fn a_row_unique_to_one_side_is_clean_for_sections_and_symbols_alike() {
    // The shipped fixture pair hit exactly this: `.calib` and `calib_apply` exist only in the base
    // build. Calling them ambiguous would claim a repeated name that does not exist, and would
    // inflate the ambiguity count the reader is told to watch.
    let mut base = simple_side("snap-a");
    base.sections
        .push(section(90, known(".calib"), 32, size(32)));
    base.symbols.push(symbol(
        90,
        known("calib_apply"),
        Some(0x0800_1000),
        size(16),
        "Index(1)",
    ));
    let target = simple_side("snap-b");

    let diff = compare(&base, &target).expect("comparable");
    let section = find_section(&diff, ".calib");
    assert_eq!(section.change, ChangeKind::Removed);
    assert!(!section.ambiguous);
    let symbol = find_symbol(&diff, "calib_apply");
    assert_eq!(symbol.change, ChangeKind::Removed);
    assert!(!symbol.ambiguous);
    assert_eq!(diff.counts.sections_ambiguous, 0);
    assert_eq!(diff.counts.symbols_ambiguous, 0);
    assert!(
        diff.warnings.iter().all(
            |warning| warning.code != "SECTION-AMBIGUOUS" && warning.code != "SYMBOL-AMBIGUOUS"
        ),
        "no name repeats here: {:?}",
        diff.warnings
    );
}

#[test]
fn an_unnamed_section_cannot_be_matched_and_is_reported_as_unpaired() {
    let mut base = simple_side("snap-a");
    base.sections
        .push(section(70, unknown("SHT_NULL name offset 0"), 8, size(8)));
    let mut target = simple_side("snap-b");
    target
        .sections
        .push(section(71, unknown("SHT_NULL name offset 0"), 8, size(8)));

    let diff = compare(&base, &target).expect("comparable");
    let rows: Vec<_> = diff
        .section_changes
        .iter()
        .filter(|row| !row.name_known)
        .collect();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row.ambiguous));
    assert_eq!(
        rows.iter()
            .filter(|r| r.change == ChangeKind::Removed)
            .count(),
        1
    );
    assert_eq!(
        rows.iter()
            .filter(|r| r.change == ChangeKind::Added)
            .count(),
        1
    );
}

#[test]
fn a_symbol_matches_only_when_name_kind_and_binding_all_agree() {
    let base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    target.symbols[0].binding = "Weak".to_owned();

    let diff = compare(&base, &target).expect("comparable");
    let mqtt_rows: Vec<_> = diff
        .symbol_changes
        .iter()
        .filter(|row| row.name == "mqtt_task")
        .collect();
    assert_eq!(
        mqtt_rows.len(),
        2,
        "a different binding is a different thing, not a change"
    );
    assert!(mqtt_rows.iter().any(|row| row.change == ChangeKind::Added));
    assert!(
        mqtt_rows
            .iter()
            .any(|row| row.change == ChangeKind::Removed)
    );
}

#[test]
fn a_duplicated_symbol_key_is_never_paired_by_position() {
    let mut base = simple_side("snap-a");
    base.symbols.push(symbol(
        90,
        known("sensor_fifo"),
        Some(0x0800_0900),
        size(20),
        "Index(1)",
    ));
    let target = simple_side("snap-b");

    let diff = compare(&base, &target).expect("comparable");
    let rows: Vec<_> = diff
        .symbol_changes
        .iter()
        .filter(|row| row.name == "sensor_fifo")
        .collect();
    assert!(rows.iter().all(|row| row.ambiguous));
    assert_eq!(diff.counts.symbols_ambiguous, rows.len());
    assert!(rows.iter().all(|row| row.size.delta.is_none()));
}

#[test]
fn a_symbol_that_grew_reports_a_positive_size_delta_and_no_other_change() {
    let base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    target.symbols[0].size = size(56);

    let diff = compare(&base, &target).expect("comparable");
    let mqtt = find_symbol(&diff, "mqtt_task");
    assert_eq!(mqtt.size.delta, Some(16));
    assert_eq!(mqtt.differing_fields, vec!["size".to_owned()]);
}

#[test]
fn an_address_that_moved_is_a_change_because_movement_is_what_compare_exists_to_show() {
    let base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    target.symbols[1].address = Some(0x0800_0060);

    let diff = compare(&base, &target).expect("comparable");
    let sensor = find_symbol(&diff, "sensor_fifo");
    assert!(sensor.differing_fields.contains(&"address".to_owned()));
    assert_eq!(
        sensor.size.delta,
        Some(0),
        "movement alone does not change a size"
    );
    assert_eq!(
        sensor.base.as_ref().and_then(|s| s.address),
        Some(0x0800_0050)
    );
    assert_eq!(
        sensor.target.as_ref().and_then(|s| s.address),
        Some(0x0800_0060)
    );
}

#[test]
fn a_symbol_that_changed_section_is_reported_as_changed() {
    let base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    target.symbols[2].section_ref = "Index(4)".to_owned();

    let diff = compare(&base, &target).expect("comparable");
    assert!(
        find_symbol(&diff, "g_threshold")
            .differing_fields
            .contains(&"section".to_owned())
    );
}

#[test]
fn added_and_removed_symbols_keep_their_size_without_inventing_a_delta() {
    let base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    target.symbols.push(symbol(
        500,
        known("packet_router"),
        Some(0x0800_1000),
        size(80),
        "Index(1)",
    ));
    target
        .symbols
        .retain(|s| s.name.value() != Some(&"g_threshold".to_owned()));

    let diff = compare(&base, &target).expect("comparable");
    let added = find_symbol(&diff, "packet_router");
    assert_eq!(added.change, ChangeKind::Added);
    assert_eq!(added.size.base, None);
    assert_eq!(added.size.target, Some(80));
    assert_eq!(added.size.delta, None);

    let removed = find_symbol(&diff, "g_threshold");
    assert_eq!(removed.change, ChangeKind::Removed);
    assert_eq!(removed.size.base, Some(4));
    assert_eq!(removed.size.target, None);
    assert_eq!(removed.size.delta, None);
}

#[test]
fn the_same_inputs_twice_produce_byte_identical_ordering() {
    let mut base = simple_side("snap-a");
    base.sections[0].file_size = 90;
    base.symbols.push(symbol(
        600,
        known("aaa_task"),
        Some(0x0800_2000),
        size(8),
        "Index(1)",
    ));
    let mut target = simple_side("snap-b");
    target
        .sections
        .push(section(99, known(".ota"), 64, size(64)));

    let first = compare(&base, &target).expect("first");
    let second = compare(&base, &target).expect("second");
    assert_eq!(
        first, second,
        "a diff must be reproducible, not merely plausible"
    );

    let keys: Vec<_> = second
        .section_changes
        .iter()
        .map(|row| row.key.clone())
        .collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(
        keys, sorted,
        "section rows are ordered by key, not by hash order"
    );

    let names: Vec<_> = second
        .symbol_changes
        .iter()
        .map(|row| row.name.clone())
        .collect();
    let mut sorted_names = names.clone();
    sorted_names.sort();
    assert_eq!(names, sorted_names, "symbol rows are ordered by name");
}

#[test]
fn comparing_does_not_mutate_either_snapshot_input() {
    let base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    target.sections[0].file_size = 120;
    let base_before = base.clone();
    let target_before = target.clone();

    let _diff = compare(&base, &target).expect("comparable");

    assert_eq!(base, base_before);
    assert_eq!(target, target_before);
}

#[test]
fn object_attribution_is_reported_unavailable_with_the_reason_that_is_true() {
    let base = simple_side("snap-a");
    let target = simple_side("snap-b");

    let diff = compare(&base, &target).expect("comparable");
    assert!(!diff.attribution.available);
    assert!(
        diff.attribution
            .reason
            .contains("no object-file or module attribution"),
        "the reason names what is missing rather than hedging: {}",
        diff.attribution.reason
    );
}

#[test]
fn a_missing_map_never_blocks_the_comparison_but_always_shows_the_downgrade() {
    let base = snapshot(
        "snap-a",
        budget(1000),
        budget(400),
        elf_only(),
        vec![],
        vec![],
    );
    let target = snapshot(
        "snap-b",
        budget(1100),
        budget(400),
        map_backed(),
        vec![],
        vec![],
    );

    let diff = compare(&base, &target).expect("a comparison without a MAP is still a comparison");
    assert_eq!(diff.memory.nonvolatile.delta, Some(100));
    let warning = diff
        .memory
        .evidence_warning
        .as_ref()
        .expect("the weaker side is visible");
    assert!(warning.contains("the base build has no stored GNU ld MAP"));
    assert!(
        diff.warnings
            .iter()
            .any(|warning| warning.code == "MAP-EVIDENCE")
    );
    assert!(
        !warning.contains("cannot compare"),
        "weaker evidence is not the same as no comparison: {warning}"
    );
}

#[test]
fn both_sides_without_a_map_says_so_instead_of_naming_the_wrong_side() {
    let base = snapshot(
        "snap-a",
        budget(1000),
        budget(400),
        elf_only(),
        vec![],
        vec![],
    );
    let target = snapshot(
        "snap-b",
        budget(1000),
        budget(400),
        elf_only(),
        vec![],
        vec![],
    );

    let diff = compare(&base, &target).expect("comparable");
    assert!(
        diff.memory
            .evidence_warning
            .as_deref()
            .is_some_and(|warning| warning.contains("neither build has a stored GNU ld MAP"))
    );
}

#[test]
fn top_growth_contains_only_changed_rows_with_a_positive_known_delta() {
    let mut base = simple_side("snap-a");
    base.symbols.push(symbol(
        700,
        known("big_existing"),
        Some(0x0800_3000),
        size(100),
        "Index(1)",
    ));
    let mut target = simple_side("snap-b");
    target.symbols.push(symbol(
        700,
        known("big_existing"),
        Some(0x0800_3000),
        size(400),
        "Index(1)",
    ));
    target.symbols.push(symbol(
        800,
        known("brand_new"),
        Some(0x0800_4000),
        size(900),
        "Index(1)",
    ));
    target
        .sections
        .push(section(90, known(".huge"), 5000, size(5000)));

    let diff = compare(&base, &target).expect("comparable");
    let symbol_growth = diff.top_symbol_growth(10);
    assert_eq!(
        symbol_growth
            .iter()
            .map(|e| e.key.clone())
            .collect::<Vec<_>>(),
        vec!["big_existing".to_owned()],
        "a 900-byte addition is not growth of something that already existed"
    );

    let section_growth = diff.top_section_growth(10);
    assert!(
        section_growth
            .iter()
            .all(|e| e.change == ChangeKind::Changed),
        "growth ranks changed rows only: {section_growth:?}"
    );

    let additions = diff.largest_added_symbols(10);
    assert_eq!(additions[0].key, "brand_new");
    assert_eq!(additions[0].delta, None);
    assert_eq!(additions[0].bytes, Some(900));

    let added_sections = diff.largest_added_sections(10);
    assert_eq!(added_sections[0].key, ".huge");
}

#[test]
fn growth_lists_honour_the_limit_and_stay_ordered_when_the_inputs_are_reordered() {
    let mut base = simple_side("snap-a");
    let mut target = simple_side("snap-b");
    for (index, entry) in base.sections.iter_mut().enumerate() {
        entry.memory_size = size(10 * (index as u64 + 1));
    }
    for (index, entry) in target.sections.iter_mut().enumerate() {
        entry.memory_size = size(100 * (index as u64 + 1));
    }

    let forward = compare(&base, &target).expect("comparable");
    let mut shuffled_target = target.clone();
    shuffled_target.sections.reverse();
    let reordered = compare(&base, &shuffled_target).expect("reordered");

    let top = forward.top_section_growth(2);
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].key, ".bss", "the largest delta comes first");
    assert_eq!(top[0].delta, Some(360));
    assert_eq!(
        top,
        reordered.top_section_growth(2),
        "row order in storage must not change the ranking"
    );
}

#[test]
fn a_delta_too_large_for_a_signed_count_is_absent_rather_than_wrapped() {
    let mut base = simple_side("snap-a");
    base.sections[0].file_size = 0;
    let mut target = simple_side("snap-b");
    target.sections[0].file_size = u64::MAX;

    let diff = compare(&base, &target).expect("no panic on a hostile stored number");
    let text = find_section(&diff, ".text");
    assert_eq!(text.file_size.delta, None);
    assert_eq!(
        text.file_size.reason.as_deref(),
        Some("the difference does not fit a signed 64-bit count")
    );
}

#[test]
fn a_huge_symbol_table_is_diffed_without_pairwise_work_and_stays_correct() {
    // 50 000 rows per side, one real change, one repeated key. A quadratic matcher would turn this
    // single test into the slowest step of the gate; the generous bound below only catches an
    // accidental O(n^2) rewrite, and is not a benchmark or a performance claim.
    let side = |id: &str, mutate: fn(&mut DiffSymbol)| -> DiffSnapshotInput {
        let mut symbols: Vec<DiffSymbol> = (0..50_000_i64)
            .map(|ordinal| {
                symbol(
                    ordinal,
                    known(&format!("task_{ordinal}")),
                    Some(0x0800_0000 + ordinal as u64 * 4),
                    size(16),
                    "Index(1)",
                )
            })
            .collect();
        mutate(&mut symbols[12_345]);
        snapshot(
            id,
            budget(1_000_000),
            budget(200_000),
            map_backed(),
            vec![],
            symbols,
        )
    };

    let start = std::time::Instant::now();
    let diff = compare(
        &side("snap-a", |_symbols| {}),
        &side("snap-b", |symbol| symbol.size = size(24)),
    )
    .expect("comparable");
    let elapsed = start.elapsed();

    assert_eq!(diff.counts.symbols_changed, 1);
    assert_eq!(diff.unchanged.symbols, 49_999);
    assert_eq!(diff.symbol_changes.len(), 1);
    assert_eq!(find_symbol(&diff, "task_12345").size.delta, Some(8));
    assert!(
        elapsed.as_secs() < 30,
        "the matcher is expected to stay near linear; took {elapsed:?}"
    );
}

#[test]
fn a_repeated_symbol_name_across_both_sides_still_completes_and_marks_every_row_ambiguous() {
    let repeated = |id: &str| -> DiffSnapshotInput {
        let symbols: Vec<DiffSymbol> = (0..2_000_i64)
            .map(|ordinal| {
                symbol(
                    ordinal,
                    known("static_fn"),
                    Some(0x0800_0000),
                    size(8),
                    "Index(1)",
                )
            })
            .collect();
        snapshot(id, budget(100), budget(100), map_backed(), vec![], symbols)
    };

    let diff = compare(&repeated("snap-a"), &repeated("snap-b")).expect("comparable");
    assert_eq!(diff.counts.symbols_ambiguous, 4_000);
    assert_eq!(diff.counts.symbols_changed, 0);
    assert_eq!(diff.unchanged.symbols, 0);
    assert!(
        diff.warnings
            .iter()
            .any(|warning| warning.code == "SYMBOL-AMBIGUOUS")
    );
}
