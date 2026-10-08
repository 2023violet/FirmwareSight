/**
 * U1's Overview page, and the shared primitives it is built from.
 *
 * The page is new, so these are its first tests. They are written around the one thing that would make
 * an Overview dishonest - a summary that states more than the facts it was given - rather than around
 * pixel counts: what a session with nothing analyzed must say, what a verdict must not claim, and which
 * numbers must appear as Unknown rather than as zero.
 */

import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Overview, type OverviewTarget } from './Overview';
import { TopBar } from './components/TopBar';
import type {
  AnalysisSummaryDto,
  GateFindingRowDto,
  GateRunDto,
  ProjectContextDto,
  ProjectPolicyDto,
  SelectionDto,
} from './ipc/types';
import { precedes } from './test/order';

/**
 * Two full snapshot ids, in the grammar `build_snapshot.rs:35` produces: `snap-<artifact sha256>-<normalization>`.
 * They are long on purpose. A guard that only works because the fixture ids are short would be a guard that
 * truncation is allowed to break, and truncation is exactly what §5.A has to survive.
 */
const SNAP_A = `snap-${'a1b2c3d4'.repeat(8)}-norm-3`;
const SNAP_B = `snap-${'e5f60718'.repeat(8)}-norm-3`;

function summary(over: Partial<AnalysisSummaryDto> = {}): AnalysisSummaryDto {
  return {
    source: 'artifact',
    artifact: {
      fileName: 'relay.elf',
      kind: 'elf',
      sha256: 'a3f9c1e2b4d5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f',
      byteSize: 497_840,
      parserId: 'elf-header-v1',
      architecture: 'arm',
      bitness: '32',
      endianness: 'little',
      entryPoint: '0x0800_0148',
      entryPointUnknownReason: null,
      buildId: null,
      buildIdUnknownReason: 'no .note.gnu.build-id section',
    },
    identity: {
      schema: 'urn:firmwaresight:schema:analysis:1',
      schemaStability: 'stable',
      snapshotId: 'snap-7f3a-2',
      normalizationVersion: 'norm-3',
      createdByFwsightVersion: '0.6.0',
    },
    memory: {
      accountingRule: 'adr-0021-load-and-flags',
      layoutSource: 'firmware.map',
      weakestEvidenceBasis: 'map.region-and-elf-load',
      admissibleForHardBlock: true,
      nonvolatileImageFootprint: {
        state: 'exact',
        classification: 'observed',
        bytes: 497_840,
        unattributed: [],
        reason: null,
      },
      runtimeRamFootprint: {
        state: 'partial',
        classification: 'derived',
        bytes: 65_536,
        unattributed: ['.bss.nocache'],
        reason: 'one symbol has no size in the MAP',
      },
      dualAccountedSections: ['.data'],
      excludedMetadataBytes: 14_690,
    },
    sectionCount: 9,
    symbolCount: 1_284,
    capabilities: {
      elf: 'supported',
      sections: 'supported',
      symbols: 'provided',
      debugInfo: 'partial',
      map: 'provided',
      objectAttribution: 'partial',
      git: 'available',
    },
    evidenceSummary: { total: 212, observed: 180, derived: 24, declared: 4, unknown: 4 },
    ...over,
  };
}

function finding(over: Partial<GateFindingRowDto> = {}): GateFindingRowDto {
  return {
    id: 'f-1',
    ruleId: 'signed_image.required',
    state: 'BLOCK',
    effectiveSeverity: 'BLOCK',
    summary: 'Signature header not found at offset 0x0200.',
    evidenceRefs: ['ev-91'],
    remediation: 'Run the signing step, then re-run the gate.',
    acceptable: false,
    acceptance: null,
    ...over,
  };
}

