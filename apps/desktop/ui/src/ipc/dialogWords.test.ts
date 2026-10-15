import { afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { dialogWords, setDialogLanguage, withDialog, type DialogKind } from './dialogWords';

const KINDS: DialogKind[] = [
  'pickArtifact',
  'pickMap',
  'pickProjectConfig',
  'pickReleaseFile',
  'pickBundleFolder',
  'saveAnalysisVisual',
  'saveCompareVisual',
  'saveGateVisual',
  'saveCompareJson',
  'saveCompareHtml',
  'savePolicy',
  'saveDiagnostics',
];

afterEach(() => setDialogLanguage(null));

describe('native dialog words', () => {
  it('sends nothing until a language is set, so the shell keeps its English', () => {
    expect(dialogWords('pickArtifact')).toBeUndefined();
    expect(withDialog('pickMap', { selectionId: 's' })).toEqual({ selectionId: 's' });
    expect(withDialog('pickArtifact')).toBeUndefined();
  });

  it('speaks the language on screen', () => {
    setDialogLanguage('zh-CN');
    expect(withDialog('pickMap', { selectionId: 's' })).toMatchObject({
      selectionId: 's',
      dialog: { title: '选择这个固件的链接器 MAP 文件' },
    });
    setDialogLanguage('en');
    expect(dialogWords('pickMap')?.title).toBe('Choose the linker MAP for this firmware');
  });

  for (const language of ['en', 'zh-CN'] as const) {
    it(`gives every dialog a title within the shell's bounds, and every Save a one-{file} message (${language})`, () => {
      setDialogLanguage(language);
      for (const kind of KINDS) {
        const words = dialogWords(kind);
        expect(words?.title?.length ?? 0).toBeGreaterThan(0);
        expect(words?.title?.length ?? 0).toBeLessThanOrEqual(160);
        if (kind.startsWith('save')) {
          expect(words?.replaceMessage?.split('{file}').length).toBe(2);
          expect(words?.replaceConfirm).toBeTruthy();
          expect(words?.replaceKeep).toBeTruthy();
        }
        if (words?.filter !== null && words?.filter !== undefined) {
          expect(words.filter.split('{format}').length).toBe(2);
        }
      }
    });
  }
});

describe('focus after a native dialog', () => {
  it('returns focus to the control that opened the dialog', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const bridge = await import('./bridge');
    const button = document.createElement('button');
    document.body.appendChild(button);
    button.focus();
    vi.mocked(invoke).mockImplementationOnce(async () => {
      // What Windows does when a native dialog closes: focus goes to the window, not the button.
      button.blur();
      return null;
    });
    await bridge.selectArtifact();
    expect(document.activeElement).toBe(button);
    button.remove();
  });
});
