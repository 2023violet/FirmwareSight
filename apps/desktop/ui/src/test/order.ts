/**
 * Document order, for the tests that are about hierarchy rather than about content.
 *
 * U1P §21 asks for structural tests that say which region a reader meets first, and it forbids
 * coordinate assertions. `compareDocumentPosition` is the DOM's own answer to "which comes first", so a
 * page can be pinned on its reading order without a single pixel being measured.
 *
 * Containment is deliberately excluded: a region that holds another one is not *before* it, and a test
 * that let that pass would be satisfied by nesting rather than by ordering.
 */
export function precedes(before: Element, after: Element): boolean {
  if (before === after || before.contains(after) || after.contains(before)) {
    return false;
  }
  return (before.compareDocumentPosition(after) & Node.DOCUMENT_POSITION_FOLLOWING) !== 0;
}
