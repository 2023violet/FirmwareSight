import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';
import { getAnalysisSummary, listFixtures } from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type {
  AnalysisSummaryDto,
  ErrorEnvelopeDto,
  FixtureOptionDto,
} from './ipc/types';

// The bridge is the boundary; mocking it keeps these tests about what the screen shows rather
// than about Tauri's invoke plumbing, which its own tests cover.
vi.mock('./ipc/bridge', () => ({
  listFixtures: vi.fn(),
  getAnalysisSummary: vi.fn(),
}));

const listFixturesMock = vi.mocked(listFixtures);
const analyzeMock = vi.mocked(getAnalysisSummary);

const FIXTURES: readonly FixtureOptionDto[] = [
  { key: 'p0_basic', label: 'P0 basic (ELF only, no MAP)' },
  { key: 'p0_dual_region', label: 'P0 dual region (ELF + GNU ld MAP)' },
];

function ok<T>(value: T): IpcOutcome<T> {
  return { ok: true, value };
}

function fail(envelope: ErrorEnvelopeDto): IpcOutcome<never> {
  return { ok: false, envelope };
}

const PARSE_FAILURE: ErrorEnvelopeDto = {
  code: 'ERR-PARSE-2002',
  message: 'Could not parse artifact as ELF.',
  operationId: 'op-1a2b',
  details: 'malformed artifact: truncated section table',
  remediation: 'Choose the linker ELF output rather than a stripped or truncated copy.',
};

/**
 * A summary spelled out field by field, exactly like the generated type.
 *
 * Nothing is omitted: a test fixture that skipped fields would hide a real shape change, and the
 * generated contract requires every one of them (nulls included).
 */
function summary(overrides: Partial<AnalysisSummaryDto> = {}): AnalysisSummaryDto {
  const base: AnalysisSummaryDto = {
    fixture: 'p0_dual_region',
    artifact: {
      fileName: 'firmware.elf',
      kind: 'elf',
      sha256: 'b'.repeat(64),
      byteSize: 5432,
      parserId: 'object-elf/0.40',
      architecture: 'Arm',
      bitness: '32',
      endianness: 'little',
      entryPoint: '0x080003f8',
      entryPointUnknownReason: null,
      buildId: 'BuildId("9e42c1")',
      buildIdUnknownReason: null,
    },
    identity: {
      schema: 'firmwaresight.analyze/p0-internal-1',
      schemaStability: 'p0-internal',
      snapshotId: 'snap-deadbeef-p0-normalize-1',
      normalizationVersion: 'p0-normalize-1',
      createdByFwsightVersion: '0.1.0',
    },
    memory: {
      accountingRule: 'adr-0021-dual-budget',
      layoutSource: 'map',
      weakestEvidenceBasis: 'map-memory-configuration+elf-load',
      admissibleForHardBlock: true,
      nonvolatileImageFootprint: {
        state: 'exact',
        classification: 'observed',
        bytes: 4200,
        unattributed: [],
        reason: null,
      },
      runtimeRamFootprint: {
        state: 'exact',
        classification: 'observed',
        bytes: 1024,
        unattributed: [],
        reason: null,
      },
      dualAccountedSections: ['section:3(.data)'],
      excludedMetadataBytes: 210,
    },
    sectionCount: 12,
    symbolCount: 34,
    capabilities: {
      elf: 'supported',
      sections: 'available',
      symbols: 'available',
      debugInfo: 'available',
      map: 'provided',
      objectAttribution: 'available',
      git: 'unknown',
    },
    evidenceSummary: {
      total: 9,
      observed: 6,
      derived: 2,
      declared: 1,
      unknown: 0,
      fromMap: true,
    },
  };
  return { ...base, ...overrides };
}

beforeEach(() => {
  listFixturesMock.mockReset();
  analyzeMock.mockReset();
  listFixturesMock.mockResolvedValue(ok(FIXTURES));
  analyzeMock.mockResolvedValue(ok(summary()));
});

