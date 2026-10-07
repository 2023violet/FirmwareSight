/**
 * The Compare page: two builds this application already analyzed, and what moved between them.
 *
 * Four rules decide the shape of this screen.
 *
 * 1. **Nothing is computed here.** Deltas, change kinds, comparability and ranking come from Core
 *    through the shell; this page chooses words and layout, and filters, sorts and pages by asking
 *    the shell to do the same (`AGENTS.md` 3, `04_TECH/14` 3).
 * 2. **Absence is a word, not a zero.** An added section has no old size, so its Old column reads
 *    `Not present`, and a delta that could not be computed reads `Unknown` with the reason beside it
 *    (DESIGN.md 5; prompt §23, §38).
 * 3. **The full lists are the answer; the ranking is a way to them.** Top growth and largest
 *    additions are capped navigation aids, and each entry is a control that filters the table below
 *    to the row it names (prompt §25, §26, §41).
 * 4. **A failed comparison does not delete the last good one.** The summary moves only when the
 *    shell returns one, and the surviving report says which pair of builds it belongs to and which
 *    pair it does not (prompt §42).
 */

import { useCallback, useEffect, useRef, useState } from 'react';

import styles from './Compare.module.css';
import { Button } from './components/Button';
import { ErrorPanel } from './components/ErrorPanel';
import { Page, ScrollArea } from './components/Layout';
import { PageHeader } from './components/PageHeader';
import { EmptyState } from './components/Panel';
import { Pager, SortHeader } from './components/Table';
import { SizeUnitSwitch } from './components/SizeUnitSwitch';
import { StateBadge, type StateName } from './components/StateBadge';
import { evidenceBasisCaption } from './evidenceBasis';
import { ABSENT, formatDelta, formatSize, truncateMiddle, type SizeUnit } from './format';
import {
  compareSnapshots,
  exportCompareHtml,
  exportCompareJson,
  listCompareCandidates,
  querySectionChanges,
  querySymbolChanges,
} from './ipc/bridge';
import type {
  ByteDeltaDto,
  CandidatePageDto,
  ChangeKindCountsDto,
  ChangeKindFilterDto,
  CompareCandidateDto,
  CompareSummaryDto,
  ContributorDto,
  DiffCountsDto,
  DiffMemoryDto,
  DiffSideMemoryDto,
  ErrorEnvelopeDto,
  ExportOutcomeDto,
  SectionChangePageDto,
  SectionChangeRowDto,
  SectionChangeSortDto,
  SortDirDto,
  SymbolChangePageDto,
  SymbolChangeRowDto,
  SymbolChangeSortDto,
} from './ipc/types';
import { cx } from './styles/classnames';

/**
 * The additions lists are one small page of the authoritative change table, ranked by the shell on
 * the same quantity the growth lists use. They are not a second copy of the diff (prompt §25).
 */
const ADDITION_LIMIT = 5;

/** Which side of a change row is being shown: only the unpaired side can be genuinely absent. */
type Side = 'base' | 'target';

/** A drill-down request: the row the reader chose, in the table that holds it. */
type Focus = { readonly key: string };

/** The largest additions, read back from the change tables. */
type Additions = {
  readonly sections: readonly ContributorDto[];
  readonly symbols: readonly ContributorDto[];
};

/** All, Added, Removed, Changed - the four views prompt §25 asks the full lists to offer. */
const KIND_FILTERS: readonly {
  readonly label: string;
  readonly value: ChangeKindFilterDto | null;
}[] = [
  { label: 'All', value: null },
  { label: 'Added', value: 'added' },
  { label: 'Removed', value: 'removed' },
  { label: 'Changed', value: 'changed' },
];

