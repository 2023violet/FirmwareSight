/**
 * The one place that turns a memory-evidence basis into words a person reads.
 *
 * `weakest_basis` and a budget row's basis cross IPC as identifiers the domain owns: the Debug name of
 * `MemoryEvidenceBasis` (`crates/firmwaresight-core/src/domain/memory.rs`) or the kebab spelling the
 * wire and storage use. Both are portable values, and neither is a caption — printing one in a
 * human-facing column is the defect L20 closed on Compare (`P5_VALIDATION/P5_SUPPORTABILITY_REPORT.md`)
 * and F2 found again on Release. Mapping lives here so every surface says the same sentence, and a
 * value outside the list stays visible as what it is rather than being dressed up as a known one.
 *
 * Presentation only: no enum is renamed, no basis is recomputed, nothing here is serialized, and each
 * surface keeps its own wording for a basis that was never recorded.
 */

const EVIDENCE_BASIS_CAPTIONS = new Map<string, string>([
  ['MapRegionAndElfLoad', 'MAP regions + ELF load evidence'],
  ['map-memory-configuration+elf-load', 'MAP regions + ELF load evidence'],
  ['RegionConfigAndElfLoad', 'configured regions + ELF load evidence'],
  ['region-config+elf-load', 'configured regions + ELF load evidence'],
  ['ElfAddressAndFlags', 'ELF address/flags evidence'],
  ['elf-address-and-flags', 'ELF address/flags evidence'],
  ['SectionNameHeuristic', 'section-name heuristic'],
  ['section-name-heuristic', 'section-name heuristic'],
  ['Insufficient', 'insufficient evidence'],
  ['unattributed', 'insufficient evidence'],
]);

export function evidenceBasisCaption(basis: string): string {
  return EVIDENCE_BASIS_CAPTIONS.get(basis) ?? 'Unrecognized evidence basis';
}
