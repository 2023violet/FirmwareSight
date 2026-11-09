//! Where a build's bytes are (ADR-0039): one tree per budget side, from region down to symbol.
//!
//! Every byte figure is one Core already decided: a section's side charge from the footprint
//! (ADR-0021), an object's from [`attribution::charge`] of its MAP rows (ADR-0033), a symbol's from its
//! ELF size. This module only nests them and states what the children leave over, so the UI can draw a
//! treemap without computing anything.
//!
//! The tree is conserved: when a parent and all its children are known, the gap is an explicit child
//! (linker bytes under a section, `(no symbol)` under an object), never silently dropped. Unknown is a
//! node with a reason and no bytes, never zero.

use super::attribution::{self, ObjectContribution};
use super::evidence::EvidenceClass;
use super::memory::{MemoryContribution, MemoryFootprint, RegionKind};
use super::symbol::{Symbol, SymbolKind};

/// The budget a tree is drawn for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// Flash: the load image.
    Nonvolatile,
    /// RAM at runtime.
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Root,
    Region,
    /// The sections no region of this side's kind lists.
    NoRegion,
    Section,
    Object,
    Symbol,
    /// Section bytes no object row accounts for: fill, stubs, sections without rows.
    LinkerBytes,
    /// Object bytes no symbol accounts for.
    NoSymbol,
    /// Children folded away by [`CompositionNode::window`].
    More,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositionNode {
    /// A path-like id, unique in the tree and stable for the same inputs.
    pub id: String,
    pub kind: NodeKind,
    pub name: String,
    /// `None` is Unknown, with `reason`.
    pub bytes: Option<u64>,
    pub class: EvidenceClass,
    pub reason: Option<String>,
    /// Where a reader re-checks the figure: an ELF section locator, a MAP line, a symbol address.
    pub locator: Option<String>,
    /// A region's capacity, for the Inspector. Not a free-space claim.
    pub capacity: Option<u64>,
    /// How many children were folded into a `More` node.
    pub folded: usize,
    pub children: Vec<CompositionNode>,
}

impl CompositionNode {
    fn new(
        id: String,
        kind: NodeKind,
        name: impl Into<String>,
        bytes: Option<u64>,
        class: EvidenceClass,
    ) -> Self {
        Self {
            id,
            kind,
            name: name.into(),
            bytes,
            class,
            reason: None,
            locator: None,
            capacity: None,
            folded: 0,
            children: Vec::new(),
        }
    }

    /// The node with this id, anywhere below (or at) this one.
    #[must_use]
    pub fn find(&self, id: &str) -> Option<&Self> {
        if self.id == id {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(id))
    }

    /// A bounded copy for a bridge or a renderer: `depth` levels below this node, at most
    /// `max_children` per node (largest first); the rest fold into one `More` node carrying their sum
    /// when every one of them is known (ADR-0039 D6).
    #[must_use]
    pub fn window(&self, depth: usize, max_children: usize) -> Self {
        let mut out = Self {
            children: Vec::new(),
            ..self.clone()
        };
        if depth == 0 {
            out.folded = self.children.len();
            return out;
        }
        let keep = max_children.max(1);
        for child in self.children.iter().take(keep) {
            out.children.push(child.window(depth - 1, max_children));
        }
        if self.children.len() > keep {
            let rest = &self.children[keep..];
            let sum = rest
                .iter()
                .try_fold(0_u64, |acc, c| c.bytes.and_then(|b| acc.checked_add(b)));
            let mut more = Self::new(
                format!("{}/more", self.id),
                NodeKind::More,
                format!("{} more", rest.len()),
                sum,
                EvidenceClass::Derived,
            );
            more.folded = rest.len();
            if sum.is_none() {
                more.class = EvidenceClass::Unknown;
                more.reason = Some("at least one folded entry is Unknown".to_owned());
            }
            out.children.push(more);
        }
        out
    }
}

