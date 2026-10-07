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
import type { AnalysisSummaryDto, GateFindingRowDto, GateRunDto } from './ipc/types';
import { precedes } from './test/order';

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

function renderOverview(over: {
  readonly summary?: AnalysisSummaryDto | null;
  readonly gateRun?: GateRunDto | null;
} = {}) {
  const onOpen = vi.fn<(target: OverviewTarget) => void>();
  render(
    <Overview
      summary={over.summary ?? null}
      selection={null}
      gateRun={over.gateRun ?? null}
      project={null}
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
