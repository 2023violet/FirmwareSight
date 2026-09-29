// The application's single import point for IPC types.
//
// `./generated/*` is ts-rs output and is never edited, so every module imports the shapes from
// here instead of reaching into that directory. When a type is added or renamed, this file is the
// only hand-written place that has to change.

export type { AnalysisSummaryDto } from './generated/AnalysisSummaryDto';
export type { ArtifactDto } from './generated/ArtifactDto';
export type { BudgetDto } from './generated/BudgetDto';
export type { CapabilitiesDto } from './generated/CapabilitiesDto';
export type { ErrorEnvelopeDto } from './generated/ErrorEnvelopeDto';
export type { EvidenceClassDto } from './generated/EvidenceClassDto';
export type { EvidencePageDto } from './generated/EvidencePageDto';
export type { EvidenceRequestDto } from './generated/EvidenceRequestDto';
export type { EvidenceRowDto } from './generated/EvidenceRowDto';
export type { EvidenceSortDto } from './generated/EvidenceSortDto';
export type { EvidenceSummaryDto } from './generated/EvidenceSummaryDto';
export type { FixtureKey } from './generated/FixtureKey';
export type { FixtureOptionDto } from './generated/FixtureOptionDto';
export type { IdentityDto } from './generated/IdentityDto';
export type { MemorySummaryDto } from './generated/MemorySummaryDto';
export type { SectionPageDto } from './generated/SectionPageDto';
export type { SectionRequestDto } from './generated/SectionRequestDto';
export type { SectionRowDto } from './generated/SectionRowDto';
export type { SectionSortDto } from './generated/SectionSortDto';
export type { SelectionDto } from './generated/SelectionDto';
export type { SortDirDto } from './generated/SortDirDto';
export type { SymbolPageDto } from './generated/SymbolPageDto';
export type { SymbolRequestDto } from './generated/SymbolRequestDto';
export type { SymbolRowDto } from './generated/SymbolRowDto';
export type { SymbolSortDto } from './generated/SymbolSortDto';
