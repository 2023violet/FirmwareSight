import { invoke } from '@tauri-apps/api/core';
import { beforeEach, describe, expect, it, vi, type Mock } from 'vitest';

import { getAnalysisSummary, isErrorEnvelope, listFixtures, toEnvelope } from './bridge';
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
  it('returns the payload the shell sent', async () => {
    invokeMock.mockResolvedValue([{ key: 'p0_basic', label: 'P0 basic' }]);

    const outcome = await listFixtures();
    expect(outcome.ok).toBe(true);
    if (outcome.ok) {
      expect(outcome.value).toHaveLength(1);
    }
  });

  it('sends a fixture key, never a path', async () => {
    invokeMock.mockResolvedValue({});

    await getAnalysisSummary('p0_dual_region');
    expect(invokeMock).toHaveBeenCalledWith('get_analysis_summary', {
      fixture: 'p0_dual_region',
    });
  });

  it('turns a rejected envelope into a typed failure', async () => {
    invokeMock.mockRejectedValue(ENVELOPE);

    const outcome = await listFixtures();
    expect(outcome.ok).toBe(false);
    if (!outcome.ok) {
      expect(outcome.envelope).toEqual(ENVELOPE);
    }
  });
});
