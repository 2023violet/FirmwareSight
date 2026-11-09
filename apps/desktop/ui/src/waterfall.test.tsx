//! D7 growth waterfall (ADR-0039 D4): Core's steps as a table with a neutral bar aid.
//!
//! - growth reads ▲ and shrinkage ▼ in text, so direction never depends on colour;
//! - an Unknown step has no bar and states its reason; the totals still show;
//! - switching budget or grain asks the shell again.

import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { GrowthWaterfall } from './GrowthWaterfall';
import { queryGrowthWaterfall } from './ipc/bridge';
import type { WaterfallDto } from './ipc/types';
import { render } from './test/render';

vi.mock('./ipc/bridge', () => ({ queryGrowthWaterfall: vi.fn() }));
const fallMock = vi.mocked(queryGrowthWaterfall);

const step = (kind: WaterfallDto['steps'][number]['kind'], label: string, delta: number | null, total: number | null, extra = {}) => ({
  kind,
  label,
  change: kind === 'item' ? 'changed' : null,
  delta,
  total,
  folded: 0,
  reason: null,
  ...extra,
});

const FALL: WaterfallDto = {
  available: true,
  reason: null,
  side: 'flash',
  libraries: false,
  steps: [
    step('start', 'base', null, 108),
    step('item', 'imu.o', 54, 162),
    step('item', 'old.o', -2, 160, { change: 'removed' }),
    step('linker', 'linker and unattributed', null, null, { reason: 'a total is Unknown' }),
    step('end', 'target', null, 192),
  ],
};

describe('GrowthWaterfall', () => {
  beforeEach(() => {
    fallMock.mockReset();
  });

  it('reads direction in text, keeps Unknown without a bar, and shows both totals', async () => {
    fallMock.mockResolvedValue({ ok: true, value: FALL });
    render(<GrowthWaterfall diffId="diff-1" unit="bytes" />);
    const table = await screen.findByRole('table');
    expect(within(table).getByText('▲54 bytes')).toBeTruthy();
    expect(within(table).getByText('▼2 bytes')).toBeTruthy();
    expect(within(table).getByText('removed', { exact: false })).toBeTruthy();
    const linker = within(table).getByRole('rowheader', { name: 'Linker and unattributed' }).closest('tr');
    expect(linker?.textContent).toContain('Unknown');
    expect(linker?.textContent).toContain('a total is Unknown');
    expect(within(table).getByText('108 bytes')).toBeTruthy();
    expect(within(table).getByText('192 bytes')).toBeTruthy();
  });

  it('asks again for RAM and for libraries', async () => {
    fallMock.mockResolvedValue({ ok: true, value: FALL });
    render(<GrowthWaterfall diffId="diff-1" unit="bytes" />);
    await screen.findByRole('table');
    fireEvent.click(screen.getByRole('button', { name: 'RAM' }));
    await waitFor(() => {
      expect(fallMock).toHaveBeenLastCalledWith({ diffId: 'diff-1', side: 'ram', libraries: false, topN: 12 });
    });
    fireEvent.click(screen.getByRole('button', { name: 'Libraries' }));
    await waitFor(() => {
      expect(fallMock).toHaveBeenLastCalledWith({ diffId: 'diff-1', side: 'ram', libraries: true, topN: 12 });
    });
  });
});
