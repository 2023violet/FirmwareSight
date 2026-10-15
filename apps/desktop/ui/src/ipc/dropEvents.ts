/**
 * Drag-and-drop intake, as the WebView sees it.
 *
 * The Rust window handles the operating system's drop and keeps the paths; this module only hears the
 * phases (`drop_intake.rs`): `hover` and `leave` for the drop target, `dropped` with file names and a
 * single-use token, or `refused` with a reason word. Accepting a drop sends the token back - plus the
 * selection a lone MAP should attach to - and receives the same `SelectionDto` the file dialog produces.
 *
 * Outside the Tauri window (unit tests, a plain browser) there is no event bus; subscribing then does
 * nothing rather than throwing, because a missing drop target is not a reason for the page to fail.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { toEnvelope, type IpcOutcome } from './bridge';
import type { DropEventDto, SelectionDto } from './types';

export const DROP_EVENT = 'fs-intake-drop';

/** Subscribe to drop phases. Returns the unsubscribe function. */
export function subscribeToDrops(handler: (event: DropEventDto) => void): () => void {
  let stop: (() => void) | null = null;
  let cancelled = false;
  try {
    listen<DropEventDto>(DROP_EVENT, (event) => {
      handler(event.payload);
    }).then(
      (unlisten) => {
        if (cancelled) {
          unlisten();
        } else {
          stop = unlisten;
        }
      },
      () => undefined,
    );
  } catch {
    // No Tauri event bus here; there is nothing to subscribe to.
  }
  return () => {
    cancelled = true;
    stop?.();
  };
}

/** Stage the files of the drop named by `token`. A lone MAP attaches to `selectionId`. */
export async function acceptDroppedFiles(
  token: string,
  selectionId: string | null,
): Promise<IpcOutcome<SelectionDto>> {
  try {
    return { ok: true, value: await invoke<SelectionDto>('accept_dropped_files', { token, selectionId }) };
  } catch (reason: unknown) {
    return { ok: false, envelope: toEnvelope(reason) };
  }
}