async function analyze(button = 'Analyze') {
  render(<App />);
  fireEvent.click(await screen.findByRole('button', { name: button }));
}

describe('P0 summary screen', () => {
  it('offers only the fixtures the shell declares', async () => {
    render(<App />);
    const select = (await screen.findByLabelText('Fixture')) as HTMLSelectElement;
    expect(Array.from(select.options, (option) => option.value)).toEqual([
      'p0_basic',
      'p0_dual_region',
    ]);
  });

  it('shows the core facts the same command line prints', async () => {
    await analyze();

    expect(await screen.findByText('5,432 bytes')).toBeDefined();
    expect(screen.getByText('Arm 32-bit little')).toBeDefined();
    expect(screen.getByText('0x080003f8')).toBeDefined();
    expect(screen.getByText('4,200 bytes')).toBeDefined();
    expect(screen.getByText('1,024 bytes')).toBeDefined();
    expect(screen.getByText('section:3(.data)')).toBeDefined();
    expect(screen.getByText('snap-deadbeef-p0-normalize-1')).toBeDefined();
  });

  it('labels evidence quality with a word and an icon, never colour alone', async () => {
    await analyze();

    expect(await screen.findByText('Admissible for a hard limit')).toBeDefined();
    expect(screen.getAllByText('Exact').length).toBe(2);
    expect(screen.getByText('Git')).toBeDefined();
  });

  it('treats Git as unknown rather than as a failure', async () => {
    await analyze();
    // Wait for the report to replace the "Analyzing" state before reading a row.
    await screen.findByText('5,432 bytes');

    const row = screen.getByText('Git').closest('div');
    expect(row?.textContent).toContain('unknown');
    // A degraded optional input is an Unknown state, not a BLOCK.
    expect(row?.querySelector('svg')).not.toBeNull();
    expect(row?.textContent).not.toContain('BLOCK');
  });

  it('reports a non-admissible basis as Unknown and says what would strengthen it', async () => {
    analyzeMock.mockResolvedValue(
      ok(
        summary({
          memory: {
            ...summary().memory,
            layoutSource: 'none',
            weakestEvidenceBasis: 'section-name-heuristic',
            admissibleForHardBlock: false,
          },
        }),
      ),
    );

    await analyze();

    expect(await screen.findByText('Not admissible for a hard limit')).toBeDefined();
    expect(screen.getByText(/Supply the linker MAP to strengthen it/)).toBeDefined();
  });

  it('presents a typed error with what happened, code, what to do and a diagnostics id', async () => {
    analyzeMock.mockResolvedValue(fail(PARSE_FAILURE));

    await analyze();

    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain('Could not parse artifact as ELF.');
    expect(alert.textContent).toContain('ERR-PARSE-2002');
    expect(alert.textContent).toContain('op-1a2b');
    expect(alert.textContent).toContain('Choose the linker ELF output');
    expect(screen.queryByText('5,432 bytes')).toBeNull();
  });

  it('refuses to render a fixture the generated union does not name', async () => {
    listFixturesMock.mockResolvedValue(
      ok([{ key: 'not_in_the_union', label: 'Drifted fixture' }]),
    );

    render(<App />);
    await waitFor(() => {
      expect(screen.getByRole('alert').textContent).toContain('not_in_the_union');
    });
  });

  it('asks the shell to analyze a fixture key, never a path', async () => {
    render(<App />);
    const select = await screen.findByLabelText('Fixture');
    fireEvent.change(select, { target: { value: 'p0_basic' } });
    fireEvent.click(screen.getByRole('button', { name: 'Analyze' }));

    await waitFor(() => {
      expect(analyzeMock).toHaveBeenCalledWith('p0_basic');
    });
    expect(JSON.stringify(analyzeMock.mock.lastCall)).not.toContain('fixtures/');
  });
});