function gateRun(over: Partial<GateRunDto> = {}): GateRunDto {
  return {
    runId: 'run-12',
    snapshotId: 'snap-7f3a-2',
    baselineSnapshotId: 'snap-5c1a-1',
    projectName: 'relay-controller',
    policySource: 'firmwaresight.toml',
    policySha256: 'b4d5'.repeat(16),
    createdAt: '2026-10-06T09:14:22Z',
    overallEffectiveSeverity: 'BLOCK',
    dispositionEffectiveSeverity: 'BLOCK',
    counts: { pass: 4, review: 1, block: 1, unknown: 1, notApplicable: 1 },
    findings: [finding()],
    warnings: [],
    git: {
      available: true,
      headCommit: '9f3e21d',
      exactTag: null,
      dirty: false,
      summary: 'workspace at 9f3e21d, clean',
      reason: null,
    },
    policy: null,
    tables: null,
    recordNote: null,
    ...over,
  };
}

/**
 * A run whose own answer is PASS, judged against `snapshotId`. Used wherever a test needs to prove a green
 * verdict did *not* reach a build it never judged: a BLOCK fixture would pass that assertion for the wrong
 * reason.
 */
function passRun(snapshotId: string): GateRunDto {
  return gateRun({
    snapshotId,
    overallEffectiveSeverity: 'PASS',
    dispositionEffectiveSeverity: 'PASS',
    counts: { pass: 8, review: 0, block: 0, unknown: 0, notApplicable: 2 },
    findings: [
      finding({
        id: 'f-git',
        ruleId: 'git.clean',
        state: 'PASS',
        effectiveSeverity: 'PASS',
        summary: 'The workspace has no uncommitted changes.',
      }),
    ],
  });
}

function policy(over: Partial<ProjectPolicyDto> = {}): ProjectPolicyDto {
  const base: ProjectPolicyDto = {
    projectName: 'relay-controller',
    requiredArtifactKinds: null,
    flashBudget: null,
    ramBudget: null,
    requireCleanGit: null,
    requireReleaseNotes: null,
    releaseNotesPath: null,
    versionSource: null,
    versionPattern: null,
    expectedVersion: null,
    expectedCommit: null,
    flashGrowthReviewBytes: null,
    ramGrowthReviewBytes: null,
    unknownEvidenceReviewCount: null,
    onUnknown: {
      gitClean: 'review',
      commitMatchesRelease: 'review',
      versionMatch: 'review',
      flashBudget: 'block',
      ramBudget: 'block',
      baselineGrowth: 'review',
      releaseNotes: 'review',
    },
  };
  return { ...base, ...over };
}

function project(over: Partial<ProjectContextDto> = {}): ProjectContextDto {
  const base: ProjectContextDto = {
    projectName: 'relay-controller',
    configFileName: 'firmwaresight.toml',
    configSchemaVersion: 1,
    policy: policy(),
    policySha256: 'b4d5'.repeat(16),
    warnings: [],
    unknownKeys: [],
  };
  return { ...base, ...over };
}

function selection(over: Partial<SelectionDto> = {}): SelectionDto {
  const base: SelectionDto = {
    selectionId: 'sel-1',
    fileName: 'relay.elf',
    mapFileName: 'relay.map',
    mapAttached: true,
  };
  return { ...base, ...over };
}

/** The `App`-shaped props, with the defaults `App` itself starts from. */
function renderOverview(over: {
  readonly summary?: AnalysisSummaryDto | null;
  readonly gateRun?: GateRunDto | null;
  readonly selection?: SelectionDto | null;
  readonly analyzedSelectionId?: string | null;
  readonly project?: ProjectContextDto | null;
} = {}) {
  const onOpen = vi.fn<(target: OverviewTarget) => void>();
  render(
    <Overview
      summary={over.summary ?? null}
      selection={over.selection ?? null}
      analyzedSelectionId={over.analyzedSelectionId ?? null}
      gateRun={over.gateRun ?? null}
      project={over.project ?? null}
      unit="bytes"
      onOpen={onOpen}
    />,
  );
  return onOpen;
}