export function Compare({
  unit,
  onUnitChange,
  lastAnalyzedSnapshotId,
  onGoToAnalyze,
}: {
  readonly unit: SizeUnit;
  readonly onUnitChange: (unit: SizeUnit) => void;
  readonly lastAnalyzedSnapshotId: string | null;
  readonly onGoToAnalyze: () => void;
}) {
  const [candidates, setCandidates] = useState<CandidatePageDto | null>(null);
  const [candidateError, setCandidateError] = useState<ErrorEnvelopeDto | null>(null);
  const [baseId, setBaseId] = useState<string | null>(null);
  const [targetId, setTargetId] = useState<string | null>(null);
  const [summary, setSummary] = useState<CompareSummaryDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [comparing, setComparing] = useState(false);
  const [sectionFocus, setSectionFocus] = useState<Focus | null>(null);
  const [symbolFocus, setSymbolFocus] = useState<Focus | null>(null);
  const [additions, setAdditions] = useState<Additions | null>(null);
  const [exportNote, setExportNote] = useState<string | null>(null);
  const [exportError, setExportError] = useState<ErrorEnvelopeDto | null>(null);
  const [exporting, setExporting] = useState(false);

  const additionsRequest = useRef(0);

  /**
   * The stored builds, one bounded page at a time, read by Rust from SQLite rather than from the
   * filesystem: a build whose original file has since moved stays comparable.
   */
  useEffect(() => {
    let abandoned = false;
    void listCompareCandidates({ offset: null, limit: null }).then((outcome) => {
      if (abandoned) {
        return;
      }
      if (outcome.ok) {
        setCandidates(outcome.value);
        setCandidateError(null);
      } else {
        setCandidateError(outcome.envelope);
      }
    });
    return () => {
      abandoned = true;
    };
  }, []);

  /**
   * Name a default pair once the list exists (prompt §18).
   *
   * The build Analyze last proved is the one the reader most likely wants to compare, so it becomes
   * the target and the next distinct build becomes the base. Choosing is not comparing: no diff runs
   * until the reader presses Compare.
   */
  useEffect(() => {
    if (
      candidates === null ||
      candidates.rows.length === 0 ||
      baseId !== null ||
      targetId !== null
    ) {
      return;
    }
    const mostRecent = candidates.rows[0];
    if (mostRecent === undefined) {
      return;
    }
    const preferred =
      candidates.rows.find((row) => row.snapshotId === lastAnalyzedSnapshotId) ?? mostRecent;
    const other = candidates.rows.find((row) => row.snapshotId !== preferred.snapshotId) ?? null;
    setTargetId(preferred.snapshotId);
    setBaseId(other === null ? null : other.snapshotId);
  }, [baseId, candidates, lastAnalyzedSnapshotId, targetId]);

  /** More stored builds, appended to the list the selectors already hold. */
  const loadMore = useCallback(async () => {
    if (candidates === null || candidates.nextOffset === null) {
      return;
    }
    const outcome = await listCompareCandidates({ offset: candidates.nextOffset, limit: null });
    if (!outcome.ok) {
      setCandidateError(outcome.envelope);
      return;
    }
    setCandidates({
      rows: [...candidates.rows, ...outcome.value.rows],
      total: candidates.total,
      offset: candidates.offset,
      limit: candidates.limit,
      nextOffset: outcome.value.nextOffset,
    });
  }, [candidates]);

  /**
   * Compare the chosen pair, and keep the previous comparison on the failure path.
   *
   * `summary` only ever moves when the shell returns one, so a failed attempt changes the error and
   * the label on the surviving report - nothing else.
   */
  const compare = useCallback(async () => {
    if (baseId === null || targetId === null || baseId === targetId) {
      return;
    }
    setComparing(true);
    const outcome = await compareSnapshots({ baseSnapshotId: baseId, targetSnapshotId: targetId });
    setComparing(false);

    if (outcome.ok) {
      setSummary(outcome.value);
      setError(null);
      setExportNote(null);
      setExportError(null);
      setSectionFocus(null);
      setSymbolFocus(null);
      return;
    }
    setError(outcome.envelope);
  }, [baseId, targetId]);

  const swap = useCallback(() => {
    setBaseId(targetId);
    setTargetId(baseId);
  }, [baseId, targetId]);

  /* The largest additions come from the same bounded query the table below uses, so a capped list
     and the full list cannot disagree about which rows exist. If this read fails, that table asks
     the shell the same question and reports the failure where its own rows belong. */
  useEffect(() => {
    if (summary === null) {
      setAdditions(null);
      return;
    }
    const id = ++additionsRequest.current;
    const diffId = summary.diffId;
    void Promise.all([
      querySectionChanges({
        diffId,
        filter: null,
        changeKind: 'added',
        sort: 'memorySize',
        direction: 'desc',
        offset: 0,
        limit: ADDITION_LIMIT,
      }),
      querySymbolChanges({
        diffId,
        filter: null,
        changeKind: 'added',
        sort: 'size',
        direction: 'desc',
        offset: 0,
        limit: ADDITION_LIMIT,
      }),
    ]).then(([sections, symbols]) => {
      if (additionsRequest.current !== id) {
        return;
      }
      if (!sections.ok || !symbols.ok) {
        setAdditions(null);
        return;
      }
      setAdditions({
        sections: sections.value.rows.map(sectionAddition),
        symbols: symbols.value.rows.map(symbolAddition),
      });
    });
  }, [summary]);

  const exportAs = useCallback(
    async (format: 'json' | 'html') => {
      if (summary === null) {
        return;
      }
      setExporting(true);
      const outcome =
        format === 'json'
          ? await exportCompareJson(summary.diffId)
          : await exportCompareHtml(summary.diffId);
      setExporting(false);

      if (!outcome.ok) {
        // A failed export does not invalidate the comparison (prompt §42), so nothing above moves.
        setExportError(outcome.envelope);
        setExportNote(null);
        return;
      }
      setExportError(null);
      setExportNote(exportWords(outcome.value));
    },
    [summary],
  );

  const rows = candidates?.rows ?? [];
  const base = rows.find((row) => row.snapshotId === baseId) ?? null;
  const target = rows.find((row) => row.snapshotId === targetId) ?? null;
  const bothChosen = base !== null && target !== null;
  const samePair = bothChosen && baseId === targetId;
  const hasTwoBuilds = (candidates?.total ?? 0) >= 2;
  // The report answers for the pair it was computed from, not for the pair the selectors happen to
  // name now. A swap or a failed attempt leaves the previous diff standing, so the screen says whose
  // facts the reader is looking at (prompt §18).
  const stale =
    summary !== null &&
    (error !== null ||
      baseId !== summary.base.snapshotId ||
      targetId !== summary.target.snapshotId);

  return (
    <Page>
      <PageHeader
        title="Compare"
        subhead="Pick two builds FirmwareSight has already analyzed and stored, and it reports what moved between those recorded facts. A comparison does not re-read the original files, and nothing here writes to them."
      />

      {candidateError === null ? null : (
        <ErrorPanel
          envelope={candidateError}
          label="Build list error"
          heading="The stored builds would not load"
        />
      )}

      {candidates === null ? (
        <p className={styles['status']} role="status" aria-live="polite">
          Reading the stored builds…
        </p>
      ) : null}

      {candidates === null || hasTwoBuilds || candidateError !== null ? null : (
        // Fewer than two builds is a fact about the session, and the screen says so instead of
        // inventing a comparison out of one build (prompt §19).
        <EmptyState
          label="Nothing to compare yet"
          message="Analyze another firmware build before comparing."
          nextStep={<Button onClick={onGoToAnalyze}>Go to Analyze</Button>}
        />
      )}

      {candidates === null || !hasTwoBuilds ? null : (
        <section className={styles['selectors']} aria-label="Build selection">
          <SnapshotPicker
            id="fs-base"
            label="Old / Base"
            rows={rows}
            selected={baseId}
            lastAnalyzedSnapshotId={lastAnalyzedSnapshotId}
            unit={unit}
            disabled={comparing}
            onSelect={setBaseId}
          />
          <SnapshotPicker
            id="fs-target"
            label="New / Target"
            rows={rows}
            selected={targetId}
            lastAnalyzedSnapshotId={lastAnalyzedSnapshotId}
            unit={unit}
            disabled={comparing}
            onSelect={setTargetId}
          />
          <div className={styles['pickerActions']}>
            {candidates.nextOffset === null ? null : (
              <Button
                onClick={() => {
                  void loadMore();
                }}
                disabled={comparing}
              >
                Show more builds
              </Button>
            )}
            <Button onClick={swap} disabled={!bothChosen || samePair || comparing}>
              Swap
            </Button>
            <Button
              variant="primary"
              onClick={() => {
                void compare();
              }}
              disabled={!bothChosen || samePair || comparing}
            >
              Compare
            </Button>
          </div>
        </section>
      )}

      {samePair ? (
        // Refusing is not a failure to report: the pair is named, and the action stays locked until
        // the reader changes it (prompt §18).
        <p className={styles['warning']} role="note">
          Both sides name the same build
          {base === null ? '.' : `: ${buildLabel(base)}.`} A build compared with itself reports
          nothing, which is not the same as nothing having changed. Choose a different Old / Base or
          New / Target.
        </p>
      ) : null}

      {comparing ? (
        <p className={styles['status']} role="status" aria-live="polite">
          Comparing…
        </p>
      ) : null}

      {error === null ? null : (
        <ErrorPanel envelope={error} label="Comparison error" heading="Comparison failed" />
      )}

      <SizeUnitSwitch unit={unit} onSelect={onUnitChange} />

      {summary === null ? null : (
        <section
          className={styles['report']}
          aria-label={stale ? 'Last good comparison' : 'Build comparison'}
        >
          {stale ? (
            <p className={styles['stale']} role="note">
              {error === null
                ? `This is the comparison of ${buildLabel(summary.base)} and ${buildLabel(summary.target)} that you asked for. The pair selected above is a different one; press Compare to move to it.`
                : `Previous comparison of ${buildLabel(summary.base)} and ${buildLabel(summary.target)}. The attempt above produced no result, so nothing here was replaced.`}
            </p>
          ) : null}

          <MemoryComparison memory={summary.memory} unit={unit} />
          <EvidenceNotice summary={summary} />
          <WhatMoved counts={summary.counts} />

          <Ranking
            summary={summary}
            additions={additions}
            unit={unit}
            onPickSection={(key) => {
              setSectionFocus({ key });
            }}
            onPickSymbol={(key) => {
              setSymbolFocus({ key });
            }}
          />

          {/* One pair of tables for one comparison: each resets its own filter when the handle it
              was given changes. */}
          <SectionChanges
            diffId={summary.diffId}
            unit={unit}
            focus={sectionFocus}
          />
          <SymbolChanges
            diffId={summary.diffId}
            unit={unit}
            focus={symbolFocus}
          />

          <Exports
            note={exportNote}
            error={exportError}
            exporting={exporting}
            onExport={(format) => {
              void exportAs(format);
            }}
          />

          <footer className={styles['footer']}>
            <span className={styles['monoSmall']}>
              Base {summary.base.snapshotId} · {truncateMiddle(summary.base.sha256, 8)}
            </span>
            <span className={styles['monoSmall']}>
              Target {summary.target.snapshotId} · {truncateMiddle(summary.target.sha256, 8)}
            </span>
            <span>
              The diff id is this session&rsquo;s handle for the computed comparison: not stored, not
              portable, and not part of what the diff proves.
            </span>
          </footer>
        </section>
      )}
    </Page>
  );
}