/// The composition tree of one budget side.
#[must_use]
pub fn build(
    side: Side,
    footprint: &MemoryFootprint,
    objects: &[ObjectContribution],
    symbols: &[Symbol],
) -> CompositionNode {
    let total = match side {
        Side::Nonvolatile => &footprint.nonvolatile,
        Side::Runtime => &footprint.runtime_ram,
    };
    let root_name = match side {
        Side::Nonvolatile => "Flash",
        Side::Runtime => "RAM",
    };
    let mut root = CompositionNode::new(
        "root".to_owned(),
        NodeKind::Root,
        root_name,
        total.bytes(),
        total.classification(),
    );
    if let super::memory::ByteTotal::Partial { reason, .. }
    | super::memory::ByteTotal::Unknown { reason } = total
    {
        root.reason = Some(reason.clone());
    }

    let symbols = symbol_index(symbols);
    let charged: Vec<(&MemoryContribution, Option<u64>)> = footprint
        .contributions
        .iter()
        .map(|c| (c, side_bytes(c, side)))
        .filter(|(_, bytes)| *bytes != Some(0))
        .collect();

    let kind = match side {
        Side::Nonvolatile => RegionKind::Nonvolatile,
        Side::Runtime => RegionKind::VolatileRam,
    };
    let regions: Vec<_> = footprint
        .region_usage
        .iter()
        .filter(|r| r.kind == kind)
        .collect();
    let mut placed = vec![false; charged.len()];

    if regions.is_empty() {
        for (i, (contribution, bytes)) in charged.iter().enumerate() {
            placed[i] = true;
            root.children.push(section_node(
                "root",
                contribution,
                *bytes,
                side,
                footprint,
                objects,
                &symbols,
            ));
        }
    } else {
        for region in &regions {
            let id = format!("root/r:{}", region.name);
            let mut node = CompositionNode::new(
                id.clone(),
                NodeKind::Region,
                region.name.clone(),
                None,
                EvidenceClass::Observed,
            );
            node.capacity = region.capacity_bytes;
            node.locator = Some(region.evidence_id.clone());
            for (i, (contribution, bytes)) in charged.iter().enumerate() {
                if !placed[i]
                    && region
                        .sections
                        .iter()
                        .any(|s| s.section_locator == contribution.section_locator)
                {
                    placed[i] = true;
                    node.children.push(section_node(
                        &id,
                        contribution,
                        *bytes,
                        side,
                        footprint,
                        objects,
                        &symbols,
                    ));
                }
            }
            if node.children.is_empty() {
                continue;
            }
            close_sum(&mut node);
            root.children.push(node);
        }
        let rest: Vec<usize> = (0..charged.len()).filter(|&i| !placed[i]).collect();
        if !rest.is_empty() {
            let id = "root/r:(no region)".to_owned();
            let mut node = CompositionNode::new(
                id.clone(),
                NodeKind::NoRegion,
                "(no region)",
                None,
                EvidenceClass::Derived,
            );
            node.reason =
                Some("no region of this kind in the region table lists these sections".to_owned());
            for i in rest {
                let (contribution, bytes) = charged[i];
                node.children.push(section_node(
                    &id,
                    contribution,
                    bytes,
                    side,
                    footprint,
                    objects,
                    &symbols,
                ));
            }
            close_sum(&mut node);
            root.children.push(node);
        }
    }
    sort_children(&mut root);
    root
}

fn side_bytes(c: &MemoryContribution, side: Side) -> Option<u64> {
    match side {
        Side::Nonvolatile => c.nonvolatile_bytes.value().copied(),
        Side::Runtime => c.runtime_ram_bytes.value().copied(),
    }
}

fn charge_side(row: &ObjectContribution, footprint: &MemoryFootprint, side: Side) -> Option<u64> {
    let cost = attribution::charge(row, footprint);
    match side {
        Side::Nonvolatile => cost.nonvolatile,
        Side::Runtime => cost.runtime_ram,
    }
}

