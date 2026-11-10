/**
 * History · Trends (ADR-0041 D4). The component lays out what one IPC read returned: these tests check that
 * every value reaches the table, that Unknown stays Unknown and a floor says so, that the three region
 * answers read differently, and that selection and details work from the keyboard's table.
 */

import { fireEvent, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { Trends } from './Trends';
import { queryHistoryTrend } from './ipc/bridge';
import type { HistoryTrendDto, TrendPointDto } from './ipc/types';
import { render } from './test/render';

vi.mock('./ipc/bridge', () => ({ queryHistoryTrend: vi.fn() }));
const trendMock = vi.mocked(queryHistoryTrend);

function point(index: number, overrides: Partial<TrendPointDto> = {}): TrendPointDto {
  return {
    buildId: `build-${index}`,
    snapshotId: `snap-${index}`,
    fileName: `fw-${index}.elf`,
    storedAt: `2026-11-0${index + 1}T10:00:00Z`,
    flash: { state: 'exact', bytes: 1000 + index * 100, delta: index === 0 ? null : 100 },
    ram: { state: 'exact', bytes: 500, delta: index === 0 ? null : 0 },
    gate: null,
    releases: [],
    pinned: false,
    regions: 'recorded',
    ...overrides,
  };
}

function trend(overrides: Partial<HistoryTrendDto> = {}): HistoryTrendDto {
  return {
    points: [
      point(0, { regions: 'notRecorded', pinned: true }),
      point(1, { flash: { state: 'unknown', bytes: null, delta: null }, gate: 'BLOCK' }),
      point(2, { ram: { state: 'partial', bytes: 700, delta: null }, releases: ['1.4.0'], gate: 'PASS' }),
    ],
    regions: [
      {
        name: 'RAM',
        kind: 'ram',
        cells: [
          { state: 'notRecorded', capacity: null, used: null, remaining: null, exceeded: false, exact: false },
          { state: 'recorded', capacity: 1000, used: 600, remaining: 400, exceeded: false, exact: true },
          { state: 'recorded', capacity: 1000, used: 940, remaining: 60, exceeded: false, exact: true },
        ],
        latest: { pointIndex: 2, capacity: 1000, used: 940, remaining: 60, remainingPerMille: 60, exceeded: false, exact: true },
      },
      {
        name: 'FLASH',
        kind: 'flash',
        cells: [
          { state: 'notRecorded', capacity: null, used: null, remaining: null, exceeded: false, exact: false },
          { state: 'recorded', capacity: 4096, used: 1000, remaining: 3096, exceeded: false, exact: true },
          { state: 'recorded', capacity: 4096, used: 1200, remaining: null, exceeded: false, exact: false },
        ],
        latest: { pointIndex: 2, capacity: 4096, used: 1200, remaining: null, remainingPerMille: null, exceeded: false, exact: false },
      },
    ],
    moversFlash: [{ object: 'imu.o', values: [10, null, 64], first: 10, last: 64, delta: 54 }],
    moversRam: [],
    totalBuilds: 3,
    limit: 100,
    ...overrides,
  };
}

async function show(value: HistoryTrendDto = trend()) {
  trendMock.mockResolvedValue({ ok: true, value });
  const onInspect = vi.fn();
  const onCompare = vi.fn();
  render(<Trends unit="bytes" onInspect={onInspect} onCompare={onCompare} />);
  await screen.findByRole('region', { name: 'Trends over stored builds' });
  return { onInspect, onCompare };
}

function table(): HTMLElement {
  return screen.getByRole('table', { name: /Stored builds in the trend/ });
}

describe('History trends', () => {
  beforeEach(() => {
    trendMock.mockReset();
  });

  it('lists every stored build newest first, with Unknown, floors, Gate verdicts and marks', async () => {
    await show();
    const rows = within(table()).getAllByRole('row').slice(1);
    expect(rows.map((row) => within(row).getAllByRole('rowheader')[0]?.textContent)).toEqual([
      'fw-2.elf',
      'fw-1.elf',
      'fw-0.elf',
    ]);
    const [newest, middle, oldest] = rows as [HTMLElement, HTMLElement, HTMLElement];
    // The Flash cell of the Unknown build: a badge, never a number.
    const flashCell = middle.querySelectorAll('td')[2] as HTMLElement;
    expect(within(flashCell).getByText('Unknown')).toBeDefined();
    expect(flashCell.textContent).not.toMatch(/\d/);
    expect(within(middle).getByText('Blocked')).toBeDefined();
    expect(within(newest).getByText(/at least 700 bytes/)).toBeDefined();
    expect(within(newest).getByText('1.4.0')).toBeDefined();
    expect(within(newest).getByText('Passed')).toBeDefined();
    expect(within(oldest).getByText('Baseline')).toBeDefined();
    expect(within(oldest).getByText('No run')).toBeDefined();
  });

  it('names the region with the least headroom and keeps older builds Unknown, not zero', async () => {
    await show();
    const lead = screen.getByText('Least headroom').closest('div') as HTMLElement;
    expect(within(lead).getByText('RAM')).toBeDefined();
    expect(within(lead).getByText('60 bytes left of 1,000 bytes (6.0%)')).toBeDefined();
    const regions = screen.getByRole('region', { name: 'Region headroom' });
    expect(within(regions).getAllByText('1 older builds: Unknown (stored before this version)')).toHaveLength(2);
    // A partial region names its floor and claims no headroom.
    expect(within(regions).getByText('At least 1,200 bytes used of 4,096 bytes; headroom not exact')).toBeDefined();
    expect(within(regions).queryByText(/2,896 bytes left/)).toBeNull();
  });

  it('compares two selected builds, older first, from the table', async () => {
    const { onCompare } = await show();
    const compare = screen.getByRole('button', { name: 'Compare selected (0/2)' });
    expect(compare.hasAttribute('disabled')).toBe(true);
    fireEvent.click(screen.getByRole('checkbox', { name: /Select fw-2\.elf/ }));
    fireEvent.click(screen.getByRole('checkbox', { name: /Select fw-0\.elf/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Compare selected (2/2)' }));
    expect(onCompare).toHaveBeenCalledWith('snap-0', 'snap-2');
  });

  it('opens a build in the Inspector with its stored facts', async () => {
    const { onInspect } = await show();
    fireEvent.click(screen.getByRole('button', { name: 'Details for fw-0.elf' }));
    const [evidence] = onInspect.mock.calls[0] as [{ details: [string, string][] }];
    const details = Object.fromEntries(evidence.details);
    expect(details['Region usage']).toBe('Unknown (stored before this version)');
    expect(details['Pinned baseline']).toBe('Yes');
    expect(details['Build id']).toBe('build-0');
  });

  it('switches the object movers between Flash and RAM', async () => {
    await show();
    const movers = screen.getByRole('region', { name: 'Biggest object movers' });
    expect(within(movers).getByText('imu.o')).toBeDefined();
    fireEvent.click(within(movers).getByRole('button', { name: 'RAM' }));
    expect(within(movers).getByText('No object moved across the stored builds with a MAP.')).toBeDefined();
  });

  it('says so when nothing is stored', async () => {
    trendMock.mockResolvedValue({ ok: true, value: trend({ points: [], regions: [], moversFlash: [], totalBuilds: 0 }) });
    render(<Trends unit="bytes" onInspect={vi.fn()} onCompare={vi.fn()} />);
    expect(await screen.findByText(/No build has been stored yet/)).toBeDefined();
    expect(screen.queryByRole('table')).toBeNull();
  });

  it('shows a failed read as an error with its code', async () => {
    trendMock.mockResolvedValue({
      ok: false,
      envelope: { code: 'ERR-STORAGE-4009', message: 'busy', operationId: 'op-1', details: null, remediation: null },
    });
    render(<Trends unit="bytes" onInspect={vi.fn()} onCompare={vi.fn()} />);
    expect(await screen.findByText('ERR-STORAGE-4009')).toBeDefined();
  });
});
