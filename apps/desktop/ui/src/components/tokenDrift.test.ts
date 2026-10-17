/**
 * The numbers the shell cannot read from `tokens.css`, held to the token file.
 *
 * A media query cannot use a custom property, and `tauri.conf.json` is not CSS, so the two breakpoints
 * and the window minimum are written out by hand in three places (R1 review S3). This is what stops one
 * of them drifting from `assets/design-tokens.json` on its own.
 */

import { describe, expect, it } from 'vitest';

import tokenFile from '../../../../../assets/design-tokens.json';
import tauriConf from '../../../src-tauri/tauri.conf.json';
import { DOCK_QUERY } from './Inspector';
import { NARROW_QUERY } from './Rail';

interface Tokens {
  readonly layout: {
    readonly breakpoint: { readonly rail_collapse: number; readonly inspector_dock: number };
    readonly desktop_min: { readonly width: number; readonly height: number };
  };
}

interface TauriConf {
  readonly app: { readonly windows: readonly { readonly minWidth: number; readonly minHeight: number }[] };
}

const tokens: Tokens = tokenFile;

describe('hand-written layout numbers', () => {
  it('match the token breakpoints', () => {
    expect(NARROW_QUERY).toBe(`(width < ${tokens.layout.breakpoint.rail_collapse}px)`);
    expect(DOCK_QUERY).toBe(`(width >= ${tokens.layout.breakpoint.inspector_dock}px)`);
  });

  it('give the window the token minimum', () => {
    const conf: TauriConf = tauriConf;
    const window = conf.app.windows[0];
    expect(window?.minWidth).toBe(tokens.layout.desktop_min.width);
    expect(window?.minHeight).toBe(tokens.layout.desktop_min.height);
  });
});
