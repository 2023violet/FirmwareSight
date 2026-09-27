// Typed surface between the WebView and the shell.
//
// Two rules hold here: nothing is `any` (P0 prompt 42, ADR-0019), and a rejection carries the
// same `ErrorEnvelope` shape the CLI prints, so one failure looks the same on both surfaces
// instead of becoming a bare string in one and an object in the other.
//
// Calls resolve to an outcome rather than throwing. The `unknown` a rejected `invoke` produces
// is narrowed exactly once, here, so no component has to catch-and-guess.

import { invoke } from '@tauri-apps/api/core';

import type {
  AnalysisSummaryDto,
  ErrorEnvelopeDto,
  FixtureKey,
  FixtureOptionDto,
} from './types';

/** The commands the shell actually registers. A typo here is a compile error, not a surprise. */
const LIST_FIXTURES = 'list_fixtures';
const GET_ANALYSIS_SUMMARY = 'get_analysis_summary';

export type IpcOutcome<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly envelope: ErrorEnvelopeDto };

export async function listFixtures(): Promise<IpcOutcome<readonly FixtureOptionDto[]>> {
  return await call<readonly FixtureOptionDto[]>(LIST_FIXTURES);
}

export async function getAnalysisSummary(
  fixture: FixtureKey,
): Promise<IpcOutcome<AnalysisSummaryDto>> {
  return await call<AnalysisSummaryDto>(GET_ANALYSIS_SUMMARY, { fixture });
}

async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<IpcOutcome<T>> {
  try {
    return { ok: true, value: await invoke<T>(command, args) };
  } catch (reason: unknown) {
    return { ok: false, envelope: toEnvelope(reason) };
  }
}

/**
 * Narrow a rejection to the error envelope.
 *
 * A value crossing the Tauri boundary is JSON, so the shape is checked rather than asserted.
 * Anything that fails the check is reported as an internal error with its raw form attached -
 * guessing a code for it would write a fact that was never observed.
 */
export function toEnvelope(reason: unknown): ErrorEnvelopeDto {
  if (isErrorEnvelope(reason)) {
    return reason;
  }
  return {
    code: 'ERR-INTERNAL-9001',
    message: 'FirmwareSight hit an internal error.',
    operationId: 'unavailable',
    details: describe(reason),
    remediation: 'Report this message; no artifact data is needed.',
  };
}

export function isErrorEnvelope(value: unknown): value is ErrorEnvelopeDto {
  if (typeof value !== 'object' || value === null) {
    return false;
  }
  const candidate: Record<string, unknown> = { ...value };
  return (
    typeof candidate['code'] === 'string' &&
    typeof candidate['message'] === 'string' &&
    typeof candidate['operationId'] === 'string'
  );
}

function describe(reason: unknown): string {
  if (typeof reason === 'string') {
    return reason;
  }
  if (reason instanceof Error) {
    return reason.message;
  }
  try {
    return JSON.stringify(reason) ?? String(reason);
  } catch {
    // Circular or otherwise unserializable; the type name is still honest evidence.
    return `unrepresentable ${typeof reason}`;
  }
}
