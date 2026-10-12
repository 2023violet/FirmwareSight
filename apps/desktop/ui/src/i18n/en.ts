/**
 * The English catalog, which is the key authority.
 *
 * `ADR-0032` D2 makes this the thing the Chinese catalog is typed against: every key that exists in a
 * namespace here must exist in the same namespace there, and a key that exists only there is an error
 * too. That is what a missing translation is supposed to mean in this product - the build does not
 * compile - rather than a runtime log nobody reads. `pnpm typecheck` is already gate step 5, so the
 * compiler reaches that verdict before any test does.
 *
 * The words live one file per namespace under `./en/`, with the matching Chinese under `./zh/`, so a
 * page's translation is a pair of files a reviewer can read side by side instead of one growing
 * document that two people cannot edit at once. What does not move is the rule the fragments are
 * typed against: this object's shape is the contract.
 *
 * Keys are dotted, lowercase and semantic (`settings.language.heading`), never the English sentence
 * itself. A sentence as a key turns a copy edit into a schema change and makes the second language
 * inherit the first one's wording by construction.
 *
 * What is deliberately absent: the five Gate states and the four evidence classes. Those are Core's
 * vocabulary, rendered from Core's words, and `ADR-0032` D5 forbids renaming them in either language.
 */

import { common } from './en/common';
import { analyze } from './en/analyze';
import { compare } from './en/compare';
import { gate } from './en/gate';
import { release } from './en/release';
import { history } from './en/history';
import { settings } from './en/settings';
import { help } from './en/help';
import { errors } from './en/errors';

export const en = { common, analyze, compare, gate, release, history, settings, help, errors } as const;

/** The shape every catalog must have: the namespaces, the keys, and nothing else. */
export type Catalog = typeof en;
export type Namespace = keyof Catalog;
