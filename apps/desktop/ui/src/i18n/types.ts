/**
 * The catalog's own types, in the one place that can name them without a cycle.
 *
 * `en.ts` composes the namespace fragments, so a fragment cannot import the whole catalog back into
 * itself — but it can import these, because this file only ever looks at `en`'s *types*.
 */

import type { Catalog, Namespace } from './en';

/** Every key of one namespace, with ordinary `string` values. */
export type CatalogFor<N extends Namespace> = { [K in keyof Catalog[N]]: string };

/** All nine namespaces, each with the keys its English fragment declares and nothing else. */
export type CatalogShape = { [N in Namespace]: CatalogFor<N> };
