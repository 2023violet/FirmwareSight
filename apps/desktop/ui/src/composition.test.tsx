//! D7 Composition view (ADR-0039): the treemap is a picture of Core's numbers, never a source of them.
//!
//! - every child of the focus node is listed in the table, an Unknown one with its reason and no area;
//! - a drillable cell asks the shell for that node; the path's crumbs ask for an ancestor;
//! - switching Flash/RAM asks for the other budget from the top;
//! - a cell's accessible name carries its name, bytes and share, so the picture is never the only way.

import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { CompositionView } from './Composition';
import { queryComposition } from './ipc/bridge';
import type { CompositionDto, CompositionNodeDto } from './ipc/types';
import { render } from './test/render';

vi.mock('./ipc/bridge', () => ({ queryComposition: vi.fn() }));
const compositionMock = vi.mocked(queryComposition);

function node(partial: Partial<CompositionNodeDto> & Pick<CompositionNodeDto, 'id' | 'name'>): CompositionNodeDto {
  return {
    kind: 'object',
    bytes: null,
    classification: 'observed',
    reason: null,
    locator: null,
    capacity: null,
    folded: 0,
    children: [],
    ...partial,
  };
}

const ROOT: CompositionDto = {
  available: true,
  reason: null,
  side: 'flash',
  path: [{ id: 'root', name: 'Flash' }],
  node: node({
    id: 'root',
    name: 'Flash',
    kind: 'root',
    bytes: 1000,
    children: [
      node({ id: 'root/r:FLASH', name: 'FLASH', kind: 'region', bytes: 900, capacity: 4096, children: [node({ id: 'x', name: '.text', kind: 'section', bytes: 900 })] }),
      node({ id: 'root/linker', name: 'linker bytes', kind: 'linkerBytes', bytes: 100, classification: 'derived' }),
      node({ id: 'root/u', name: 'RW_RAM', kind: 'section', bytes: null, classification: 'unknown', reason: 'compressed: stored share unknown' }),
    ],
  }),
};

const REGION: CompositionDto = {
  ...ROOT,
  path: [
    { id: 'root', name: 'Flash' },
    { id: 'root/r:FLASH', name: 'FLASH' },
  ],
  node: node({ id: 'root/r:FLASH', name: 'FLASH', kind: 'region', bytes: 900, capacity: 4096, children: [node({ id: 'x', name: '.text', kind: 'section', bytes: 900 })] }),
};

describe('CompositionView', () => {
  beforeEach(() => {
    compositionMock.mockReset();
  });

  it('lists every part, Unknown with its reason and no area, and names cells with bytes and share', async () => {
    compositionMock.mockResolvedValue({ ok: true, value: ROOT });
    render(<CompositionView snapshotId="snap-1" unit="bytes" onInspect={vi.fn()} />);
    const table = await screen.findByRole('table', { name: 'Parts of Flash' });
    expect(within(table).getByText('compressed: stored share unknown')).toBeTruthy();
    const map = screen.getByRole('group', { name: /Treemap of Flash/ });
    const cells = within(map).getAllByRole('button');
    expect(cells).toHaveLength(2);
    expect(cells[0]?.getAttribute('aria-label')).toBe('Open FLASH, 900 bytes, 90.0 %');
    expect(compositionMock).toHaveBeenCalledWith({ snapshotId: 'snap-1', side: 'flash', focus: null });
  });

  it('drills into a cell, comes back through the path, and switches budget from the top', async () => {
    compositionMock.mockResolvedValueOnce({ ok: true, value: ROOT }).mockResolvedValueOnce({ ok: true, value: REGION }).mockResolvedValue({ ok: true, value: ROOT });
    render(<CompositionView snapshotId="snap-1" unit="bytes" onInspect={vi.fn()} />);
    fireEvent.click(await screen.findByRole('button', { name: /^Open FLASH/ }));
    await waitFor(() => {
      expect(compositionMock).toHaveBeenLastCalledWith({ snapshotId: 'snap-1', side: 'flash', focus: 'root/r:FLASH' });
    });
    const crumbs = await screen.findByRole('navigation', { name: 'Where you are' });
    fireEvent.click(await within(crumbs).findByRole('button', { name: 'Flash' }));
    await waitFor(() => {
      expect(compositionMock).toHaveBeenLastCalledWith({ snapshotId: 'snap-1', side: 'flash', focus: 'root' });
    });
    fireEvent.click(screen.getByRole('button', { name: 'RAM' }));
    await waitFor(() => {
      expect(compositionMock).toHaveBeenLastCalledWith({ snapshotId: 'snap-1', side: 'ram', focus: null });
    });
  });

  it('says why when the build is not held, instead of drawing an empty map', async () => {
    compositionMock.mockResolvedValue({ ok: true, value: { available: false, reason: 'analyze it again', side: 'flash', path: [], node: null } });
    render(<CompositionView snapshotId="snap-1" unit="bytes" onInspect={vi.fn()} />);
    expect((await screen.findByRole('status')).textContent).toBe('analyze it again');
    expect(screen.queryByRole('table')).toBeNull();
  });
});
