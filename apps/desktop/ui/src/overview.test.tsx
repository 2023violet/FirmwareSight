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
import type { ComponentProps } from 'react';

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

/**
 * U1P-R2 red-before-green guards for `U1P-V2-02`.
 *
 * The defect is not the Gate card, which U1P-R1 already made honest. It is the rest of the page: while a newly
 * chosen artifact waits for Analyze, `InputRow` paints the *retained* ELF/MAP/Git capabilities and takes only the
 * MAP detail line from the live selection, and `Facts` prints the retained Flash/RAM/Symbols/Evidence figures with
 * nothing near them saying they belong to a previous result. A reader who selects `firmware.elf` from another
 * directory therefore meets another build's facts under this build's name.
 *
 * R2-T1, R2-T4, R2-T8, R2-T9, R2-T10 and R2-T11 are regression locks on behaviour the Architect has already
 * accepted, so they are expected to pass before the change as well as after it; they are reported as locks, never
 * counted as proof that the fix landed. R2-T2, R2-T3, R2-T5, R2-T6, R2-T7 and R2-T12 must fail on `60f10a6`.
 */
describe('U1P-R2 a pending selection may not wear the previous build’s facts', () => {
  /** The retained analysis: an analyzed `firmware.elf` whose run judged SNAP_B. */
  function retained(): AnalysisSummaryDto {
    return summary({
      artifact: { ...summary().artifact, fileName: 'firmware.elf' },
      identity: { ...summary().identity, snapshotId: SNAP_B },
    });
  }

  /** The same shell shape `App` holds when a second file has been chosen but not yet analyzed. */
  function pendingProps(over: Partial<ComponentProps<typeof Overview>> = {}): ComponentProps<typeof Overview> {
    return {
      summary: retained(),
      selection: selection({ selectionId: 'sel-3', fileName: 'firmware.elf', mapAttached: true }),
      analyzedSelectionId: 'sel-2',
      gateRun: passRun(SNAP_B),
      project: null,
      unit: 'bytes',
      onOpen: () => undefined,
      ...over,
    };
  }

  it('R2-T1 keeps the current selection’s own capabilities, figures and verdict', () => {
    renderOverview({
      summary: summary({ identity: { ...summary().identity, snapshotId: SNAP_A } }),
      selection: selection({ selectionId: 'sel-1', fileName: 'firmware.elf' }),
      analyzedSelectionId: 'sel-1',
      gateRun: passRun(SNAP_A),
    });

    const inputs = screen.getByRole('group', { name: 'Input capabilities' });
    expect(within(inputs).getByRole('region', { name: 'ELF' }).textContent).toContain('supported');
    expect(within(inputs).getByRole('region', { name: 'MAP' }).textContent).toContain('provided');
    expect(screen.getByRole('group', { name: 'Key figures' })).toBeDefined();
    expect(screen.getByRole('region', { name: 'Can we ship now?' }).textContent).toMatch(/Clear —/);
    expect(screen.queryByRole('region', { name: 'Previous analysis' })).toBeNull();
  });

  it('R2-T2 does not offer retained chips or figures as the pending selection’s current facts', () => {
    render(
      <Overview {...pendingProps()} />,
    );

    // The current-status band belongs to a build the page no longer describes, so it is not shown as current.
    expect(screen.queryByRole('group', { name: 'Input capabilities' })).toBeNull();
    expect(screen.queryByRole('group', { name: 'Key figures' })).toBeNull();

    const current = screen.getByRole('group', { name: 'Selected artifact' });
    expect(current.textContent).toContain('firmware.elf');
    expect(current.textContent).toMatch(/not analyzed yet/i);
    // Nothing that reads like a capability verdict may sit in the current band.
    for (const word of ['supported', 'provided', 'available']) {
      expect(current.textContent).not.toContain(word);
    }

    // The retained result survives, labelled and below the decision.
    const previous = screen.getByRole('region', { name: 'Previous analysis' });
    expect(previous.textContent).toContain('497,840');
    expect(previous.textContent).toContain(SNAP_B);

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    expect(precedes(current, ship)).toBe(true);
    expect(precedes(ship, previous)).toBe(true);
  });

  it('R2-T3 keeps a MAP chip and its explanation inside one artifact’s context', () => {
    render(
      <Overview {...pendingProps({ selection: selection({ selectionId: 'sel-3', fileName: 'pump.elf', mapAttached: false, mapFileName: null }) })} />,
    );

    const current = screen.getByRole('group', { name: 'Selected artifact' });
    const mapCell = within(current).getByRole('region', { name: 'MAP' });
    expect(mapCell.textContent).toContain('No MAP attached to this selection');
    // The proven contradiction: a green "provided" pill over the sentence that says the map is missing.
    expect(mapCell.textContent).not.toContain('Symbol-level analysis depends on it');
    expect(current.textContent).not.toContain('provided');

    const previous = screen.getByRole('region', { name: 'Previous analysis' });
    expect(within(previous).getByRole('region', { name: 'MAP' }).textContent).toContain('provided');
  });

  it('R2-T4 gives the pending selection no current PASS, BLOCK or REVIEW', () => {
    render(<Overview {...pendingProps()} />);

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    expect(ship.textContent).toMatch(/Not assessed for the selected artifact/);
    expect(ship.textContent).not.toMatch(/Clear —/);
    expect(ship.textContent).not.toMatch(/Blocked —/);
  });

  it('R2-T5 makes the pending selection’s primary step an analysis, not a Gate claim', () => {
    const onOpen = vi.fn<(target: OverviewTarget) => void>();
    render(<Overview {...pendingProps({ onOpen })} />);

    const analyze = screen.getByRole('button', { name: 'Analyze selected artifact' });
    expect(analyze.className).toContain('primary');
    expect(screen.queryByRole('button', { name: 'Choose this build on Release Gate' })).toBeNull();

    fireEvent.click(analyze);
    expect(onOpen).toHaveBeenLastCalledWith('analyze');

    // The Release trip stays available and is named for what it actually opens: the stored run, not a new verdict.
    const release = screen.getByRole('button', { name: 'View previous Gate run' });
    fireEvent.click(release);
    expect(onOpen).toHaveBeenLastCalledWith('release');
    expect(screen.queryByText(/Rerun|Re-run|run the gate now/i)).toBeNull();
  });

  it('R2-T6 keeps a failed new selection from becoming a fresh success and labels what it left behind', () => {
    render(
      <Overview
        {...pendingProps({
          selection: selection({ selectionId: 'sel-9', fileName: 'not-an-elf.elf', mapAttached: false, mapFileName: null }),
        })}
      />,
    );

    const current = screen.getByRole('group', { name: 'Selected artifact' });
    expect(current.textContent).toContain('not-an-elf.elf');
    expect(current.textContent).toMatch(/not analyzed yet/i);

    const previous = screen.getByRole('region', { name: 'Previous analysis' });
    expect(previous.textContent).toContain('not an analysis of not-an-elf.elf');
    expect(previous.textContent).toContain('firmware.elf');
    expect(screen.getByRole('group', { name: /Previous key figures/i })).toBeDefined();
  });

  it('R2-T7 returns the current figures when the pending selection is genuinely analyzed', () => {
    const { rerender } = render(<Overview {...pendingProps()} />);
    expect(screen.queryByRole('group', { name: 'Input capabilities' })).toBeNull();

    // The same props `App` passes the moment `onLastGoodChange` has moved `lastGood` and its handle together.
    // The Flash figure is `memory.nonvolatileImageFootprint.bytes`, not the artifact size, so the new result has
    // to differ in the field the page actually reads.
    rerender(
      <Overview
        summary={summary({
          artifact: { ...summary().artifact, fileName: 'firmware.elf', byteSize: 613_568 },
          identity: { ...summary().identity, snapshotId: SNAP_A },
          symbolCount: 999,
          memory: {
            ...summary().memory,
            nonvolatileImageFootprint: {
              ...summary().memory.nonvolatileImageFootprint,
              bytes: 771_000,
            },
          },
        })}
        selection={selection({ selectionId: 'sel-3', fileName: 'firmware.elf' })}
        analyzedSelectionId="sel-3"
        gateRun={passRun(SNAP_A)}
        project={null}
        unit="bytes"
        onOpen={() => undefined}
      />,
    );

    expect(screen.queryByRole('region', { name: 'Previous analysis' })).toBeNull();
    expect(screen.getByRole('group', { name: 'Input capabilities' })).toBeDefined();
    const figures = screen.getByRole('group', { name: 'Key figures' });
    expect(figures.textContent).toContain('771,000');
    expect(figures.textContent).toContain('999');
    expect(figures.textContent).not.toContain('497,840');
    expect(figures.textContent).not.toContain('1284');
  });

  it('R2-T8 keeps U1P-R1’s other-build guard and its full identities while the analysis stays current', () => {
    renderOverview({
      summary: summary({ identity: { ...summary().identity, snapshotId: SNAP_B } }),
      selection: selection({ selectionId: 'sel-2' }),
      analyzedSelectionId: 'sel-2',
      gateRun: passRun(SNAP_A),
    });

    // §4.F: a currently authoritative summary keeps its capabilities and figures; only the verdict stays borrowed-free.
    expect(screen.getByRole('group', { name: 'Input capabilities' })).toBeDefined();
    expect(screen.getByRole('group', { name: 'Key figures' })).toBeDefined();
    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    expect(ship.textContent).toMatch(/judged a different build/);
    expect(ship.textContent).toContain(SNAP_A);
    expect(ship.textContent).toContain(SNAP_B);
    expect(ship.textContent).not.toMatch(/Clear —/);
  });

  it('R2-T9 keeps U1P-R1’s other-policy guard neutral', () => {
    renderOverview({
      summary: summary({ identity: { ...summary().identity, snapshotId: SNAP_B } }),
      selection: selection({ selectionId: 'sel-2' }),
      analyzedSelectionId: 'sel-2',
      gateRun: passRun(SNAP_B),
      project: project({ policySha256: 'c0ffee'.repeat(10) + 'c0' }),
    });

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    expect(ship.textContent).toMatch(/different policy fingerprint/);
    expect(ship.textContent).toContain('c0ffee'.repeat(10) + 'c0');
    expect(ship.textContent).not.toMatch(/Clear —/);
  });

  it('R2-T10 leaves the real verdict alone when run, build and policy all agree', () => {
    renderOverview({
      summary: summary({ identity: { ...summary().identity, snapshotId: SNAP_B } }),
      selection: selection({ selectionId: 'sel-2' }),
      analyzedSelectionId: 'sel-2',
      gateRun: passRun(SNAP_B),
      project: project(),
    });

    const ship = screen.getByRole('region', { name: 'Can we ship now?' });
    expect(ship.textContent).toMatch(/Clear —/);
    expect(ship.textContent).toContain(`Judged snapshot ${SNAP_B}`);
    expect(screen.queryByRole('group', { name: 'Selected artifact' })).toBeNull();
  });

  it('R2-T11 says plainly when there is neither a selection nor an analysis', () => {
    renderOverview({ summary: null, selection: null, analyzedSelectionId: null, gateRun: null });

    expect(screen.getByText(/Nothing has been analyzed in this session yet/)).toBeDefined();
    expect(screen.queryByRole('group', { name: /Key figures/ })).toBeNull();
    expect(screen.queryByRole('group', { name: /Previous/ })).toBeNull();
    expect(screen.getByRole('button', { name: 'Choose an artifact' })).toBeDefined();
  });

  it('R2-T12 names a previous result in text a reader can hear, not only in a distant warning', () => {
    render(<Overview {...pendingProps()} />);

    const previous = screen.getByRole('region', { name: 'Previous analysis' });
    const note = within(previous).getByRole('note');
    expect(note.textContent).toMatch(/^Previous analysis of firmware\.elf\./);
    expect(note.textContent).toMatch(/same name|not an analysis of/);
    // The disclaimer is metadata-styled secondary text in the same block as the figures it disclaims. Mutation
    // M4 swapped this class for the generic `.quiet` and every other assertion still passed, so the style that
    // keeps the note subordinate to the answer is asserted here rather than trusted: a note painted at body
    // weight beside a verdict is a note nobody reads as a caveat.
    expect(note.className).toContain('stale');
    // Every retained region says so in its own accessible name.
    expect(
      within(previous)
        .getAllByRole('group', { name: /^Previous/i })
        .map((region) => region.getAttribute('aria-label')),
    ).toEqual(['Previous input capabilities', 'Previous key figures']);
    expect(previous.textContent).toContain(SNAP_B);
  });
});