/**
 * Names one stored build the way the pickers do. Two builds of one artifact share a file name, so a
 * name alone cannot tell the pair on screen from the pair just selected - which is exactly what the
 * "this is not that comparison" note has to do (prompt §18).
 */
function buildLabel(row: { readonly fileName: string; readonly sha256: string }): string {
  return `${row.fileName} · ${row.sha256.slice(0, 12)}`;
}

/** One selector, with the recorded facts of the build it currently names. */
function SnapshotPicker({
  id,
  label,
  rows,
  selected,
  lastAnalyzedSnapshotId,
  unit,
  disabled,
  onSelect,
}: {
  readonly id: string;
  readonly label: string;
  readonly rows: readonly CompareCandidateDto[];
  readonly selected: string | null;
  readonly lastAnalyzedSnapshotId: string | null;
  readonly unit: SizeUnit;
  readonly disabled: boolean;
  readonly onSelect: (snapshotId: string | null) => void;
}) {
  const chosen = rows.find((row) => row.snapshotId === selected) ?? null;

  return (
    <div className={styles['picker']}>
      <label className={styles['pickerLabel']} htmlFor={id}>
        {label}
      </label>
      <select
        id={id}
        className={styles['select']}
        value={selected ?? ''}
        disabled={disabled}
        onChange={(event) => {
          onSelect(event.target.value === '' ? null : event.target.value);
        }}
      >
        <option value="">Choose a build</option>
        {rows.map((row) => (
          <option
            key={row.snapshotId}
            value={row.snapshotId}
            title={`${row.fileName} · sha256 ${row.sha256} · imported ${row.importedAt}`}
          >
            {buildLabel(row)}
            {row.snapshotId === lastAnalyzedSnapshotId ? ' · last analyzed' : ''}
          </option>
        ))}
      </select>

      {chosen === null ? (
        <p className={styles['sideFacts']}>No build named yet.</p>
      ) : (
        <p className={styles['sideFacts']}>
          <span className={styles['mono']}>{formatSize(chosen.byteSize, unit)}</span>
          <span>{chosen.architecture}</span>
          <span>
            Nonvolatile {budgetWords(chosen.nonvolatile.state, chosen.nonvolatile.bytes, unit)}
          </span>
          <span>
            Runtime RAM {budgetWords(chosen.runtimeRam.state, chosen.runtimeRam.bytes, unit)}
          </span>
          <span className={styles['monoSmall']}>imported {chosen.importedAt}</span>
        </p>
      )}
    </div>
  );
}

function budgetWords(state: string, bytes: number | null, unit: SizeUnit): string {
  return `${state} ${formatSize(bytes, unit)}`;
}

/**
 * The two memory totals, each with both sides and the signed difference.
 *
 * Comparability is the strength of *this* comparison, not of one side: an exact total measured
 * against a partial one is a partial delta, which is why the word sits beside the figures rather
 * than under a heading of its own (prompt §38, `04_TECH/23`).
 */
