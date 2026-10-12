/**
 * The Simplified Chinese catalog.
 *
 * Every fragment here is typed against the English one of the same name, so a key added on one side
 * and not the other is a compile error (`ADR-0032` D2). That is the whole missing-translation story:
 * this catalog cannot be quietly incomplete.
 *
 * Wording rules, all of them `ADR-0032` D5 and product v0.2 §97:
 * - Machine vocabulary is not translated. `PASS` / `REVIEW` / `BLOCK` / `UNKNOWN` / `N/A` and
 *   `Observed` / `Derived` / `Declared` / `Unknown` appear in their engine spelling even inside a
 *   Chinese sentence, because translating them would rename a verdict.
 * - `{placeholder}` sets match English exactly; a parameter is inserted, never a sentence joined.
 * - No markup. A value is text and is rendered as text.
 * - `KiB`, hashes, paths, ids and numbers are never rewritten.
 * - 简体中文 only. There is no Traditional catalog, and a reader on a `zh-TW` desktop is resolved
 *   here rather than being promised something that does not exist.
 */

import { common } from './zh/common';
import { analyze } from './zh/analyze';
import { compare } from './zh/compare';
import { gate } from './zh/gate';
import { release } from './zh/release';
import { history } from './zh/history';
import { settings } from './zh/settings';
import { help } from './zh/help';
import { errors } from './zh/errors';

export const zh = { common, analyze, compare, gate, release, history, settings, help, errors };
