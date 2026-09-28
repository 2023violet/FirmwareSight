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
export type { EvidenceSummaryDto } from './generated/EvidenceSummaryDto';
export type { FixtureKey } from './generated/FixtureKey';
export type { FixtureOptionDto } from './generated/FixtureOptionDto';
export type { IdentityDto } from './generated/IdentityDto';
export type { MemorySummaryDto } from './generated/MemorySummaryDto';
export type { SelectionDto } from './generated/SelectionDto';
