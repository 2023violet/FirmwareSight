/**
 * The words a native dialog shows, in the language on screen.
 *
 * Native pickers, Save dialogs and the replace confirmation are drawn by the operating system from strings
 * the Rust shell passes. `ADR-0032` keeps Rust independent of the UI language, so the shell cannot pick the
 * language itself; instead every bridge call that opens a dialog sends these words along, and
 * `dialog_text.rs` displays them after bounding them, falling back to English for anything missing. The
 * replace message carries a `{file}` placeholder that only Rust fills, so the dialog always names the file
 * Rust is about to replace.
 *
 * The language is set by `I18nProvider` whenever the resolved catalog changes. Before that happens - in a
 * unit test of the bridge, say - no words are sent and the shell keeps its English.
 */

import { translate, type Translate } from '../i18n';
import type { ResolvedLanguage } from '../i18n/language';
import type { DialogTextDto } from './types';

export type DialogKind =
  | 'pickArtifact'
  | 'pickMap'
  | 'pickProjectConfig'
  | 'pickReleaseFile'
  | 'pickBundleFolder'
  | 'saveAnalysisVisual'
  | 'saveCompareVisual'
  | 'saveGateVisual'
  | 'saveCompareJson'
  | 'saveCompareHtml'
  | 'savePolicy'
  | 'saveDiagnostics';

let current: Translate | null = null;

/** Called by the i18n provider; `null` stops sending words. */
export function setDialogLanguage(resolved: ResolvedLanguage | null): void {
  current = resolved === null ? null : translate(resolved);
}

function replaceWords(t: Translate, policy = false): Partial<DialogTextDto> {
  return {
    replaceTitle: t('common', 'dialog.replaceTitle'),
    replaceMessage: t('common', policy ? 'dialog.replacePolicyMessage' : 'dialog.replaceMessage'),
    replaceConfirm: t('common', 'dialog.replaceConfirm'),
    replaceKeep: t('common', 'dialog.replaceKeep'),
  };
}

/** The words for one dialog, or `undefined` when no language has been set. */
export function dialogWords(kind: DialogKind): DialogTextDto | undefined {
  const t = current;
  if (t === null) {
    return undefined;
  }
  const base: DialogTextDto = {
    title: null,
    filter: null,
    replaceTitle: null,
    replaceMessage: null,
    replaceConfirm: null,
    replaceKeep: null,
  };
  switch (kind) {
    case 'pickArtifact':
      return { ...base, title: t('common', 'dialog.pickArtifact') };
    case 'pickMap':
      return { ...base, title: t('common', 'dialog.pickMap') };
    case 'pickProjectConfig':
      return { ...base, title: t('common', 'dialog.pickProjectConfig') };
    case 'pickReleaseFile':
      return { ...base, title: t('common', 'dialog.pickReleaseFile') };
    case 'pickBundleFolder':
      return { ...base, title: t('common', 'dialog.pickBundleFolder') };
    case 'saveAnalysisVisual':
      return { ...base, ...replaceWords(t), title: t('common', 'dialog.saveAnalysisVisual'), filter: t('common', 'dialog.filterReport') };
    case 'saveCompareVisual':
      return { ...base, ...replaceWords(t), title: t('common', 'dialog.saveCompareVisual'), filter: t('common', 'dialog.filterReport') };
    case 'saveGateVisual':
      return { ...base, ...replaceWords(t), title: t('common', 'dialog.saveGateVisual'), filter: t('common', 'dialog.filterReport') };
    case 'saveCompareJson':
      return { ...base, ...replaceWords(t), title: t('common', 'dialog.saveCompareJson'), filter: t('common', 'dialog.filterExport') };
    case 'saveCompareHtml':
      return { ...base, ...replaceWords(t), title: t('common', 'dialog.saveCompareHtml'), filter: t('common', 'dialog.filterExport') };
    case 'savePolicy':
      return { ...base, ...replaceWords(t, true), title: t('common', 'dialog.savePolicy') };
    case 'saveDiagnostics':
      return { ...base, ...replaceWords(t), title: t('common', 'dialog.saveDiagnostics'), filter: t('common', 'dialog.filterExport') };
  }
}

/** `args` plus `dialog` when there are words to send; unchanged otherwise. */
export function withDialog(
  kind: DialogKind,
  args?: Record<string, unknown>,
): Record<string, unknown> | undefined {
  const words = dialogWords(kind);
  if (words === undefined) {
    return args;
  }
  return { ...(args ?? {}), dialog: words };
}