/// Whether a MAP row belongs to this output section: same name, and when two sections share the name
/// (armlink, ADR-0036) the one whose runtime span holds the row.
fn row_in_section(
    row: &ObjectContribution,
    c: &MemoryContribution,
    footprint: &MemoryFootprint,
) -> bool {
    let Some(name) = c.section_name.value() else {
        return false;
    };
    if row.output_section.as_deref() != Some(name.as_str()) {
        return false;
    }
    let same_name = footprint
        .contributions
        .iter()
        .filter(|o| o.section_name.value() == Some(name))
        .count();
    if same_name <= 1 {
        return true;
    }
    c.runtime_span
        .is_some_and(|(start, size)| row.address >= start && row.address - start < size)
}

fn section_node(
    parent: &str,
    c: &MemoryContribution,
    bytes: Option<u64>,
    side: Side,
    footprint: &MemoryFootprint,
    objects: &[ObjectContribution],
    symbols: &[IndexedSymbol],
) -> CompositionNode {
    let name = c
        .section_name
        .value()
        .cloned()
        .unwrap_or_else(|| c.section_locator.clone());
    let id = format!("{parent}/s:{}", c.section_locator);
    let class = if bytes.is_some() {
        EvidenceClass::Observed
    } else {
        EvidenceClass::Unknown
    };
    let mut node = CompositionNode::new(id.clone(), NodeKind::Section, name, bytes, class);
    node.locator = Some(c.section_locator.clone());
    if bytes.is_none() {
        let fact = match side {
            Side::Nonvolatile => &c.nonvolatile_bytes,
            Side::Runtime => &c.runtime_ram_bytes,
        };
        node.reason = fact.reason_if_unknown().map(str::to_owned);
    }

    // objects, in first-row order, then summed per key
    let mut order: Vec<String> = Vec::new();
    let mut groups: Vec<Vec<&ObjectContribution>> = Vec::new();
    for row in objects
        .iter()
        .filter(|r| r.size > 0 && row_in_section(r, c, footprint))
    {
        match order.iter().position(|k| *k == row.object) {
            Some(i) => groups[i].push(row),
            None => {
                order.push(row.object.clone());
                groups.push(vec![row]);
            }
        }
    }
    for (key, rows) in order.iter().zip(groups) {
        node.children
            .push(object_node(&id, key, &rows, side, footprint, symbols));
    }
    if !node.children.is_empty() || bytes.is_some() {
        add_remainder(
            &mut node,
            NodeKind::LinkerBytes,
            "linker bytes",
            "fill, linker stubs and bytes no object row names",
        );
    }
    sort_children(&mut node);
    node
}

fn object_node(
    parent: &str,
    key: &str,
    rows: &[&ObjectContribution],
    side: Side,
    footprint: &MemoryFootprint,
    symbols: &[IndexedSymbol],
) -> CompositionNode {
    let id = format!("{parent}/o:{key}");
    let mut bytes = Some(0_u64);
    let mut ordinary = true;
    for row in rows {
        let cost = charge_side(row, footprint, side);
        bytes = match (bytes, cost) {
            (Some(a), Some(b)) => a.checked_add(b),
            _ => None,
        };
        ordinary &= cost == Some(row.size);
    }
    let class = if bytes.is_some() {
        EvidenceClass::Observed
    } else {
        EvidenceClass::Unknown
    };
    let mut node = CompositionNode::new(id.clone(), NodeKind::Object, key, bytes, class);
    node.locator = Some(format!("map line {}", rows[0].map_line));
    if bytes == Some(0) {
        return node;
    }
    if bytes.is_none() {
        node.reason = Some(
            "at least one MAP row of this object has an Unknown charge on this budget".to_owned(),
        );
        return node;
    }
    if ordinary {
        let mut seen: Vec<(u64, u64)> = Vec::new();
        for row in rows {
            let end = row.address.saturating_add(row.size);
            let start = symbols.partition_point(|s| s.address < row.address);
            for s in symbols[start..].iter().take_while(|s| s.address < end) {
                if s.address.saturating_add(s.size) > end || seen.contains(&(s.address, s.size)) {
                    continue;
                }
                seen.push((s.address, s.size));
                let mut sym = CompositionNode::new(
                    format!("{id}/y:{}@{:#x}", s.name, s.address),
                    NodeKind::Symbol,
                    s.name.clone(),
                    Some(s.size),
                    EvidenceClass::Derived,
                );
                sym.locator = Some(format!("{:#x}", s.address));
                node.children.push(sym);
            }
        }
        if !node.children.is_empty() {
            add_remainder(
                &mut node,
                NodeKind::NoSymbol,
                "(no symbol)",
                "bytes of this object no sized symbol covers",
            );
        }
    }
    sort_children(&mut node);
    node
}

