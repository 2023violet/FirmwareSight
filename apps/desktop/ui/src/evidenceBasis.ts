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
 *
 * B1-U3 moved the six sentences into the `common` catalog (`ADR-0032` D5) and left this file as the
 * routing table only. That is deliberate: a caption is reader-facing prose, so a Chinese screen has to
 * read it in Chinese, while the identifiers above it stay exactly as Core wrote them. A value outside
 * the list still surfaces, now as the catalog's own sentence for "not recognized".
 */

import type { Translate } from './i18n';
import type { Catalog } from './i18n/en';

type BasisKey = keyof Catalog['common'];

const EVIDENCE_BASIS_KEYS = new Map<string, BasisKey>([
  ['MapRegionAndElfLoad', 'evidenceBasis.mapElf'],
  ['map-memory-configuration+elf-load', 'evidenceBasis.mapElf'],
  ['RegionConfigAndElfLoad', 'evidenceBasis.regionElf'],
  ['region-config+elf-load', 'evidenceBasis.regionElf'],
  ['ElfAddressAndFlags', 'evidenceBasis.addressFlags'],
  ['elf-address-and-flags', 'evidenceBasis.addressFlags'],
  ['SectionNameHeuristic', 'evidenceBasis.sectionName'],
  ['section-name-heuristic', 'evidenceBasis.sectionName'],
  ['Insufficient', 'evidenceBasis.insufficient'],
  ['unattributed', 'evidenceBasis.insufficient'],
]);

export function evidenceBasisCaption(basis: string, t: Translate): string {
  const key = EVIDENCE_BASIS_KEYS.get(basis);
  return t('common', key ?? 'evidenceBasis.unrecognized');
}
