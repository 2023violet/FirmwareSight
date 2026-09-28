import { invoke } from '@tauri-apps/api/core';
import { beforeEach, describe, expect, it, vi, type Mock } from 'vitest';

import {
  analyzeSelection,
  attachMap,
  clearMap,
  isErrorEnvelope,
  selectArtifact,
  toEnvelope,
} from './bridge';
import type { ErrorEnvelopeDto } from './types';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

// `invoke` is generic over the reply type, which no test double can express without an
// assertion. The assertion below is the price of faking one boundary; the production call sites
// keep their real types.
type InvokeFn = (command: string, args?: Record<string, unknown>) => Promise<unknown>;
const invokeMock = invoke as unknown as Mock<InvokeFn>;

const ENVELOPE: ErrorEnvelopeDto = {
  code: 'ERR-GUARD-0001',
  message: 'Artifact is 600000000 bytes, above the 536870912 byte full-buffer limit.',
  operationId: 'op-9f',
  details: 'artifact is 600000000 bytes, above the 536870912 byte full-buffer limit',
  remediation: 'Raise max_full_buffer_bytes only if this size is expected.',
};

beforeEach(() => {
  invokeMock.mockReset();
});

describe('error envelope narrowing', () => {
  it('passes a real envelope through untouched', () => {
    expect(toEnvelope(ENVELOPE)).toEqual(ENVELOPE);
  });

  it('keeps the raw form when a rejection is not an envelope', () => {
    const envelope = toEnvelope('webview died mid-call');

    expect(envelope.code).toBe('ERR-INTERNAL-9001');
    expect(envelope.details).toBe('webview died mid-call');
    expect(envelope.operationId).toBe('unavailable');
  });

  it('survives a value that cannot be serialized', () => {
    const circular: Record<string, unknown> = {};
    circular['self'] = circular;

    const envelope = toEnvelope(circular);
    expect(envelope.code).toBe('ERR-INTERNAL-9001');
    expect(envelope.details).toContain('unrepresentable object');
  });

  it('recognizes only the three required fields as an envelope', () => {
    expect(isErrorEnvelope(null)).toBe(false);
    expect(isErrorEnvelope('ERR-PARSE-2002')).toBe(false);
    expect(isErrorEnvelope({ code: 'X', message: 'Y' })).toBe(false);
    expect(isErrorEnvelope({ code: 'X', message: 'Y', operationId: 'Z' })).toBe(true);
    // Extra properties are the norm across a JSON boundary and must not invalidate the shape.
    expect(isErrorEnvelope({ ...ENVELOPE, extra: 1 })).toBe(true);
  });
});

describe('ipc calls', () => {
  it('opens the dialog with no argument at all', async () => {
    invokeMock.mockResolvedValue({
      selectionId: 'sel-1',
      fileName: 'app.elf',
      mapFileName: null,
      mapAttached: false,
    });

    const outcome = await selectArtifact();
    expect(outcome.ok).toBe(true);
    expect(invokeMock).toHaveBeenCalledWith('select_artifact', undefined);
  });

  it('reports a cancelled dialog as a successful call with no selection', async () => {
    invokeMock.mockResolvedValue(null);

    const outcome = await selectArtifact();
    expect(outcome.ok).toBe(true);
    if (outcome.ok) {
      expect(outcome.value).toBeNull();
    }
  });

  it('sends a selection id to every selection command, never a path', async () => {
    invokeMock.mockResolvedValue({});

    await attachMap('sel-1');
    await clearMap('sel-1');
    await analyzeSelection('sel-1');

    expect(invokeMock).toHaveBeenCalledWith('attach_map', { selectionId: 'sel-1' });
    expect(invokeMock).toHaveBeenCalledWith('clear_map', { selectionId: 'sel-1' });
    expect(invokeMock).toHaveBeenCalledWith('analyze_selection', { selectionId: 'sel-1' });
    const sent = JSON.stringify(invokeMock.mock.calls);
    expect(sent).not.toContain('path');
    // A drive-letter or POSIX absolute path in the payload would mean the WebView named a
    // location, which is exactly the boundary ADR-0025 draws.
    expect(sent).not.toMatch(/[a-zA-Z]:[\\/]/);
    expect(sent).not.toMatch(/"[^"]*\/[^"]*"/);
  });

  it('turns a rejected envelope into a typed failure', async () => {
    invokeMock.mockRejectedValue(ENVELOPE);

    const outcome = await analyzeSelection('sel-1');
    expect(outcome.ok).toBe(false);
    if (!outcome.ok) {
      expect(outcome.envelope).toEqual(ENVELOPE);
    }
  });
});