/// Add the explicit gap child when parent and children are all known; say so when they cannot agree.
fn add_remainder(node: &mut CompositionNode, kind: NodeKind, name: &str, why: &str) {
    let Some(parent) = node.bytes else { return };
    let Some(sum) = node
        .children
        .iter()
        .try_fold(0_u64, |acc, c| c.bytes.and_then(|b| acc.checked_add(b)))
    else {
        return;
    };
    match parent.checked_sub(sum) {
        Some(0) => {}
        Some(gap) => {
            let mut gap_node = CompositionNode::new(
                format!(
                    "{}/{}",
                    node.id,
                    if kind == NodeKind::NoSymbol {
                        "nosym"
                    } else {
                        "linker"
                    }
                ),
                kind,
                name,
                Some(gap),
                EvidenceClass::Derived,
            );
            gap_node.reason = Some(why.to_owned());
            node.children.push(gap_node);
        }
        None => {
            node.reason = Some(format!(
                "the parts add up to {sum} bytes, more than the {parent} bytes charged here; the charged figure is kept"
            ));
        }
    }
}

/// A region's or the no-region group's bytes: the sum of its sections when all are known.
fn close_sum(node: &mut CompositionNode) {
    let sum = node
        .children
        .iter()
        .try_fold(0_u64, |acc, c| c.bytes.and_then(|b| acc.checked_add(b)));
    node.bytes = sum;
    if sum.is_none() {
        node.class = EvidenceClass::Unknown;
        node.reason
            .get_or_insert_with(|| "at least one section here has an Unknown charge".to_owned());
    }
    sort_children(node);
}

/// Largest first; Unknown last; then by name, so the order is stable.
fn sort_children(node: &mut CompositionNode) {
    node.children.sort_by(|a, b| match (a.bytes, b.bytes) {
        (Some(x), Some(y)) => y.cmp(&x).then_with(|| a.name.cmp(&b.name)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.name.cmp(&b.name),
    });
}

struct IndexedSymbol {
    name: String,
    address: u64,
    size: u64,
}

/// Sized code and data symbols, sorted by address; a global name wins over a local alias.
fn symbol_index(symbols: &[Symbol]) -> Vec<IndexedSymbol> {
    let mut out: Vec<(IndexedSymbol, u8)> = symbols
        .iter()
        .filter(|s| {
            matches!(
                s.kind,
                SymbolKind::Function
                    | SymbolKind::Object
                    | SymbolKind::Common
                    | SymbolKind::NotType
            )
        })
        .filter_map(|s| {
            let name = s.name.value()?.clone();
            let address = *s.address.value()?;
            let size = *s.size.value()?;
            (size > 0 && !name.is_empty()).then(|| {
                let rank = match s.binding {
                    super::symbol::SymbolBinding::Global => 0,
                    super::symbol::SymbolBinding::Weak => 1,
                    _ => 2,
                };
                // A Thumb function's address carries bit 0; its bytes start at the even address.
                let address = if s.kind == SymbolKind::Function {
                    address & !1
                } else {
                    address
                };
                (
                    IndexedSymbol {
                        name,
                        address,
                        size,
                    },
                    rank,
                )
            })
        })
        .collect();
    out.sort_by(|(a, ra), (b, rb)| {
        a.address
            .cmp(&b.address)
            .then(ra.cmp(rb))
            .then_with(|| a.name.cmp(&b.name))
    });
    out.into_iter().map(|(s, _)| s).collect()
}

#[cfg(test)]
mod tests;