function MemoryComparison({
  memory,
  unit,
}: {
  readonly memory: DiffMemoryDto;
  readonly unit: SizeUnit;
}) {
  return (
    <section className={styles['section']} aria-label="Memory comparison">
      <h2>Memory</h2>
      <p className={styles['caption']}>
        Delta = target &minus; base. A positive delta is growth. Unknown and{' '}
        &ldquo;{ABSENT}&rdquo; are never written as 0.
      </p>
      <div className={styles['rows']}>
        <MemoryRow term="Nonvolatile / load image" change={memory.nonvolatile} unit={unit} />
        <MemoryRow term="Runtime RAM" change={memory.runtimeRam} unit={unit} />
        <div className={styles['row']}>
          <span className={styles['term']}>Pair comparability</span>
          <span className={styles['value']}>
            <Comparability value={memory.comparability} />
            <span className={styles['note']}>
              The weaker side caps the pair, so this is the strength of the delta, not of either
              build alone.
            </span>
          </span>
        </div>
        <SideEvidenceRow label="Base evidence" side={memory.base} />
        <SideEvidenceRow label="Target evidence" side={memory.target} />
      </div>
    </section>
  );
}

function MemoryRow({
  term,
  change,
  unit,
}: {
  readonly term: string;
  readonly change: ByteDeltaDto;
  readonly unit: SizeUnit;
}) {
  return (
    <div className={styles['row']}>
      <span className={styles['term']}>{term}</span>
      <span className={styles['value']}>
        <Figure axis="Old" text={formatSize(change.base, unit)} />
        <Figure axis="New" text={formatSize(change.target, unit)} />
        <DeltaFigure change={change} unit={unit} />
      </span>
    </div>
  );
}

/**
 * One number with the word that says which axis it belongs to.
 *
 * A delta may take the diff colour, because the sign and the word are both on the page; a side value
 * never does, since `Not present` and `Unknown` are not colours.
 */
function Figure({
  axis,
  text,
  delta,
  reason,
}: {
  readonly axis: string;
  readonly text: string;
  readonly delta?: number | null;
  readonly reason?: string | null;
}) {
  const sign =
    delta === undefined || delta === null
      ? undefined
      : delta > 0
        ? styles['deltaPositive']
        : delta < 0
          ? styles['deltaNegative']
          : undefined;
  return (
    <span className={styles['figure']}>
      {/* The axis word carries its own spacing, so the pair reads as one sentence in the DOM and not
          only as two columns on the screen. */}
      <span className={styles['axis']}>{`${axis} `}</span>
      <span className={cx(styles['mono'], sign)}>{text}</span>
      {reason === undefined || reason === null ? null : (
        <span className={styles['note']}> {reason}</span>
      )}
    </span>
  );
}

function DeltaFigure({
  change,
  unit,
}: {
  readonly change: ByteDeltaDto;
  readonly unit: SizeUnit;
}) {
  return (
    <Figure
      axis="Delta"
      text={formatDelta(change.delta, unit)}
      delta={change.delta}
      reason={change.delta === null ? change.reason : null}
    />
  );
}

/// What Compare adds to the shared evidence-basis caption: its own word for a side that recorded none.
///
/// The mapping of the five bases the memory model can name lives in `evidenceBasis.ts`, in the
/// vocabulary Analyze already uses, so this page and Release cannot drift apart.
function basisCaption(basis: string | null): string {
  if (basis === null) {
    return 'unknown';
  }
  return evidenceBasisCaption(basis);
}

function SideEvidenceRow({
  label,
  side,
}: {
  readonly label: string;
  readonly side: DiffSideMemoryDto;
}) {
  return (
    <div className={styles['row']}>
      <span className={styles['term']}>{label}</span>
      <span className={styles['value']}>
        <StateBadge
          state={side.mapBacked ? 'PASS' : 'UNKNOWN'}
          label={side.mapBacked ? 'Linker MAP used' : 'No linker MAP'}
        />
        <span className={styles['monoSmall']}>layout {side.layoutSource}</span>
        <span className={styles['monoSmall']}>weakest basis {basisCaption(side.weakestEvidenceBasis)}</span>
        <span className={styles['note']}>
          {side.footprintRowPresent
            ? 'a footprint row was recorded'
            : 'no footprint row was recorded, which is not a record of zero'}
        </span>
      </span>
    </div>
  );
}

/**
 * The facts that say how much of this comparison rests on evidence (prompt §43).
 *
 * Only what the shell recorded appears: a warning the diff raised, the attribution capability, and
 * the per-side layout basis in the Memory block. No per-row provenance is claimed, because the
 * stored facts do not support one.
 */
function EvidenceNotice({ summary }: { readonly summary: CompareSummaryDto }) {
  const { memory, warnings, objectChanges } = summary;
  const lines: readonly string[] = [
    ...(memory.evidenceWarning === null ? [] : [memory.evidenceWarning]),
    ...warnings.map((warning) => `${warning.code}: ${warning.message}`),
  ];

  return (
    <section className={styles['section']} aria-label="Capability and evidence">
      <h2>Capability and evidence</h2>
      {lines.length === 0 ? (
        <p className={styles['status']}>No evidence warning was recorded for this pair.</p>
      ) : (
        <ul className={styles['noticeList']}>
          {lines.map((line) => (
            <li key={line} className={styles['noticeItem']}>
              {line}
            </li>
          ))}
        </ul>
      )}
      <div className={styles['row']}>
        {/* L19: this page answers "did any object file change between the two builds", which is not
            the question Analyze's capability row answers. The two labels must stay distinct. */}
        <span className={styles['term']}>Object-level change attribution</span>
        <span className={styles['value']}>
          <StateBadge
            state={objectChanges.available ? 'PASS' : 'N/A'}
            label={objectChanges.available ? 'available' : 'unavailable'}
            note={objectChanges.available ? undefined : objectChanges.reason}
          />
        </span>
      </div>
    </section>
  );
}

/**
 * How many rows moved, in diff vocabulary.
 *
 * Added, Removed and Changed are not Gate states and borrow no Gate icon: each is a word with a
 * count, and the unpaired count says what the pairing could not decide (prompt §40).
 */
