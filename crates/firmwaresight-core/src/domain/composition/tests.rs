use super::*;
use crate::domain::identity::Fact;
use crate::domain::memory::{
    ByteTotal, LayoutSource, MemoryEvidenceBasis, MemoryRegionUsage, RegionAddressKind,
    RegionSectionUsage, RegionUsageRisk, RegionUsageState,
};
use crate::domain::symbol::{SymbolBinding, SymbolSectionRef};

fn contribution(
    index: usize,
    name: &str,
    flash: Fact<u64>,
    ram: Fact<u64>,
    span: (u64, u64),
) -> MemoryContribution {
    MemoryContribution {
        section_index: index,
        section_locator: format!("elf.section_header[{index}]"),
        section_name: Fact::known(name.to_owned()),
        nonvolatile_bytes: flash,
        runtime_ram_bytes: ram,
        basis: MemoryEvidenceBasis::MapRegionAndElfLoad,
        rule: "test",
        runtime_span: Some(span),
    }
}

fn region(
    name: &str,
    kind: RegionKind,
    capacity: u64,
    sections: &[&MemoryContribution],
) -> MemoryRegionUsage {
    MemoryRegionUsage {
        name: name.to_owned(),
        kind,
        origin: 0,
        capacity_bytes: Some(capacity),
        used_bytes: None,
        remaining_bytes: None,
        remaining_reason: "test",
        state: RegionUsageState::Exact,
        risk: RegionUsageRisk::WithinCapacity,
        evidence_source: "test",
        evidence_id: format!("map:{name}"),
        address_kind: RegionAddressKind::Virtual,
        sections: sections
            .iter()
            .map(|c| RegionSectionUsage {
                section_locator: c.section_locator.clone(),
                section_name: c.section_name.clone(),
                address_kind: RegionAddressKind::Virtual,
                address: 0,
                bytes: 0,
                virtual_address: Fact::unknown("test"),
                load_address: Fact::unknown("test"),
                evidence_basis: MemoryEvidenceBasis::MapRegionAndElfLoad,
            })
            .collect(),
        reasons: Vec::new(),
    }
}

fn footprint(
    contributions: Vec<MemoryContribution>,
    regions: Vec<MemoryRegionUsage>,
    flash: u64,
    ram: u64,
) -> MemoryFootprint {
    MemoryFootprint {
        nonvolatile: ByteTotal::Exact { bytes: flash },
        runtime_ram: ByteTotal::Exact { bytes: ram },
        contributions,
        excluded_metadata_bytes: 0,
        weakest_basis: Some(MemoryEvidenceBasis::MapRegionAndElfLoad),
        layout_source: LayoutSource::MapMemoryConfiguration,
        region_usage: regions,
    }
}

fn row(
    object: &str,
    archive: Option<&str>,
    output: &str,
    address: u64,
    size: u64,
) -> ObjectContribution {
    ObjectContribution {
        object: object.to_owned(),
        archive: archive.map(str::to_owned),
        member: object.to_owned(),
        input_section: ".text".to_owned(),
        output_section: Some(output.to_owned()),
        address,
        size,
        map_line: 7,
    }
}

fn symbol(name: &str, address: u64, size: u64, kind: SymbolKind) -> Symbol {
    Symbol {
        name: Fact::known(name.to_owned()),
        address: Fact::known(address),
        size: Fact::known(size),
        kind,
        binding: SymbolBinding::Global,
        section: SymbolSectionRef::Index(1),
    }
}

/// .text (100 B Flash) and .data (8 B Flash, 8 B RAM) in FLASH; .data and .bss (16 B) in RAM.
fn fixture() -> (MemoryFootprint, Vec<ObjectContribution>, Vec<Symbol>) {
    let text = contribution(
        1,
        ".text",
        Fact::known(100),
        Fact::known(0),
        (0x0800_0000, 100),
    );
    let data = contribution(2, ".data", Fact::known(8), Fact::known(8), (0x2000_0000, 8));
    let bss = contribution(
        3,
        ".bss",
        Fact::known(0),
        Fact::known(16),
        (0x2000_0008, 16),
    );
    let regions = vec![
        region("FLASH", RegionKind::Nonvolatile, 1024, &[&text, &data]),
        region("RAM", RegionKind::VolatileRam, 512, &[&data, &bss]),
    ];
    let fp = footprint(vec![text, data, bss], regions, 108, 24);
    let rows = vec![
        row("main.o", None, ".text", 0x0800_0000, 60),
        row("libm.a(x.o)", Some("libm.a"), ".text", 0x0800_003c, 30),
        row("main.o", None, ".data", 0x2000_0000, 8),
    ];
    let symbols = vec![
        symbol("main", 0x0800_0001, 40, SymbolKind::Function), // Thumb: bit 0 set
        symbol("helper", 0x0800_0028, 16, SymbolKind::Function),
        symbol("not_sized", 0x0800_0030, 0, SymbolKind::Function),
        symbol("g_state", 0x2000_0000, 8, SymbolKind::Object),
    ];
    (fp, rows, symbols)
}