describe('Overview before there is anything to overview', () => {
  it('says the session has produced nothing, and invents no numbers to fill the space', () => {
    renderOverview();

    expect(screen.getByText(/Nothing has been analyzed in this session yet/)).toBeDefined();
    expect(screen.queryByText('Flash footprint')).toBeNull();
    expect(screen.queryByText('497,840')).toBeNull();
    // A verdict with no run behind it is the one thing this page must never render.
    expect(screen.getByText(/has not run the gate in this session/)).toBeDefined();
    expect(screen.queryByText(/Blocked —/)).toBeNull();
    expect(screen.queryByText(/Clear —/)).toBeNull();
  });

  it('offers the next step as navigation, and takes no action of its own', () => {
    const onOpen = renderOverview();

    fireEvent.click(screen.getByRole('button', { name: 'Choose an artifact' }));
    expect(onOpen).toHaveBeenCalledWith('analyze');
    fireEvent.click(screen.getByRole('button', { name: 'Open Release Gate' }));
    expect(onOpen).toHaveBeenCalledWith('release');
  });
});

describe('Overview with a real analysis and a real run', () => {
  it('names each input capability with a word and a glyph, never colour alone', () => {
    const { container } = render(
      <Overview
        summary={summary()}
        selection={null}
        analyzedSelectionId={null}
        gateRun={null}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    const inputs = screen.getByRole('group', { name: 'Input capabilities' });
    // Each word is asserted inside the panel it belongs to: one concatenated text check would pass even
    // if a panel showed another panel's capability, which is the mix-up that matters.
    for (const { term, word } of [
      { term: 'ELF', word: 'supported' },
      { term: 'MAP', word: 'provided' },
      { term: 'Git', word: 'available' },
    ]) {
      const card = within(inputs).getByRole('region', { name: term });
      expect(card.textContent).toContain(word);
    }
    // Three pills, and at least one glyph per pill: the words are not carried by colour.
    expect(container.querySelectorAll('svg').length).toBeGreaterThanOrEqual(3);
  });

  it("states the verdict from the run's own counts", () => {
    renderOverview({ summary: summary(), gateRun: gateRun() });

    expect(screen.getByText(/Blocked — 1 rule\(s\) failed/)).toBeDefined();
    expect(screen.getByText(/A blocked build cannot produce a bundle/)).toBeDefined();
    // The strip is named by its accessible name, not by a label element: it is a graphic whose whole
    // sentence is the count line.
    expect(
      screen.getByRole('img', { name: /Findings by state in this run/ }),
    ).toBeDefined();
  });

  it('keeps the limit of the claim inside the card that makes it', () => {
    renderOverview({ summary: summary(), gateRun: gateRun() });

    // DESIGN.md 9: "Can we ship now?" is policy readiness, never a legal, security or compliance
    // conclusion. The sentence has to be findable in the same region as the verdict.
    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    expect(ship.textContent).toContain('not a legal, security or product-compliance conclusion');
  });

  it('says so when accepted reviews moved the aggregate, and names the aggregate without them', () => {
    renderOverview({
      summary: summary(),
      gateRun: gateRun({
        dispositionEffectiveSeverity: 'REVIEW',
        overallEffectiveSeverity: 'BLOCK',
        findings: [
          finding({
            id: 'f-2',
            ruleId: 'flash_budget.soft',
            state: 'REVIEW',
            effectiveSeverity: 'REVIEW',
            acceptable: true,
            acceptance: {
              findingId: 'f-2',
              actor: 'engineer',
              reason: 'expected growth',
              recordedAt: '2026-10-06T09:20:04Z',
            } as unknown as GateFindingRowDto['acceptance'],
          }),
        ],
      }),
    });

    expect(screen.getByText(/accepted review\(s\) counted/)).toBeDefined();
    expect(screen.getByText(/read BLOCK/)).toBeDefined();
  });

  it('shows Unknown for a footprint with no number, never 0', () => {
    renderOverview({
      summary: summary({
        memory: {
          ...summary().memory,
          nonvolatileImageFootprint: {
            state: 'unknown',
            classification: 'unknown',
            bytes: null,
            unattributed: [],
            reason: 'no allocatable section carried an address and flag pair',
          },
        },
      }),
      gateRun: null,
    });

    const flash = screen.getByText('Flash footprint').parentElement;
    expect(flash?.textContent).toContain('Unknown');
    expect(flash?.textContent).not.toMatch(/\b0 bytes\b/);
  });

  it('lists only the findings that are open, and says where the rest live', () => {
    renderOverview({
      summary: summary(),
      gateRun: gateRun({
        findings: [
          finding(),
          finding({ id: 'f-2', ruleId: 'flash_budget.soft', state: 'REVIEW' }),
          finding({ id: 'f-3', ruleId: 'entry_point.valid', state: 'PASS' }),
          finding({ id: 'f-4', ruleId: 'sbom.present', state: 'N/A' }),
        ],
      }),
    });

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    expect(ship.textContent).toContain('signed_image.required');
    expect(ship.textContent).toContain('flash_budget.soft');
    // A passing rule and a rule that does not apply are not open findings.
    expect(ship.textContent).not.toContain('entry_point.valid');
    expect(ship.textContent).not.toContain('sbom.present');
  });
});

describe('U1P Overview hierarchy', () => {
  it('reads inputs, then the verdict, then the figures that back it', () => {
    renderOverview({ summary: summary(), gateRun: gateRun() });

    const inputs = screen.getByRole('group', { name: 'Input capabilities' });
    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    const figures = screen.getByRole('group', { name: 'Key figures' });

    // §8: the verdict is the page's one dominant element, so it sits between what went in and the
    // numbers that support it. A KPI strip above the sentence would be a figure answering nothing.
    expect(precedes(inputs, ship)).toBe(true);
    expect(precedes(ship, figures)).toBe(true);

    // The capability band is three separate answers, not one merged string, and the figures band holds
    // the count words the same page used to scatter.
    expect(
      within(inputs)
        .getAllByRole('region')
        .map((cell) => cell.getAttribute('aria-label')),
    ).toEqual(['ELF', 'MAP', 'Git']);
    expect(
      Array.from(figures.querySelectorAll('[class*="summaryLabel"]')).map((cell) => cell.textContent),
    ).toEqual(['Flash footprint', 'Runtime RAM', 'Symbols', 'Evidence']);
  });
});

describe('the product bar', () => {
  it('states the local-first promise on every page, with or without a project', () => {
    const { unmount } = render(<TopBar project={null} />);
    expect(screen.getByText(/nothing leaves this machine/)).toBeDefined();
    expect(screen.queryByText('relay-controller')).toBeNull();
    unmount();

    render(<TopBar project={'relay-controller'} />);
    expect(screen.getByText('relay-controller')).toBeDefined();
    expect(screen.getByText(/nothing leaves this machine/)).toBeDefined();
  });
});

/**
 * U1P-R1 red-before-green guards.
 *
 * These five run against `a2e6b30`'s unmodified `Overview.tsx` first, and they must fail there: the page has no
 * subject guard, so a run made for one build is rendered as the shipping answer about another. They are written
 * on the props the component already accepts (`summary`, `selection`, `gateRun`, `project`) so the defect is
 * proved in the product and not manufactured by a new prop.
 */
describe('U1P-R1 the run in hand must be about the build on screen', () => {
  it('T2 does not answer for build B with build A PASS', () => {
    render(
      <Overview
        summary={summary({ identity: { ...summary().identity, snapshotId: SNAP_B } })}
        selection={selection({ selectionId: 'sel-2' })}
        analyzedSelectionId="sel-2"
        gateRun={passRun(SNAP_A)}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    // §5.H in its strongest form: the clear-build sentence must not appear at all, not merely appear somewhere
    // else on the page, while the page is describing a build no run has judged.
    expect(screen.queryByText(/Clear —/)).toBeNull();
    expect(ship.textContent).toMatch(/Not assessed for this build/);
    expect(ship.textContent).toContain(SNAP_A);
    expect(ship.textContent).toContain(SNAP_B);
  });

  it('T3 does not answer for build B with build A BLOCK either', () => {
    render(
      <Overview
        summary={summary({ identity: { ...summary().identity, snapshotId: SNAP_B } })}
        selection={selection({ selectionId: 'sel-2' })}
        analyzedSelectionId="sel-2"
        gateRun={gateRun({ snapshotId: SNAP_A })}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    // The mismatch is stated, and stated about the right things.
    expect(ship.textContent).toMatch(/Not assessed for this build/);
    const retained = within(ship).getByRole('group', { name: /Retained run/ });
    expect(retained.textContent).toContain(SNAP_A);
    expect(retained.textContent).toContain(SNAP_B);
    // A BLOCK belongs to A, so no BLOCK sentence may be read as this build's answer - and the aggregate A
    // really has is not restated here at all, which is the shape §5.H is safest in.
    expect(screen.queryByText(/Blocked —/)).toBeNull();
    expect(within(ship).queryByText(/BLOCK/)).toBeNull();
  });

  it('T6 does not call a stored run the answer about a session that analyzed nothing', () => {
    render(
      <Overview
        summary={null}
        selection={null}
        analyzedSelectionId={null}
        gateRun={gateRun({ snapshotId: SNAP_A })}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    expect(ship.textContent).toMatch(/No analysis in this session/);
    expect(ship.textContent).toContain(SNAP_A);
    expect(screen.queryByText(/Blocked —/)).toBeNull();
  });

  it('T10 does not call the old run an answer about the policy loaded now', () => {
    render(
      <Overview
        summary={summary({ identity: { ...summary().identity, snapshotId: SNAP_A } })}
        selection={selection({ selectionId: 'sel-1' })}
        analyzedSelectionId="sel-1"
        gateRun={gateRun({ snapshotId: SNAP_A, policySha256: 'b4d5'.repeat(16) })}
        project={project({ policySha256: 'c9aa'.repeat(16) })}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    expect(ship.textContent).toMatch(/Not assessed under the policy loaded now/);
    expect(ship.textContent).toContain('b4d5'.repeat(16));
    expect(ship.textContent).toContain('c9aa'.repeat(16));
    expect(screen.queryByText(/Clear —/)).toBeNull();
  });

  it('T11 reads the judged target, never the baseline', () => {
    render(
      <Overview
        // The displayed analysis is the run's *baseline*, which is the one identity that must not satisfy the guard.
        summary={summary({ identity: { ...summary().identity, snapshotId: SNAP_A } })}
        selection={selection({ selectionId: 'sel-1' })}
        analyzedSelectionId="sel-1"
        gateRun={gateRun({ snapshotId: SNAP_B, baselineSnapshotId: SNAP_A })}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    expect(ship.textContent).toMatch(/Not assessed for this build/);
    expect(screen.queryByText(/Blocked —/)).toBeNull();
  });
});

/**
 * U1P-R1 the states the corrective must leave alone, and the way back to a real answer.
 *
 * Half of these are the "do not weaken what was accepted" guards: a subject guard that also quietly changed a
 * verdict sentence, dropped the counts, or cost the page its figures would be a redesign wearing a fix.
 */
describe('U1P-R1 a matched subject still gets its own answer', () => {
  it('T1 names the judged snapshot in full inside the verdict region', () => {
    render(
      <Overview
        summary={summary({ identity: { ...summary().identity, snapshotId: SNAP_A } })}
        selection={selection({ selectionId: 'sel-1' })}
        analyzedSelectionId="sel-1"
        gateRun={passRun(SNAP_A)}
        project={project({ policySha256: 'b4d5'.repeat(16) })}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    const answer = within(ship).getByRole('group', { name: 'Readiness for the build this page describes' });
    expect(within(answer).getByText(/Clear —/)).toBeDefined();
    // The full identity, not the eight-character form the header uses.
    expect(answer.textContent).toContain(SNAP_A);
    expect(answer.textContent).not.toMatch(/\.\.\./);
    expect(within(answer).getByRole('img', { name: /Findings by state in this run/ })).toBeDefined();
    expect(ship.textContent).toContain('run-12');
    expect(ship.textContent).toContain('2026-10-06T09:14:22Z');
  });

  it('T4 leaves REVIEW with an accepted review, and UNKNOWN, exactly as Core aggregated them', () => {
    const view = render(
      <Overview
        summary={summary({ identity: { ...summary().identity, snapshotId: SNAP_A } })}
        selection={selection()}
        analyzedSelectionId="sel-1"
        gateRun={gateRun({
          snapshotId: SNAP_A,
          overallEffectiveSeverity: 'BLOCK',
          dispositionEffectiveSeverity: 'REVIEW',
          counts: { pass: 4, review: 1, block: 1, unknown: 0, notApplicable: 1 },
          findings: [
            finding({
              id: 'f-2',
              ruleId: 'flash_budget.soft',
              state: 'REVIEW',
              effectiveSeverity: 'REVIEW',
              acceptable: true,
              acceptance: {
                findingId: 'f-2',
                actor: 'engineer',
                reason: 'expected growth',
                recordedAt: '2026-10-06T09:20:04Z',
              } as unknown as GateFindingRowDto['acceptance'],
            }),
          ],
        })}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    const answer = screen.getByRole('group', { name: 'Readiness for the build this page describes' });
    expect(answer.textContent).toMatch(/Needs a decision/);
    expect(answer.textContent).toMatch(/accepted review\(s\) counted/);
    expect(answer.textContent).toMatch(/read BLOCK/);

    view.rerender(
      <Overview
        summary={summary({ identity: { ...summary().identity, snapshotId: SNAP_A } })}
        selection={selection()}
        analyzedSelectionId="sel-1"
        gateRun={gateRun({
          snapshotId: SNAP_A,
          overallEffectiveSeverity: 'UNKNOWN',
          dispositionEffectiveSeverity: 'UNKNOWN',
          counts: { pass: 3, review: 0, block: 0, unknown: 2, notApplicable: 1 },
          findings: [],
        })}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    const unknownAnswer = screen.getByRole('group', { name: 'Readiness for the build this page describes' });
    expect(unknownAnswer.textContent).toMatch(/Not evaluated — 2 finding\(s\)/);
    // The counts are the run's, echoed as it reported them. This page does not recount them.
    expect(
      within(unknownAnswer).getByRole('img', { name: /Findings by state in this run/ }).textContent,
    ).toMatch(/2/);
  });

  it('T8 brings the answer back when the run and the analysis become the same build', () => {
    const view = render(
      <Overview
        summary={summary({ identity: { ...summary().identity, snapshotId: SNAP_B } })}
        selection={selection({ selectionId: 'sel-2' })}
        analyzedSelectionId="sel-2"
        gateRun={gateRun({ snapshotId: SNAP_A })}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );
    expect(screen.getByRole('region', { name: 'Can we ship now?' }).textContent)
      .toMatch(/Not assessed for this build/);

    view.rerender(
      <Overview
        summary={summary({ identity: { ...summary().identity, snapshotId: SNAP_B } })}
        selection={selection({ selectionId: 'sel-2' })}
        analyzedSelectionId="sel-2"
        gateRun={gateRun({ snapshotId: SNAP_B })}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    const answer = within(screen.getByRole('region', { name: 'Can we ship now?' })).getByRole('group', {
      name: 'Readiness for the build this page describes',
    });
    expect(answer.textContent).toMatch(/Blocked —/);
    expect(answer.textContent).toContain(SNAP_B);
  });

  it('T9 speaks for the newest run and stops speaking for the one it replaced', () => {
    const view = render(
      <Overview
        summary={summary({ identity: { ...summary().identity, snapshotId: SNAP_A } })}
        selection={selection()}
        analyzedSelectionId="sel-1"
        gateRun={gateRun({ snapshotId: SNAP_A })}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );
    expect(screen.getByRole('region', { name: 'Can we ship now?' }).textContent).toContain('run-12');

    view.rerender(
      <Overview
        summary={summary({ identity: { ...summary().identity, snapshotId: SNAP_B } })}
        selection={selection({ selectionId: 'sel-2' })}
        analyzedSelectionId="sel-2"
        gateRun={gateRun({ runId: 'run-99', snapshotId: SNAP_B, baselineSnapshotId: SNAP_A })}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    expect(ship.textContent).toContain('run-99');
    expect(ship.textContent).toContain(SNAP_B);
    expect(ship.textContent).not.toContain('run-12');
    // A new run's baseline is not a second subject claim: the page names the target it judged, and only that.
    expect(ship.textContent).not.toContain(SNAP_A);
  });
});

/** §5.C, §5.E, §5.G, and the parts of §14 that say "and nothing else broke". */
describe('U1P-R1 the neutral states and the page around them', () => {
  it('T5 does not let an unanalyzed selection inherit a matching run', () => {
    render(
      <Overview
        summary={summary({ identity: { ...summary().identity, snapshotId: SNAP_A } })}
        selection={selection({ selectionId: 'sel-2', fileName: 'pump.elf' })}
        analyzedSelectionId="sel-1"
        gateRun={passRun(SNAP_A)}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    // The subject ids match, so only the selection handle can stop the PASS - which is what §5.C asks for.
    expect(ship.textContent).toMatch(/Not assessed for the selected artifact/);
    expect(screen.queryByText(/Clear —/)).toBeNull();
    expect(ship.textContent).toContain(SNAP_A);
  });

  it('T7 keeps the no-run state honest, and its step still navigates', () => {
    const onOpen = renderOverview({ summary: summary(), gateRun: null });

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    expect(ship.textContent).toMatch(/has not run the gate in this session/);
    expect(screen.queryByText(/Clear —/)).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Run it on Release Gate' }));
    expect(onOpen).toHaveBeenCalledWith('release');
  });

  it('T12 keeps the inputs, the figures and the purely navigational actions intact', () => {
    const onOpen = renderOverview({
      summary: summary(),
      gateRun: passRun('snap-7f3a-2'),
      selection: selection(),
      analyzedSelectionId: 'sel-1',
    });

    const inputs = screen.getByRole('group', { name: 'Input capabilities' });
    expect(within(inputs).getByRole('region', { name: 'ELF' }).textContent).toContain('supported');
    expect(within(inputs).getByRole('region', { name: 'MAP' }).textContent).toContain('provided');
    expect(within(inputs).getByRole('region', { name: 'Git' }).textContent).toContain('available');

    const figures = screen.getByRole('group', { name: 'Key figures' });
    expect(figures.textContent).toContain('497,840');
    expect(figures.textContent).toContain('65,536');
    expect(figures.textContent).toContain('1284');
    expect(figures.textContent).toContain('212');

    // Every action here moves the reader somewhere. None of them analyzes, gates, accepts or writes.
    fireEvent.click(within(inputs).getByRole('button', { name: 'Open Analyze' }));
    fireEvent.click(screen.getByRole('button', { name: 'Compare builds' }));
    fireEvent.click(screen.getByRole('button', { name: 'Bundle & History' }));
    fireEvent.click(screen.getByRole('button', { name: 'Open Release Gate' }));
    expect(onOpen.mock.calls.map((call) => call[0])).toEqual(['analyze', 'compare', 'history', 'release']);
  });

  it('keeps the page order U1P was accepted on when the answer goes neutral', () => {
    const onOpen = renderOverview({
      summary: summary({ identity: { ...summary().identity, snapshotId: SNAP_B } }),
      gateRun: gateRun({ snapshotId: SNAP_A }),
      selection: selection({ selectionId: 'sel-2' }),
      analyzedSelectionId: 'sel-2',
    });

    const inputs = screen.getByRole('group', { name: 'Input capabilities' });
    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    const figures = screen.getByRole('group', { name: 'Key figures' });
    expect(precedes(inputs, ship)).toBe(true);
    expect(precedes(ship, figures)).toBe(true);
    fireEvent.click(screen.getByRole('button', { name: 'Choose this build on Release Gate' }));
    expect(onOpen).toHaveBeenCalledWith('release');
  });
});
