//! The P1 Analyze detail surface: three bounded queries over one stored snapshot.
//!
//! The commands are named for the three things a reader inspects, and they are the only detail
//! surface the shell registers. Each takes a snapshot id and a page, and returns one page: the
//! request carries no path, no table name and no statement, so the WebView cannot widen it
//! (`AGENTS.md` 7). The bounds themselves live in `firmwaresight-storage`, which means a caller
//! that asks for 100k rows gets 500 rather than the artifact.
//!
//! Projection here is a shape change, not a calculation. A byte count crosses as the number the
//! artifact carried and an address crosses as hexadecimal text, because converting a number to KiB
//! is a *presentation* decision the UI owns (US-001) and a locator scaled to a unit stops being a
//! locator. Nothing in this module parses, re-derives or re-reads an artifact.

use std::sync::Arc;

use firmwaresight_storage::{
    DEFAULT_QUERY_LIMIT, EvidenceClass, EvidenceQuery, EvidenceRow as StorageEvidenceRow,
    EvidenceSort, Fact, SectionQuery, SectionRow as StorageSectionRow, SectionSort, SortDir,
    SymbolQuery, SymbolRow as StorageSymbolRow, SymbolSort,
};
use tauri::State;

use crate::ipc::{
    ErrorEnvelopeDto, EvidenceClassDto, EvidencePageDto, EvidenceRequestDto, EvidenceRowDto,
    EvidenceSortDto, SectionPageDto, SectionRequestDto, SectionRowDto, SectionSortDto, SortDirDto,
    SymbolPageDto, SymbolRequestDto, SymbolRowDto, SymbolSortDto,
};
use crate::{Session, envelope_from_storage, next_operation_id, task_join_error};

impl Session {
    /// One bounded page of sections for a snapshot the session has stored.
    ///
    /// # Errors
    ///
    /// `ERR-STORAGE-4005` when the snapshot id names no build, and the storage envelope for a
    /// failed read. A poisoned lock is an internal envelope, the same as everywhere else here.
    pub fn query_sections(
        &self,
        request: &SectionRequestDto,
        operation_id: &str,
    ) -> Result<SectionPageDto, ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        let page = db
            .query_sections(&SectionQuery {
                snapshot_id: request.snapshot_id.clone(),
                name_contains: request.filter.clone(),
                sort_by: match request.sort {
                    SectionSortDto::Index => SectionSort::Index,
                    SectionSortDto::Name => SectionSort::Name,
                    SectionSortDto::FileSize => SectionSort::FileSize,
                    SectionSortDto::MemorySize => SectionSort::MemorySize,
                },
                direction: direction(request.direction),
                offset: to_index(request.offset),
                limit: to_limit(request.limit),
            })
            .map_err(|err| envelope_from_storage(&err, operation_id))?;

        Ok(SectionPageDto {
            rows: page.rows.into_iter().map(section_row).collect(),
            total: to_count(page.total),
            offset: to_count(page.offset),
            limit: to_count(page.limit),
            next_offset: page.next_offset.map(to_count),
        })
    }

    /// One bounded page of symbols for a snapshot the session has stored.
    ///
    /// # Errors
    ///
    /// As [`Session::query_sections`].
    pub fn query_symbols(
        &self,
        request: &SymbolRequestDto,
        operation_id: &str,
    ) -> Result<SymbolPageDto, ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        let page = db
            .query_symbols(&SymbolQuery {
                snapshot_id: request.snapshot_id.clone(),
                name_contains: request.filter.clone(),
                sort_by: match request.sort {
                    SymbolSortDto::Ordinal => SymbolSort::Ordinal,
                    SymbolSortDto::Name => SymbolSort::Name,
                    SymbolSortDto::Address => SymbolSort::Address,
                    SymbolSortDto::Size => SymbolSort::Size,
                },
                direction: direction(request.direction),
                offset: to_index(request.offset),
                limit: to_limit(request.limit),
            })
            .map_err(|err| envelope_from_storage(&err, operation_id))?;

        Ok(SymbolPageDto {
            rows: page.rows.into_iter().map(symbol_row).collect(),
            total: to_count(page.total),
            offset: to_count(page.offset),
            limit: to_count(page.limit),
            next_offset: page.next_offset.map(to_count),
        })
    }

    /// One bounded page of evidence for a snapshot the session has stored.
    ///
    /// # Errors
    ///
    /// As [`Session::query_sections`].
    pub fn query_evidence(
        &self,
        request: &EvidenceRequestDto,
        operation_id: &str,
    ) -> Result<EvidencePageDto, ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        let page = db
            .query_evidence(&EvidenceQuery {
                snapshot_id: request.snapshot_id.clone(),
                field_contains: request.filter.clone(),
                classification: request.classification.map(|class| match class {
                    EvidenceClassDto::Observed => EvidenceClass::Observed,
                    EvidenceClassDto::Derived => EvidenceClass::Derived,
                    EvidenceClassDto::Declared => EvidenceClass::Declared,
                    EvidenceClassDto::Unknown => EvidenceClass::Unknown,
                }),
                sort_by: match request.sort {
                    EvidenceSortDto::Field => EvidenceSort::Field,
                    EvidenceSortDto::Classification => EvidenceSort::Classification,
                },
                direction: direction(request.direction),
                offset: to_index(request.offset),
                limit: to_limit(request.limit),
            })
            .map_err(|err| envelope_from_storage(&err, operation_id))?;

        Ok(EvidencePageDto {
            rows: page.rows.into_iter().map(evidence_row).collect(),
            total: to_count(page.total),
            offset: to_count(page.offset),
            limit: to_count(page.limit),
            next_offset: page.next_offset.map(to_count),
        })
    }
}

