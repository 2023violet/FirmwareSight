/**
 * Join CSS-module class names.
 *
 * The project compiles with `noUncheckedIndexedAccess`, and a CSS-module import is an index
 * signature, so every lookup is `string | undefined` - that is correct, because a typo'd class
 * really can be missing. This folds the names without a non-null assertion, so a mistake shows
 * up as a class that silently does nothing rather than as a cast that hides it.
 */
export function cx(...names: ReadonlyArray<string | undefined>): string {
  return names.filter((name): name is string => name !== undefined).join(' ');
}