fn bytes(node: &CompositionNode) -> Vec<(String, Option<u64>)> {
    node.children
        .iter()
        .map(|c| (c.name.clone(), c.bytes))
        .collect()
}

#[test]
fn the_flash_tree_nests_region_section_object_symbol_and_is_conserved() {
    let (fp, rows, symbols) = fixture();
    let tree = build(Side::Nonvolatile, &fp, &rows, &symbols);
    assert_eq!((tree.name.as_str(), tree.bytes), ("Flash", Some(108)));
    assert_eq!(bytes(&tree), vec![("FLASH".to_owned(), Some(108))]);
    let flash = &tree.children[0];
    assert_eq!(flash.capacity, Some(1024));
    assert_eq!(
        bytes(flash),
        vec![
            (".text".to_owned(), Some(100)),
            (".data".to_owned(), Some(8))
        ]
    );
    let text = &flash.children[0];
    assert_eq!(
        bytes(text),
        vec![
            ("main.o".to_owned(), Some(60)),
            ("libm.a(x.o)".to_owned(), Some(30)),
            ("linker bytes".to_owned(), Some(10))
        ]
    );
    let main = &text.children[0];
    assert_eq!(
        bytes(main),
        vec![
            ("main".to_owned(), Some(40)),
            ("helper".to_owned(), Some(16)),
            ("(no symbol)".to_owned(), Some(4))
        ],
        "the Thumb bit is ignored, unsized symbols are left out, and the gap is explicit"
    );
    assert_eq!(main.children[2].class, EvidenceClass::Derived);
    // every known parent equals the sum of its children
    fn conserved(node: &CompositionNode) {
        if let (Some(parent), false) = (node.bytes, node.children.is_empty()) {
            let sum: u64 = node
                .children
                .iter()
                .map(|c| c.bytes.expect("all known here"))
                .sum();
            assert_eq!(sum, parent, "{}", node.id);
        }
        node.children.iter().for_each(conserved);
    }
    conserved(&tree);
}

#[test]
fn the_ram_tree_places_data_and_bss_and_states_sections_without_rows() {
    let (fp, rows, symbols) = fixture();
    let tree = build(Side::Runtime, &fp, &rows, &symbols);
    assert_eq!(tree.bytes, Some(24));
    let ram = &tree.children[0];
    assert_eq!(
        bytes(ram),
        vec![(".bss".to_owned(), Some(16)), (".data".to_owned(), Some(8))]
    );
    assert_eq!(
        bytes(&ram.children[0]),
        vec![("linker bytes".to_owned(), Some(16))],
        "no object row names .bss"
    );
    assert_eq!(
        bytes(&ram.children[1].children[0]),
        vec![("g_state".to_owned(), Some(8))]
    );
}

#[test]
fn a_compressed_section_leaves_its_objects_unknown_never_zero() {
    let data = contribution(
        2,
        "RW_RAM",
        Fact::known(428),
        Fact::known(2572),
        (0x2000_0000, 2572),
    );
    let fp = footprint(
        vec![data.clone()],
        vec![region("ER_ROM", RegionKind::Nonvolatile, 4096, &[&data])],
        428,
        2572,
    );
    let rows = vec![row("arm_fir_data.o", None, "RW_RAM", 0x2000_0000, 2560)];
    let tree = build(Side::Nonvolatile, &fp, &rows, &[]);
    let section = &tree.children[0].children[0];
    assert_eq!(section.bytes, Some(428));
    let object = &section.children[0];
    assert_eq!(object.bytes, None);
    assert_eq!(object.class, EvidenceClass::Unknown);
    assert!(object.reason.is_some());
    assert_eq!(
        section.children.len(),
        1,
        "no gap is claimed when a child is Unknown"
    );
}

#[test]
fn without_a_region_table_sections_sit_under_the_root() {
    let (mut fp, rows, symbols) = fixture();
    fp.region_usage.clear();
    let tree = build(Side::Nonvolatile, &fp, &rows, &symbols);
    assert_eq!(
        bytes(&tree),
        vec![
            (".text".to_owned(), Some(100)),
            (".data".to_owned(), Some(8))
        ]
    );
}

#[test]
fn a_window_folds_the_tail_into_one_more_node_with_its_sum() {
    let (fp, rows, symbols) = fixture();
    let tree = build(Side::Nonvolatile, &fp, &rows, &symbols);
    let text = tree
        .find("root/r:FLASH/s:elf.section_header[1]")
        .expect("found by id");
    let window = text.window(1, 1);
    assert_eq!(window.children.len(), 2);
    assert_eq!(window.children[1].kind, NodeKind::More);
    assert_eq!(
        (window.children[1].bytes, window.children[1].folded),
        (Some(40), 2)
    );
    assert!(
        window.children[0].children.is_empty() && window.children[0].folded == 3,
        "depth is bounded"
    );
}