function WhatMoved({ counts }: { readonly counts: DiffCountsDto }) {
  return (
    <section className={styles['section']} aria-label="What moved">
      <h2>What moved</h2>
      <div className={styles['rows']}>
        <CountRow term="Sections" counts={counts.sections} unchanged={counts.unchangedSections} />
        <CountRow term="Symbols" counts={counts.symbols} unchanged={counts.unchangedSymbols} />
      </div>
    </section>
  );
}

function CountRow({
  term,
  counts,
  unchanged,
}: {
  readonly term: string;
  readonly counts: ChangeKindCountsDto;
  readonly unchanged: number;
}) {
  return (
    <div className={styles['row']}>
      <span className={styles['term']}>{term}</span>
      <span className={cx(styles['value'], styles['counts'])}>
        <Kind kind="Added" count={counts.added} />
        <Kind kind="Removed" count={counts.removed} />
        <Kind kind="Changed" count={counts.changed} />
        <span className={styles['note']}>{String(unchanged)} unchanged</span>
        {counts.ambiguous === 0 ? null : (
          <span className={styles['unknown']}>
            {String(counts.ambiguous)} unpaired
            <span className={styles['note']}>
              the name repeats, or there is no name to match on
            </span>
          </span>
        )}
      </span>
    </div>
  );
}

function Kind({ kind, count }: { readonly kind: string; readonly count?: number }) {
  return (
    <span className={cx(styles['kind'], kindClass(kind))}>
      {kind}
      {count === undefined ? null : ` ${String(count)}`}
    </span>
  );
}

/** The word is the state; the diff roles only reinforce it. An unrecognised word stays neutral. */
function kindClass(kind: string): string | undefined {
  switch (kind) {
    case 'Added':
      return styles['kindAdded'];
    case 'Removed':
      return styles['kindRemoved'];
    case 'Changed':
      return styles['kindChanged'];
    default:
      return undefined;
  }
}

/**
 * The two capped lists, kept apart because they answer different questions (prompt §26).
 *
 * Growth is a Changed row that got bigger; an addition is a row that did not exist before. Folding
 * them into one ranking would imply an addition grew from zero, which the diff refuses to claim.
 */
function Ranking({
  summary,
  additions,
  unit,
  onPickSection,
  onPickSymbol,
}: {
  readonly summary: CompareSummaryDto;
  readonly additions: Additions | null;
  readonly unit: SizeUnit;
  readonly onPickSection: (key: string) => void;
  readonly onPickSymbol: (key: string) => void;
}) {
  return (
    <section className={styles['section']} aria-label="Top growth and largest additions">
      <h2>Top growth and largest additions</h2>
      <p className={styles['hint']}>
        These lists are navigation aids. The Section Changes and Symbol Changes tables below hold
        every changed row, and choosing an entry filters that table to the row it names.
      </p>
      <div className={styles['ranking']}>
        <div className={styles['list']}>
          <h3>Top growth</h3>
          <p className={styles['hint']}>
            Changed rows only, ranked by the runtime-size delta. An added row is not growth from zero,
            so it never appears here.
          </p>
          <Contributors
            noun="sections"
            rows={summary.topSections}
            unit={unit}
            empty="No section grew by a known amount."
            onPick={onPickSection}
          />
          <Contributors
            noun="symbols"
            rows={summary.topSymbols}
            unit={unit}
            empty="No symbol grew by a known amount."
            onPick={onPickSymbol}
          />
        </div>
        <div className={styles['list']}>
          <h3>Largest additions</h3>
          <p className={styles['hint']}>
            Added rows, ranked by their runtime size in the target build. Old is{' '}
            &ldquo;{ABSENT}&rdquo; because the row did not exist in the base build.
          </p>
          {additions === null ? (
            <p className={styles['status']}>Reading the added rows…</p>
          ) : (
            <>
              <Contributors
                noun="sections"
                rows={additions.sections}
                unit={unit}
                empty="No section was added."
                onPick={onPickSection}
              />
              <Contributors
                noun="symbols"
                rows={additions.symbols}
                unit={unit}
                empty="No symbol was added."
                onPick={onPickSymbol}
              />
            </>
          )}
        </div>
      </div>
    </section>
  );
}

