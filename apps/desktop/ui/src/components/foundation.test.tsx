/**
 * R1 foundation components (ADR-0034, `06_DELIVERY/15`).
 *
 * Each block holds one component to the promise its header comment makes: the rail keeps every page's
 * accessible name when it collapses, a delta never reads an absent side as 0, the Inspector returns focus
 * to the value that opened it, the table names its rows and columns, and so on. Pages move onto these in
 * R2–R5; until then this file is what keeps them honest.
 */

import { act, fireEvent, screen, within } from '@testing-library/react';
import { useRef, useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { render } from '../test/render';
import { BuildPicker, type BuildOption } from './BuildPicker';
import { DataTable, FilterBar, type Column } from './DataTable';
import { Absent, DeltaValue } from './DeltaValue';
import { FactValue, type FactEvidence } from './FactValue';
import { Icon, ICON_NAMES } from './Icon';
import { Inspector } from './Inspector';
import { LeadAnswer } from './LeadAnswer';
import { Notice } from './Notice';
import { Rail } from './Rail';
import { Sheet } from './Sheet';
import { StateBadge } from './StateBadge';
import { SubjectBar } from './SubjectBar';
import { Tabs, tabPanelProps } from './Tabs';

/**
 * jsdom has no `matchMedia`. This answers the two range queries the shell uses the way a window of
 * `width` px would, and `resize` moves the width and fires `change` like a real window crossing a
 * breakpoint.
 */
let resize: (width: number) => void = () => undefined;
function stubWidth(initial: number) {
  let width = initial;
  const listeners = new Set<() => void>();
  const evaluate = (query: string) => {
    const below = /width\s*<\s*(\d+)px/.exec(query);
    const atLeast = /width\s*>=\s*(\d+)px/.exec(query);
    return (
      (below === null || width < Number(below[1])) && (atLeast === null || width >= Number(atLeast[1]))
    );
  };
  vi.stubGlobal('matchMedia', (query: string) => ({
    get matches() {
      return evaluate(query);
    },
    media: query,
    addEventListener: (_type: string, listener: () => void) => listeners.add(listener),
    removeEventListener: (_type: string, listener: () => void) => listeners.delete(listener),
  }));
  resize = (next: number) => {
    width = next;
    act(() => {
      for (const listener of listeners) {
        listener();
      }
    });
  };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

const BYTES = (n: number) => `${n.toLocaleString('en-US')} B`;

const BASE_SHA = '85ae4fc4e2b8d460522ede8a00db1e1ea4cb5c3efe689962a707ae2f90c89724';
const SHA = '3c9daa4f7c09febe8e6350dded6b701ae2b95587a45db421393e42295903d9fc';

const FLASH: FactEvidence = {
  field: 'nonvolatile_image_footprint_bytes',
  value: '196 B',
  classification: 'observed',
  source: 'map',
  locator: 'memory-accounting/totals',
  rule: 'map-memory-configuration+elf-load',
  raw: '196',
};

describe('Icon', () => {
  it('draws every name in the set and hides it unless it stands alone', () => {
    const { container } = render(
      <div>
        {ICON_NAMES.map((name) => (
          <Icon key={name} name={name} />
        ))}
      </div>,
    );
    const drawn = container.querySelectorAll('svg[data-icon]');
    expect(drawn).toHaveLength(ICON_NAMES.length);
    for (const svg of drawn) {
      expect(svg.getAttribute('aria-hidden')).toBe('true');
      expect(svg.childElementCount).toBeGreaterThan(0);
    }
    render(<Icon name="pin" label="Pinned" />);
    expect(screen.getByRole('img', { name: 'Pinned' })).toBeTruthy();
  });

  it('gives each of the five states a different shape', () => {
    const shapes = (['pass', 'review', 'block', 'unknown', 'na'] as const).map((name) => {
      const { container } = render(<Icon name={name} />);
      return container.querySelector('svg')?.innerHTML ?? '';
    });
    expect(new Set(shapes).size).toBe(5);
  });

  it('keeps StateBadge as icon + label + count', () => {
    render(<StateBadge state="BLOCK" label="BLOCK" count={1} />);
    expect(screen.getByText('BLOCK')).toBeTruthy();
    expect(screen.getByText('1')).toBeTruthy();
    expect(document.querySelector('svg[data-icon="block"]')).not.toBeNull();
  });
});

describe('Rail', () => {
  const groups = [
    {
      label: 'Pages',
      entries: [
        { key: 'overview', label: 'Overview', icon: 'overview' },
        { key: 'analyze', label: 'Analyze', icon: 'analyze' },
      ],
    },
    { label: 'Help and about', entries: [{ key: 'help', label: 'Help', icon: 'help' }] },
  ] as const;

  it('marks the open page and reports a choice', () => {
    const onOpen = vi.fn();
    render(<Rail groups={groups} current="analyze" onOpen={onOpen} />);
    const pages = screen.getByRole('navigation', { name: 'Pages' });
    const analyze = within(pages).getByRole('button', { name: 'Analyze page' });
    expect(analyze.getAttribute('aria-current')).toBe('page');
    fireEvent.click(within(pages).getByRole('button', { name: 'Overview page' }));
    expect(onOpen).toHaveBeenCalledWith('overview');
  });

  it('starts collapsed below 1180 px and keeps every accessible name', () => {
    stubWidth(1100);
    render(<Rail groups={groups} current="overview" onOpen={() => undefined} />);
    const toggle = () => screen.getByRole('button', { name: 'Page names' });
    expect(toggle().getAttribute('aria-pressed')).toBe('false');
    const overview = screen.getByRole('button', { name: 'Overview page' });
    expect(overview.getAttribute('title')).toBe('Overview');
    fireEvent.click(toggle());
    expect(toggle().getAttribute('aria-pressed')).toBe('true');
    expect(screen.getByRole('button', { name: 'Overview page' }).getAttribute('title')).toBeNull();
    // The reader's choice outlives a resize across the breakpoint.
    resize(1000);
    expect(toggle().getAttribute('aria-pressed')).toBe('true');
  });

  it('follows the window across 1180 px until the reader chooses', () => {
    stubWidth(1440);
    render(<Rail groups={groups} current="overview" onOpen={() => undefined} />);
    const toggle = () => screen.getByRole('button', { name: 'Page names' });
    expect(toggle().getAttribute('aria-pressed')).toBe('true');
    resize(1179);
    expect(toggle().getAttribute('aria-pressed')).toBe('false');
    resize(1180);
    expect(toggle().getAttribute('aria-pressed')).toBe('true');
    fireEvent.click(toggle());
    expect(toggle().getAttribute('aria-pressed')).toBe('false');
  });
});

describe('FactValue and DeltaValue', () => {
  it('speaks the value with its evidence class and hands the evidence to the Inspector', () => {
    const onInspect = vi.fn();
    render(<FactValue evidence={FLASH} onInspect={onInspect} />);
    const button = screen.getByRole('button', { name: '196 B, Observed evidence. Show evidence' });
    fireEvent.click(button);
    expect(onInspect).toHaveBeenCalledWith(FLASH, button);
  });

  it('states a delta neutrally with a glyph and words', () => {
    const { container } = render(<DeltaValue delta={64} format={BYTES} />);
    expect(container.textContent).toContain('▲');
    expect(screen.getByText('increase of 64 B')).toBeTruthy();
    render(<DeltaValue delta={-4} format={BYTES} />);
    expect(screen.getByText('decrease of 4 B')).toBeTruthy();
  });

  it('never reads an absent side as zero', () => {
    const { container } = render(<DeltaValue delta={null} format={BYTES} />);
    expect(container.textContent).toContain('not present');
    expect(container.textContent).not.toContain('0');
    const zero = render(<DeltaValue delta={0} format={BYTES} />);
    expect(zero.container.textContent).toContain('no change');
    expect(zero.container.textContent).not.toContain('not present');
  });

  it('says not present in Chinese too', () => {
    const { container } = render(<Absent />, { language: 'zh-CN' });
    expect(container.textContent).toContain('不存在');
  });
});

describe('LeadAnswer', () => {
  it('puts the sentence, the one figure and its reference in a named region', () => {
    render(
      <LeadAnswer
        sentence="Flash used"
        figure={<FactValue evidence={FLASH} onInspect={() => undefined} />}
        reference="38.3% of the 512 B policy budget"
      />,
    );
    const region = screen.getByRole('region', { name: 'Flash used' });
    expect(within(region).getByText('38.3% of the 512 B policy budget')).toBeTruthy();
    expect(region.querySelectorAll('.figure')).toHaveLength(1);
  });
});

describe('Inspector', () => {
  it('docks from 1280 px, focuses its close button, and returns focus on Esc', () => {
    stubWidth(1440);
    const trigger = document.createElement('button');
    document.body.append(trigger);
    const onClose = vi.fn();
    render(<Inspector evidence={FLASH} returnFocus={trigger} onClose={onClose} />);
    const panel = screen.getByRole('complementary', { name: 'Evidence' });
    expect(within(panel).getByText('memory-accounting/totals')).toBeTruthy();
    expect(within(panel).getByText('Observed')).toBeTruthy();
    const close = screen.getByRole('button', { name: 'Close evidence panel' });
    expect(document.activeElement).toBe(close);
    fireEvent.keyDown(close, { key: 'Escape' });
    expect(onClose).toHaveBeenCalled();
    expect(document.activeElement).toBe(trigger);
    trigger.remove();
  });

  it('becomes a modal sheet below 1280 px and traps Tab inside it', () => {
    stubWidth(1100);
    render(<Inspector evidence={FLASH} returnFocus={null} onClose={() => undefined} />);
    const dialog = screen.getByRole('dialog', { name: 'Evidence' });
    expect(dialog.getAttribute('aria-modal')).toBe('true');
    const copy = within(dialog).getByRole('button', { name: 'Copy raw value' });
    copy.focus();
    fireEvent.keyDown(copy, { key: 'Tab' });
    expect(document.activeElement).toBe(within(dialog).getByRole('button', { name: 'Close evidence panel' }));
  });

  it('keeps Esc and the focus trap when focus has fallen to the page body', () => {
    stubWidth(1100);
    const onClose = vi.fn();
    render(<Inspector evidence={FLASH} returnFocus={null} onClose={onClose} />);
    const close = screen.getByRole('button', { name: 'Close evidence panel' });
    const outside = document.createElement('button');
    document.body.append(outside);
    outside.focus();
    expect(document.activeElement).toBe(close);
    close.blur();
    fireEvent.keyDown(document.body, { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(1);
    outside.remove();
  });

  it('does not remount, and keeps focus, when the window crosses 1280 px while open', () => {
    stubWidth(1440);
    render(<Inspector evidence={FLASH} returnFocus={null} onClose={() => undefined} />);
    const panel = screen.getByRole('complementary', { name: 'Evidence' });
    const copy = within(panel).getByRole('button', { name: 'Copy raw value' });
    copy.focus();
    resize(1100);
    expect(screen.getByRole('dialog', { name: 'Evidence' })).toBe(panel);
    expect(document.activeElement).toBe(copy);
  });

  it('returns focus to the trigger when the page clears the evidence', () => {
    stubWidth(1440);
    const trigger = document.createElement('button');
    document.body.append(trigger);
    const { rerender } = render(
      <Inspector evidence={FLASH} returnFocus={trigger} onClose={() => undefined} />,
    );
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Close evidence panel' }));
    rerender(<Inspector evidence={null} returnFocus={trigger} onClose={() => undefined} />);
    expect(document.activeElement).toBe(trigger);
    trigger.remove();
  });

  it('leaves out fields the evidence does not carry', () => {
    render(
      <Inspector
        evidence={{ field: 'entry_point', value: '0x08000001', classification: 'derived' }}
        returnFocus={null}
        onClose={() => undefined}
      />,
    );
    expect(screen.queryByText('Locator')).toBeNull();
    expect(screen.getByText('Derived')).toBeTruthy();
  });

  it('reports a copy it could not make instead of pretending', async () => {
    vi.stubGlobal('navigator', { clipboard: { writeText: () => Promise.reject(new Error('denied')) } });
    render(<Inspector evidence={FLASH} returnFocus={null} onClose={() => undefined} />);
    fireEvent.click(screen.getByRole('button', { name: 'Copy raw value' }));
    await act(async () => {
      await Promise.resolve();
    });
    expect(screen.getByText('Could not copy. Select the value and copy it by hand.')).toBeTruthy();
  });

  it('renders nothing when there is no evidence', () => {
    const { container } = render(<Inspector evidence={null} returnFocus={null} onClose={() => undefined} />);
    expect(container.childElementCount).toBe(0);
  });
});

describe('Tabs', () => {
  const items = [
    { key: 'memory', label: 'Memory' },
    { key: 'sections', label: 'Sections', count: 8 },
    { key: 'symbols', label: 'Symbols', count: 32 },
  ] as const;

  it('is a tablist with one tab in the Tab order and arrow-key selection', () => {
    const onSelect = vi.fn();
    render(
      <>
        <Tabs idBase="an" label="Analyze views" items={items} current="sections" onSelect={onSelect} />
        <div {...tabPanelProps('an', 'sections')}>panel</div>
      </>,
    );
    const tabs = screen.getAllByRole('tab');
    expect(tabs.map((tab) => tab.tabIndex)).toEqual([-1, 0, -1]);
    expect(screen.getByRole('tabpanel', { name: /Sections/ })).toBeTruthy();
    fireEvent.keyDown(tabs[1]!, { key: 'ArrowRight' });
    expect(onSelect).toHaveBeenLastCalledWith('symbols');
    fireEvent.keyDown(tabs[1]!, { key: 'Home' });
    expect(onSelect).toHaveBeenLastCalledWith('memory');
    fireEvent.keyDown(tabs[0]!, { key: 'ArrowLeft' });
    expect(onSelect).toHaveBeenLastCalledWith('symbols');
  });
});

describe('Notice', () => {
  it('raises an error as an alert with the four answers', () => {
    render(
      <Notice
        tone="error"
        title="Could not analyze build/app.elf"
        details={[
          ['What happened', 'The file is not an ELF.'],
          ['Why we know', 'object-elf refused the header.'],
          ['What to do', 'Choose the linker output.'],
          ['Diagnostics ID', 'op-47e8'],
        ]}
      />,
    );
    const alert = screen.getByRole('alert');
    expect(within(alert).getByText('Diagnostics ID')).toBeTruthy();
    expect(alert.querySelector('svg[data-icon="block"]')).not.toBeNull();
  });

  it('keeps Unknown neutral: a status with the hollow glyph', () => {
    render(<Notice tone="unknown" title="No object attribution">It was analyzed without a MAP.</Notice>);
    expect(screen.getByRole('status').querySelector('svg[data-icon="unknown"]')).not.toBeNull();
  });
});

describe('DataTable and FilterBar', () => {
  interface Row {
    readonly name: string;
    readonly size: number;
  }
  const rows: Row[] = [
    { name: '.text', size: 192 },
    { name: '.bss', size: 40 },
    { name: '.data', size: 4 },
  ];
  const columns: Column<Row>[] = [
    { key: 'name', header: 'Name', cell: (row) => row.name, sortValue: (row) => row.name },
    { key: 'size', header: 'Size', cell: (row) => BYTES(row.size), numeric: true, sortValue: (row) => row.size },
  ];

  it('names itself, scopes its headers and sorts on its own when uncontrolled', () => {
    render(<DataTable caption="Sections · 3" columns={columns} rows={rows} rowKey={(row) => row.name} />);
    const table = screen.getByRole('table', { name: 'Sections · 3' });
    expect(within(table).getAllByRole('columnheader')).toHaveLength(2);
    expect(within(table).getAllByRole('rowheader').map((cell) => cell.textContent)).toEqual([
      '.text',
      '.bss',
      '.data',
    ]);
    fireEvent.click(screen.getByRole('button', { name: 'Size' }));
    const size = within(table).getAllByRole('columnheader')[1]!;
    expect(size.getAttribute('aria-sort')).toBe('descending');
    fireEvent.click(screen.getByRole('button', { name: 'Size' }));
    expect(size.getAttribute('aria-sort')).toBe('ascending');
    expect(within(table).getAllByRole('rowheader').map((cell) => cell.textContent)).toEqual([
      '.data',
      '.bss',
      '.text',
    ]);
  });

  it('only reports the sort when the shell orders the rows', () => {
    const onSort = vi.fn();
    render(
      <DataTable caption="Symbols" columns={columns} rows={rows} rowKey={(row) => row.name} sort={null} onSort={onSort} />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Name' }));
    expect(onSort).toHaveBeenCalledWith({ key: 'name', direction: 'desc' });
    expect(screen.getAllByRole('rowheader')[0]!.textContent).toBe('.text');
  });

  it('filters, says when nothing is left, and opens a row in the Inspector', () => {
    const onRowAction = vi.fn();
    const { rerender } = render(
      <DataTable
        caption="Sections"
        columns={columns}
        rows={rows}
        rowKey={(row) => row.name}
        filterText={{ query: 'bs', of: (row) => row.name }}
        onRowAction={onRowAction}
        rowActionLabel={(row) => `Show evidence for ${row.name}`}
      />,
    );
    expect(screen.getAllByRole('rowheader')).toHaveLength(1);
    fireEvent.click(screen.getByRole('button', { name: 'Show evidence for .bss' }));
    expect(onRowAction).toHaveBeenCalledWith(rows[1], expect.any(HTMLElement));
    rerender(
      <DataTable
        caption="Sections"
        columns={columns}
        rows={rows}
        rowKey={(row) => row.name}
        filterText={{ query: 'nothing', of: (row) => row.name }}
      />,
    );
    expect(screen.getByText('No rows match.')).toBeTruthy();
  });

  it('labels the filter input and offers a clear button only when it holds text', () => {
    const onChange = vi.fn();
    const { rerender } = render(<FilterBar label="Filter sections" value="" onChange={onChange} shown={3} total={8} />);
    const input = screen.getByRole('searchbox', { name: 'Filter sections' });
    expect(screen.queryByRole('button', { name: 'Clear filter' })).toBeNull();
    expect(screen.getByText('3 of 8')).toBeTruthy();
    fireEvent.change(input, { target: { value: '.t' } });
    expect(onChange).toHaveBeenCalledWith('.t');
    rerender(<FilterBar label="Filter sections" value=".t" onChange={onChange} />);
    fireEvent.click(screen.getByRole('button', { name: 'Clear filter' }));
    expect(onChange).toHaveBeenLastCalledWith('');
  });
});

describe('DataTable server mode and FilterBar apply mode (R3)', () => {
  interface Row {
    readonly name: string;
    readonly size: number;
  }
  const rows: Row[] = [
    { name: 'b', size: 1 },
    { name: 'a', size: 2 },
  ];
  const columns: Column<Row>[] = [
    { key: 'size', header: 'Size', numeric: true, sortable: true, firstDirection: 'asc', cell: (row) => String(row.size) },
    { key: 'name', header: 'Name', sortable: true, firstDirection: 'asc', cell: (row) => row.name },
    { key: 'note', header: 'Note', cell: () => '-' },
  ];

  it('asks the shell for the first direction a column names, never reorders, and names rows by rowHeader', () => {
    const onSort = vi.fn();
    render(
      <DataTable caption="Rows" columns={columns} rows={rows} rowKey={(row) => row.name} rowHeader="name" sort={null} onSort={onSort} />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Name' }));
    expect(onSort).toHaveBeenLastCalledWith({ key: 'name', direction: 'asc' });
    expect(screen.getAllByRole('rowheader').map((cell) => cell.textContent)).toEqual(['b', 'a']);
    expect(screen.queryByRole('button', { name: 'Note' })).toBeNull();
  });

  it('applies a filter once per decision: typing edits the draft, Apply sends it', () => {
    const onApply = vi.fn();
    const onChange = vi.fn();
    const { rerender } = render(
      <FilterBar label="Filter rows" formLabel="Filter the rows" value="" onChange={onChange} onApply={onApply} />,
    );
    fireEvent.change(screen.getByRole('searchbox', { name: 'Filter rows' }), { target: { value: 'ab' } });
    expect(onChange).toHaveBeenCalledWith('ab');
    expect(onApply).not.toHaveBeenCalled();
    rerender(<FilterBar label="Filter rows" formLabel="Filter the rows" value="ab" onChange={onChange} onApply={onApply} />);
    fireEvent.click(screen.getByRole('button', { name: 'Apply filter' }));
    expect(onApply).toHaveBeenCalledWith('ab');
    expect(screen.getByRole('form', { name: 'Filter the rows' })).toBeDefined();
  });
});

describe('BuildPicker', () => {
  const options: BuildOption[] = [
    { id: 'a', fileName: 'firmware.elf', sha256: '85ae4fc4e2b8d460522ede8a00db1e1e', importedAt: '2026-10-16 09:12', pinned: true },
    { id: 'b', fileName: 'firmware.elf', sha256: SHA, importedAt: '2026-10-16 09:14' },
  ];

  it('opens a listbox, moves with arrows and chooses with Enter', () => {
    const onChange = vi.fn();
    render(<BuildPicker label="Base" options={options} value="a" onChange={onChange} />);
    const combo = screen.getByRole('combobox', { name: /Base/ });
    expect(combo.textContent).toContain('85ae4fc4e2b8');
    fireEvent.keyDown(combo, { key: 'ArrowDown' });
    const list = screen.getByRole('listbox');
    expect(within(list).getAllByRole('option')).toHaveLength(2);
    expect(within(list).getByText('Pinned baseline')).toBeTruthy();
    fireEvent.keyDown(combo, { key: 'ArrowDown' });
    expect(combo.getAttribute('aria-activedescendant')).toMatch(/option-1$/);
    fireEvent.keyDown(combo, { key: 'Enter' });
    expect(onChange).toHaveBeenCalledWith('b');
    expect(screen.queryByRole('listbox')).toBeNull();
  });

  it('closes on Esc without changing the choice and is disabled with no builds', () => {
    const onChange = vi.fn();
    const { rerender } = render(<BuildPicker label="Base" options={options} value="a" onChange={onChange} />);
    const combo = screen.getByRole('combobox', { name: /Base/ });
    fireEvent.keyDown(combo, { key: 'ArrowDown' });
    fireEvent.keyDown(combo, { key: 'Escape' });
    expect(screen.queryByRole('listbox')).toBeNull();
    expect(onChange).not.toHaveBeenCalled();
    rerender(<BuildPicker label="Base" options={[]} value={null} onChange={onChange} />);
    const empty = screen.getByRole('combobox', { name: /Base/ }) as HTMLButtonElement;
    expect(empty.disabled).toBe(true);
    expect(empty.textContent).toContain('No stored builds yet');
  });
});

describe('SubjectBar', () => {
  it('names a pair by file and short hash, with MAP state and freshness', () => {
    render(
      <SubjectBar
        subject={{
          kind: 'pair',
          base: { fileName: 'firmware.elf', sha256: BASE_SHA, map: true, pinned: true },
          target: { fileName: 'firmware.elf', sha256: SHA, map: false },
        }}
        freshness="retained"
      />,
    );
    const bar = screen.getByRole('region', { name: 'Subject of this page' });
    expect(within(bar).getByText('firmware.elf · 3c9daa4f7c09')).toBeTruthy();
    expect(within(bar).getByText('MAP: attached')).toBeTruthy();
    expect(within(bar).getByText('MAP: Not provided')).toBeTruthy();
    expect(within(bar).getByText('Pinned baseline')).toBeTruthy();
    expect(within(bar).getByText('Previous result kept · current selection not analyzed')).toBeTruthy();
  });

  it('says when nothing is chosen, with no freshness claim', () => {
    render(<SubjectBar subject={null} freshness="latest" />);
    expect(screen.getByText('No build chosen')).toBeTruthy();
    expect(screen.queryByText('Latest')).toBeNull();
  });
});

describe('Sheet (R4)', () => {
  function Host({ initial }: { readonly initial: boolean }) {
    const [open, setOpen] = useState(false);
    const opener = useRef<HTMLButtonElement | null>(null);
    const field = useRef<HTMLInputElement | null>(null);
    return (
      <>
        <button
          ref={opener}
          type="button"
          onClick={() => {
            setOpen(true);
          }}
        >
          Open
        </button>
        <Sheet
          open={open}
          title="Edit things"
          returnFocus={opener.current}
          {...(initial ? { initialFocus: field } : {})}
          onClose={() => {
            setOpen(false);
          }}
          actions={<button type="button">Save</button>}
        >
          <input ref={field} aria-label="Name" />
        </Sheet>
      </>
    );
  }

  it('is a modal dialog that traps Tab, closes on Esc and returns focus to its opener', () => {
    render(<Host initial={false} />);
    const opener = screen.getByRole('button', { name: 'Open' });
    opener.focus();
    fireEvent.click(opener);
    const sheet = screen.getByRole('dialog', { name: 'Edit things' });
    expect(sheet.getAttribute('aria-modal')).toBe('true');
    const close = within(sheet).getByRole('button', { name: 'Close' });
    expect(document.activeElement).toBe(close);

    // Tab from the last control wraps to the first, and Shift+Tab from the first wraps to the last.
    const save = within(sheet).getByRole('button', { name: 'Save' });
    save.focus();
    fireEvent.keyDown(save, { key: 'Tab' });
    expect(document.activeElement).toBe(close);
    fireEvent.keyDown(close, { key: 'Tab', shiftKey: true });
    expect(document.activeElement).toBe(save);

    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(document.activeElement).toBe(opener);
  });

  it('focuses the decision it asks for when given one', () => {
    render(<Host initial />);
    fireEvent.click(screen.getByRole('button', { name: 'Open' }));
    expect(document.activeElement).toBe(screen.getByRole('textbox', { name: 'Name' }));
  });
});

describe('BuildPicker none entry (R4)', () => {
  it('lists a none entry first, shows it while nothing is chosen, and reports it as null', () => {
    const chosen: (string | null)[] = [];
    const options: BuildOption[] = [{ id: 's1', fileName: 'app.elf', sha256: 'a'.repeat(64), importedAt: '2026-10-17' }];
    const { rerender } = render(
      <BuildPicker label="Baseline" noneLabel="No baseline" options={options} value={null} onChange={(id) => chosen.push(id)} />,
    );
    const button = screen.getByRole('combobox', { name: 'Baseline' });
    expect(button.textContent).toContain('No baseline');
    fireEvent.click(button);
    const entries = screen.getAllByRole('option');
    expect(entries.map((entry) => entry.textContent)).toEqual(['No baseline', expect.stringContaining('app.elf')]);
    expect(entries[0]?.getAttribute('aria-selected')).toBe('true');
    fireEvent.click(entries[1] as HTMLElement);
    rerender(
      <BuildPicker label="Baseline" noneLabel="No baseline" options={options} value="s1" onChange={(id) => chosen.push(id)} />,
    );
    fireEvent.click(screen.getByRole('combobox', { name: 'Baseline' }));
    fireEvent.click(screen.getByRole('option', { name: 'No baseline' }));
    expect(chosen).toEqual(['s1', null]);
  });
});