fn direction(value: SortDirDto) -> SortDir {
    match value {
        SortDirDto::Asc => SortDir::Asc,
        SortDirDto::Desc => SortDir::Desc,
    }
}

/// A page position that fits the storage layer's integer. A wider request is not a valid cursor,
/// and clamping it keeps the boundary total rather than panicking on a cast.
fn to_index(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// No requested size means the boundary's default; any size is still clamped again in Rust.
fn to_limit(value: Option<usize>) -> i64 {
    value.map_or(DEFAULT_QUERY_LIMIT, |value| {
        i64::try_from(value).unwrap_or(i64::MAX)
    })
}

fn to_count(value: i64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

fn section_row(row: StorageSectionRow) -> SectionRowDto {
    let (name, name_unknown_reason) = fact_text(row.name);
    let (virtual_address, virtual_address_unknown_reason) = hex_fact(row.virtual_address);
    let (load_address, load_address_unknown_reason) = hex_fact(row.load_address);
    let (memory_size, memory_size_unknown_reason) = fact_bytes(row.memory_size);
    let (region, region_unknown_reason) = fact_text(row.region);
    let (file_offset, file_offset_unknown_reason) = hex_fact(row.file_offset);

    SectionRowDto {
        index: to_count(row.index),
        name,
        name_unknown_reason,
        role: row.role,
        alloc: row.alloc,
        write: row.write,
        execute: row.execute,
        virtual_address,
        virtual_address_unknown_reason,
        load_address,
        load_address_unknown_reason,
        file_offset,
        file_offset_unknown_reason,
        file_size: row.file_size,
        memory_size,
        memory_size_unknown_reason,
        region,
        region_unknown_reason,
    }
}

fn symbol_row(row: StorageSymbolRow) -> SymbolRowDto {
    let (name, name_unknown_reason) = fact_text(row.name);
    let (size, size_unknown_reason) = fact_bytes(row.size);
    let (address, address_unknown_reason) = hex_fact(row.address);

    SymbolRowDto {
        ordinal: to_count(row.ordinal),
        name,
        name_unknown_reason,
        address,
        address_unknown_reason,
        size,
        size_unknown_reason,
        kind: row.kind,
        binding: row.binding,
        section_ref: row.section_ref,
    }
}

fn evidence_row(row: StorageEvidenceRow) -> EvidenceRowDto {
    EvidenceRowDto {
        id: row.id,
        field: row.field,
        classification: row.classification,
        source_type: row.source_type,
        source_locator: row.source_locator,
        raw_value: row.raw_value,
        rule: row.rule,
        confidence: row.confidence,
    }
}

/// A value and its reason, never both and never neither.
fn fact_text(value: Fact<String>) -> (Option<String>, Option<String>) {
    match value {
        Fact::Known(value) => (Some(value), None),
        Fact::Unknown { reason } => (None, Some(reason)),
    }
}

fn fact_bytes(value: Fact<u64>) -> (Option<u64>, Option<String>) {
    match value {
        Fact::Known(value) => (Some(value), None),
        Fact::Unknown { reason } => (None, Some(reason)),
    }
}

fn hex_fact(value: Fact<u64>) -> (Option<String>, Option<String>) {
    match value {
        Fact::Known(value) => (Some(hex_value(value)), None),
        Fact::Unknown { reason } => (None, Some(reason)),
    }
}

/// The same hexadecimal form the CLI prints for an entry point, so one address looks identical on
/// both surfaces.
fn hex_value(value: u64) -> String {
    format!("{value:#010x}")
}

/// The section table, one page at a time.
#[tauri::command]
pub(crate) async fn query_sections(
    request: SectionRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<SectionPageDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || session.query_sections(&request, &operation_id))
        .await
        .map_err(task_join_error)?
}

/// The symbol table, one page at a time.
#[tauri::command]
pub(crate) async fn query_symbols(
    request: SymbolRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<SymbolPageDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || session.query_symbols(&request, &operation_id))
        .await
        .map_err(task_join_error)?
}

/// The evidence register, one page at a time.
#[tauri::command]
pub(crate) async fn query_evidence(
    request: EvidenceRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<EvidencePageDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || session.query_evidence(&request, &operation_id))
        .await
        .map_err(task_join_error)?
}
