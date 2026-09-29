// The application's single import point for IPC types.
//
// `./generated/*` is ts-rs output and is never edited, so every module imports the shapes from
// here instead of reaching into that directory. When a type is added or renamed, this file is the
// only hand-written place that has to change.

export type { AnalysisSummaryDto } from './generated/AnalysisSummaryDto';
export type { ArtifactDto } from './generated/ArtifactDto';
export type { BudgetDto } from './generated/BudgetDto';
export type { ByteDeltaDto } from './generated/ByteDeltaDto';
export type { CandidatePageDto } from './generated/CandidatePageDto';
export type { CandidatePageRequestDto } from './generated/CandidatePageRequestDto';
export type { CapabilitiesDto } from './generated/CapabilitiesDto';
export type { ChangeKindCountsDto } from './generated/ChangeKindCountsDto';
export type { ChangeKindFilterDto } from './generated/ChangeKindFilterDto';
export type { CompareCandidateDto } from './generated/CompareCandidateDto';
export type { CompareRequestDto } from './generated/CompareRequestDto';
export type { CompareSummaryDto } from './generated/CompareSummaryDto';
export type { ContributorDto } from './generated/ContributorDto';
export type { DiffCountsDto } from './generated/DiffCountsDto';
export type { DiffMemoryDto } from './generated/DiffMemoryDto';
export type { DiffSideDto } from './generated/DiffSideDto';
export type { DiffSideMemoryDto } from './generated/DiffSideMemoryDto';
export type { DiffWarningDto } from './generated/DiffWarningDto';
export type { ErrorEnvelopeDto } from './generated/ErrorEnvelopeDto';
export type { EvidenceClassDto } from './generated/EvidenceClassDto';
export type { EvidencePageDto } from './generated/EvidencePageDto';
export type { EvidenceRequestDto } from './generated/EvidenceRequestDto';
export type { EvidenceRowDto } from './generated/EvidenceRowDto';
export type { EvidenceSortDto } from './generated/EvidenceSortDto';
export type { EvidenceSummaryDto } from './generated/EvidenceSummaryDto';
export type { ExportOutcomeDto } from './generated/ExportOutcomeDto';
export type { FixtureKey } from './generated/FixtureKey';
export type { FixtureOptionDto } from './generated/FixtureOptionDto';
export type { IdentityDto } from './generated/IdentityDto';
export type { MemorySummaryDto } from './generated/MemorySummaryDto';
export type { ObjectAttributionDto } from './generated/ObjectAttributionDto';
export type { SectionChangePageDto } from './generated/SectionChangePageDto';
export type { SectionChangeQueryDto } from './generated/SectionChangeQueryDto';
export type { SectionChangeRowDto } from './generated/SectionChangeRowDto';
export type { SectionChangeSortDto } from './generated/SectionChangeSortDto';
export type { SectionPageDto } from './generated/SectionPageDto';
export type { SectionRequestDto } from './generated/SectionRequestDto';
export type { SectionRowDto } from './generated/SectionRowDto';
export type { SectionSideDto } from './generated/SectionSideDto';
export type { SectionSortDto } from './generated/SectionSortDto';
export type { SelectionDto } from './generated/SelectionDto';
export type { SortDirDto } from './generated/SortDirDto';
export type { StoredBudgetDto } from './generated/StoredBudgetDto';
export type { SymbolChangePageDto } from './generated/SymbolChangePageDto';
export type { SymbolChangeQueryDto } from './generated/SymbolChangeQueryDto';
export type { SymbolChangeRowDto } from './generated/SymbolChangeRowDto';
export type { SymbolChangeSortDto } from './generated/SymbolChangeSortDto';
export type { SymbolPageDto } from './generated/SymbolPageDto';
export type { SymbolRequestDto } from './generated/SymbolRequestDto';
export type { SymbolRowDto } from './generated/SymbolRowDto';
export type { SymbolSideDto } from './generated/SymbolSideDto';
export type { SymbolSortDto } from './generated/SymbolSortDto';