function Contributors({
  noun,
  rows,
  unit,
  empty,
  onPick,
}: {
  readonly noun: string;
  readonly rows: readonly ContributorDto[];
  readonly unit: SizeUnit;
  readonly empty: string;
  readonly onPick: (key: string) => void;
}) {
  return (
    <div className={styles['contribGroup']}>
      <span className={styles['contribNoun']}>{noun}</span>
      {rows.length === 0 ? (
        <p className={styles['status']}>{empty}</p>
      ) : (
        <ol className={styles['contribList']}>
          {rows.map((row, index) => (
            <li key={`${row.key}:${String(index)}`}>
              <button
                type="button"
                className={styles['pick']}
                aria-label={`Show ${noun} changes for ${row.key}`}
                onClick={() => {
                  onPick(row.key);
                }}
              >
                <span className={styles['wrap']}>
                  {row.key}
                  <span className={styles['note']}> {row.changeKind}</span>
                </span>
                <span className={styles['pickFigure']}>
                  {/* A growth row shows its signed delta; an addition has no delta to show, so its
                      size stands in, labelled by the list it is in. */}
                  {row.delta === null ? formatSize(row.bytes, unit) : formatDelta(row.delta, unit)}
                </span>
              </button>
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}

/**
 * The full section list, filtered, ordered and paged by the shell.
 *
 * A row the diff has no number for keeps its word: `Not present` on the side the row did not exist
 * on, `Unknown` and the reason where a number was simply never recorded (prompt §23).
 */
function SectionChanges({
  diffId,
  unit,
  focus,
}: {
  readonly diffId: string;
  readonly unit: SizeUnit;
  readonly focus: Focus | null;
}) {
  const [offset, setOffset] = useState(0);
  const [filter, setFilter] = useState<string | null>(null);
  const [draft, setDraft] = useState('');
  const [kind, setKind] = useState<ChangeKindFilterDto | null>(null);
  const [sort, setSort] = useState<SectionChangeSortDto>('key');
  const [direction, setDirection] = useState<SortDirDto>('asc');
  const [page, setPage] = useState<SectionChangePageDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [loading, setLoading] = useState(false);
  const request = useRef(0);
  const area = useRef<HTMLElement | null>(null);
  const [seenDiffId, setSeenDiffId] = useState(diffId);

  // A new comparison must not inherit the previous pair's filter. React re-renders before it
  // commits, so no request goes out with the old name still in the box.
  if (seenDiffId !== diffId) {
    setSeenDiffId(diffId);
    setFilter(null);
    setDraft('');
    setKind(null);
    setOffset(0);
  }

  // A drill-down names the row the reader wants and moves the keyboard here; the filter travels to
  // the shell like any other, so the row is filtered from the whole table, not from one page.
  useEffect(() => {
    if (focus === null) {
      return;
    }
    setFilter(focus.key);
    setDraft(focus.key);
    setKind(null);
    setOffset(0);
    area.current?.focus();
  }, [focus]);

  useEffect(() => {
    const id = ++request.current;
    setLoading(true);
    setPage(null);
    void querySectionChanges({
      diffId,
      filter,
      changeKind: kind,
      sort,
      direction,
      offset,
      limit: null,
    }).then((outcome) => {
      if (request.current !== id) {
        return;
      }
      setLoading(false);
      if (outcome.ok) {
        setPage(outcome.value);
        setError(null);
      } else {
        setError(outcome.envelope);
      }
    });
  }, [diffId, filter, kind, sort, direction, offset]);

  const chooseSort = useCallback(
    (column: SectionChangeSortDto) => {
      setOffset(0);
      if (sort === column) {
        setDirection(direction === 'asc' ? 'desc' : 'asc');
        return;
      }
      setSort(column);
      setDirection('asc');
    },
    [direction, sort],
  );

  return (
    <section className={styles['area']} ref={area} tabIndex={-1} aria-label="Section Changes">
      <h2>Section Changes</h2>
      <ChangeFilters
        noun="sections"
        kind={kind}
        draft={draft}
        onKind={
          (value) => {
            setKind(value);
            setOffset(0);
          }
        }
        onDraftChange={setDraft}
        onApply={(value) => {
          setDraft(value);
          setFilter(value.length === 0 ? null : value);
          setOffset(0);
        }}
      />
      {loading ? (
        <p className={styles['status']} role="status" aria-live="polite">
          Loading section changes…
        </p>
      ) : null}
      {error === null ? null : (
        <ErrorPanel
          envelope={error}
          label="Section change query failed"
          heading="The section changes would not load"
        />
      )}

      {page === null ? null : (
        <>
          <ScrollArea label="Section changes table">
            <table className={styles['table']}>
              <caption className={styles['caption']}>
                Every section the diff lists as changed, filtered and ordered by the shell
              </caption>
              <thead>
                <tr>
                  <Sortable
                    label="Change"
                    column="changeKind"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <Sortable
                    label="Name"
                    column="key"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <SortHeader label="Old file" sortable={false} direction={direction} />
                  <SortHeader label="New file" sortable={false} direction={direction} />
                  <Sortable
                    label="Delta file"
                    column="delta"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <SortHeader label="Old RAM" sortable={false} direction={direction} />
                  <SortHeader label="New RAM" sortable={false} direction={direction} />
                  <SortHeader label="Delta RAM" sortable={false} direction={direction} />
                  <Sortable
                    label="RAM size"
                    column="memorySize"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <Sortable
                    label="File size"
                    column="fileSize"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <SortHeader label="Also differs" sortable={false} direction={direction} />
                </tr>
              </thead>
              <tbody>
                {page.rows.map((row, index) => (
                  <tr key={`${row.key}:${String(index)}`}>
                    <td>
                      <Kind kind={row.changeKind} />
                    </td>
                    <td className={styles['wrap']}>
                      {row.key}
                      {row.nameKnown ? null : (
                        <span className={styles['note']}> no name was recorded for this row</span>
                      )}
                      {row.ambiguous ? (
                        <span className={styles['note']}> the name repeats, so this row is unpaired</span>
                      ) : null}
                    </td>
                    <td className={styles['mono']}>
                      <SideValue change={row.fileSize} side="base" kind={row.changeKind} unit={unit} />
                    </td>
                    <td className={styles['mono']}>
                      <SideValue
                        change={row.fileSize}
                        side="target"
                        kind={row.changeKind}
                        unit={unit}
                      />
                    </td>
                    <td className={styles['mono']}>
                      <DeltaFigure change={row.fileSize} unit={unit} />
                    </td>
                    <td className={styles['mono']}>
                      <SideValue
                        change={row.memorySize}
                        side="base"
                        kind={row.changeKind}
                        unit={unit}
                      />
                    </td>
                    <td className={styles['mono']}>
                      <SideValue
                        change={row.memorySize}
                        side="target"
                        kind={row.changeKind}
                        unit={unit}
                      />
                    </td>
                    <td className={styles['mono']}>
                      <DeltaFigure change={row.memorySize} unit={unit} />
                    </td>
                    <td className={styles['mono']}>
                      {sizeText(row.memorySize, unit)}
                      {' / '}
                      {sizeText(row.fileSize, unit)}
                    </td>
                    <td>
                      <Differs row={row} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </ScrollArea>

          <Pager
            total={page.total}
            offset={page.offset}
            shown={page.rows.length}
            nextOffset={page.nextOffset}
            label="sections"
            onOffset={setOffset}
          />
        </>
      )}
    </section>
  );
}

/** The full symbol list, with the same filters and the fields a symbol row carries. */
function SymbolChanges({
  diffId,
  unit,
  focus,
}: {
  readonly diffId: string;
  readonly unit: SizeUnit;
  readonly focus: Focus | null;
}) {
  const [offset, setOffset] = useState(0);
  const [filter, setFilter] = useState<string | null>(null);
  const [draft, setDraft] = useState('');
  const [kind, setKind] = useState<ChangeKindFilterDto | null>(null);
  const [sort, setSort] = useState<SymbolChangeSortDto>('name');
  const [direction, setDirection] = useState<SortDirDto>('asc');
  const [page, setPage] = useState<SymbolChangePageDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [loading, setLoading] = useState(false);
  const request = useRef(0);
  const area = useRef<HTMLElement | null>(null);
  const [seenDiffId, setSeenDiffId] = useState(diffId);

  // Same rule as the sections table: the pair changed, so the reading of it starts clean.
  if (seenDiffId !== diffId) {
    setSeenDiffId(diffId);
    setFilter(null);
    setDraft('');
    setKind(null);
    setOffset(0);
  }

  useEffect(() => {
    if (focus === null) {
      return;
    }
    setFilter(focus.key);
    setDraft(focus.key);
    setKind(null);
    setOffset(0);
    area.current?.focus();
  }, [focus]);

  useEffect(() => {
    const id = ++request.current;
    setLoading(true);
    setPage(null);
    void querySymbolChanges({
      diffId,
      filter,
      changeKind: kind,
      sort,
      direction,
      offset,
      limit: null,
    }).then((outcome) => {
      if (request.current !== id) {
        return;
      }
      setLoading(false);
      if (outcome.ok) {
        setPage(outcome.value);
        setError(null);
      } else {
        setError(outcome.envelope);
      }
    });
  }, [diffId, filter, kind, sort, direction, offset]);

  const chooseSort = useCallback(
    (column: SymbolChangeSortDto) => {
      setOffset(0);
      if (sort === column) {
        setDirection(direction === 'asc' ? 'desc' : 'asc');
        return;
      }
      setSort(column);
      setDirection('asc');
    },
    [direction, sort],
  );

  return (
    <section className={styles['area']} ref={area} tabIndex={-1} aria-label="Symbol Changes">
      <h2>Symbol Changes</h2>
      <ChangeFilters
        noun="symbols"
        kind={kind}
        draft={draft}
        onKind={
          (value) => {
            setKind(value);
            setOffset(0);
          }
        }
        onDraftChange={setDraft}
        onApply={(value) => {
          setDraft(value);
          setFilter(value.length === 0 ? null : value);
          setOffset(0);
        }}
      />
      {loading ? (
        <p className={styles['status']} role="status" aria-live="polite">
          Loading symbol changes…
        </p>
      ) : null}
      {error === null ? null : (
        <ErrorPanel
          envelope={error}
          label="Symbol change query failed"
          heading="The symbol changes would not load"
        />
      )}

      {page === null ? null : (
        <>
          <ScrollArea label="Symbol changes table">
            <table className={styles['table']}>
              <caption className={styles['caption']}>
                Every symbol the diff lists as changed, filtered and ordered by the shell
              </caption>
              <thead>
                <tr>
                  <Sortable
                    label="Change"
                    column="changeKind"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <Sortable
                    label="Name"
                    column="name"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <SortHeader label="Kind / binding" sortable={false} direction={direction} />
                  <SortHeader label="Old address" sortable={false} direction={direction} />
                  <SortHeader label="New address" sortable={false} direction={direction} />
                  <SortHeader label="Old size" sortable={false} direction={direction} />
                  <SortHeader label="New size" sortable={false} direction={direction} />
                  <Sortable
                    label="Delta size"
                    column="sizeDelta"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <Sortable
                    label="Size"
                    column="size"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <SortHeader label="Section" sortable={false} direction={direction} />
                  <SortHeader label="Also differs" sortable={false} direction={direction} />
                </tr>
              </thead>
              <tbody>
                {page.rows.map((row, index) => (
                  <tr key={`${row.name}:${String(index)}`}>
                    <td>
                      <Kind kind={row.changeKind} />
                    </td>
                    <td className={styles['wrap']}>
                      {row.name}
                      {row.ambiguous ? (
                        <span className={styles['note']}>
                          {' '}
                          this name repeats, so the rows were left unpaired
                        </span>
                      ) : null}
                    </td>
                    <td>
                      {row.kind} / {row.binding}
                    </td>
                    <td className={styles['mono']}>{sideAddress(row, 'base')}</td>
                    <td className={styles['mono']}>{sideAddress(row, 'target')}</td>
                    <td className={styles['mono']}>
                      <SideValue change={row.size} side="base" kind={row.changeKind} unit={unit} />
                    </td>
                    <td className={styles['mono']}>
                      <SideValue change={row.size} side="target" kind={row.changeKind} unit={unit} />
                    </td>
                    <td className={styles['mono']}>
                      <DeltaFigure change={row.size} unit={unit} />
                    </td>
                    <td className={styles['mono']}>{sizeText(row.size, unit)}</td>
                    <td className={styles['mono']}>
                      {row.base === null ? ABSENT : row.base.sectionRef}
                      {' → '}
                      {row.target === null ? ABSENT : row.target.sectionRef}
                    </td>
                    <td>
                      <Differs row={row} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </ScrollArea>

          <Pager
            total={page.total}
            offset={page.offset}
            shown={page.rows.length}
            nextOffset={page.nextOffset}
            label="symbols"
            onOffset={setOffset}
          />
        </>
      )}
    </section>
  );
}

/**
 * The two export actions.
 *
 * The save path is chosen in a dialog the Rust side opens, so this page learns only the file name
 * and what happened. Declining to pick a folder, and declining to overwrite a file, are reported as
 * the normal decisions they are (prompt §35, §36).
 */
function Exports({
  note,
  error,
  exporting,
  onExport,
}: {
  readonly note: string | null;
  readonly error: ErrorEnvelopeDto | null;
  readonly exporting: boolean;
  readonly onExport: (format: 'json' | 'html') => void;
}) {
  return (
    <section className={styles['block']} aria-label="Export">
      <h2>Export</h2>
      <p className={styles['hint']}>
        Both formats carry the same facts as this screen: JSON is the portable diff document, HTML is
        one self-contained file. Neither carries a timestamp, a host path or a script.
      </p>
      <div className={styles['exports']}>
        <Button
          disabled={exporting}
          onClick={() => {
            onExport('json');
          }}
        >
          Export JSON
        </Button>
        <Button
          disabled={exporting}
          onClick={() => {
            onExport('html');
          }}
        >
          Export HTML
        </Button>
        {exporting ? (
          <p className={styles['status']} role="status" aria-live="polite">
            Waiting for the save dialog…
          </p>
        ) : null}
        {note === null ? null : (
          <p className={styles['status']} role="status" aria-live="polite">
            {note}
          </p>
        )}
      </div>
      {error === null ? null : (
        <ErrorPanel envelope={error} label="Export error" heading="The export failed" />
      )}
    </section>
  );
}

function exportWords(outcome: ExportOutcomeDto): string {
  switch (outcome.status) {
    case 'written':
      return `Wrote ${outcome.fileName ?? 'the export'} (${outcome.format.toUpperCase()}).`;
    case 'cancelled':
      return 'Export cancelled. No file was written.';
    case 'kept-existing':
      return 'Kept the file that was already there. Nothing was written.';
    default:
      // An outcome this build does not know is reported as what it is, not as a guess.
      return `The export reported an outcome this build does not know: ${outcome.status}`;
  }
}

/** A column header that orders by one of this table's sort keys. */
function Sortable<S extends string>({
  label,
  column,
  sort,
  direction,
  onSort,
}: {
  readonly label: string;
  readonly column: S;
  readonly sort: S;
  readonly direction: SortDirDto;
  readonly onSort: (column: S) => void;
}) {
  return (
    <SortHeader
      label={label}
      direction={direction}
      active={sort === column}
      onSort={() => {
        onSort(column);
      }}
    />
  );
}

function ChangeFilters({
  noun,
  kind,
  draft,
  onKind,
  onDraftChange,
  onApply,
}: {
  readonly noun: string;
  readonly kind: ChangeKindFilterDto | null;
  readonly draft: string;
  readonly onKind: (kind: ChangeKindFilterDto | null) => void;
  readonly onDraftChange: (value: string) => void;
  readonly onApply: (value: string) => void;
}) {
  const inputId = `fs-change-filter-${noun}`;

  return (
    <form
      className={styles['filters']}
      aria-label={`Filter ${noun}`}
      onSubmit={(event) => {
        event.preventDefault();
        onApply(draft);
      }}
    >
      <span className={styles['radioGroup']} role="group" aria-label={`Filter ${noun} by change kind`}>
        {KIND_FILTERS.map((entry) => (
          <span key={entry.label} className={styles['radio']}>
            <input
              id={`fs-change-${noun}-${entry.label}`}
              type="radio"
              name={`fs-change-${noun}`}
              checked={kind === entry.value}
              onChange={() => {
                onKind(entry.value);
              }}
            />
            <label htmlFor={`fs-change-${noun}-${entry.label}`}>{entry.label}</label>
          </span>
        ))}
      </span>
      <label className={styles['filterLabel']} htmlFor={inputId}>
        Filter {noun} by name
      </label>
      <input
        id={inputId}
        className={styles['filterInput']}
        type="text"
        autoComplete="off"
        value={draft}
        onChange={(event) => {
          onDraftChange(event.target.value);
        }}
      />
      <Button type="submit">Apply filter</Button>
    </form>
  );
}

/**
 * One side of a change row.
 *
 * `Not present` is reserved for the side the diff says the row did not exist on. Every other empty
 * cell is a number that was never recorded, which is Unknown and not absence (prompt §23).
 */
function SideValue({
  change,
  side,
  kind,
  unit,
}: {
  readonly change: ByteDeltaDto;
  readonly side: Side;
  readonly kind: string;
  readonly unit: SizeUnit;
}) {
  const value = side === 'base' ? change.base : change.target;
  if (value !== null) {
    return <span className={styles['mono']}>{formatSize(value, unit)}</span>;
  }
  if (isAbsentSide(kind, side)) {
    return <span className={styles['absent']}>{ABSENT}</span>;
  }
  return (
    <span className={styles['unknown']}>
      Unknown
      {change.reason === null ? null : <span className={styles['note']}> {change.reason}</span>}
    </span>
  );
}

/** The row's own size, whichever side carries it: the quantity the size columns order by. */
function sizeText(change: ByteDeltaDto, unit: SizeUnit): string {
  return formatSize(change.target ?? change.base, unit);
}

function sideAddress(row: SymbolChangeRowDto, side: Side): string {
  const record = side === 'base' ? row.base : row.target;
  if (record !== null) {
    return record.address ?? 'Unknown';
  }
  return isAbsentSide(row.changeKind, side) ? ABSENT : 'Unknown';
}

/** Only an unpaired row is absent on one side; a Changed row with no number is unknown, not absent. */
function isAbsentSide(kind: string, side: Side): boolean {
  return (kind === 'Added' && side === 'base') || (kind === 'Removed' && side === 'target');
}

/** What else the diff saw, and what it refused to call a change. */
function Differs({ row }: { readonly row: SectionChangeRowDto | SymbolChangeRowDto }) {
  return (
    <>
      {row.differingFields.length === 0 ? null : (
        <span className={styles['monoSmall']}>{row.differingFields.join(', ')}</span>
      )}
      {row.indeterminateFields.length === 0 ? null : (
        <span className={styles['note']}>
          {row.differingFields.length === 0 ? '' : ' '}
          not decidable: {row.indeterminateFields.join(', ')}
        </span>
      )}
    </>
  );
}

function Comparability({ value }: { readonly value: string }) {
  const state: StateName = value === 'exact' ? 'PASS' : value === 'partial' ? 'REVIEW' : 'UNKNOWN';
  return <StateBadge state={state} label={value} />;
}

function sectionAddition(row: SectionChangeRowDto): ContributorDto {
  return {
    key: row.key,
    changeKind: row.changeKind,
    delta: null,
    bytes: row.memorySize.target ?? row.memorySize.base,
  };
}

function symbolAddition(row: SymbolChangeRowDto): ContributorDto {
  return {
    key: row.name,
    changeKind: row.changeKind,
    delta: null,
    bytes: row.size.target ?? row.size.base,
  };
}
