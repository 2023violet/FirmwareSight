/**
 * The product bar: who this is, which project this workspace is holding, and the promise that never
 * changes.
 *
 * Every reference screen opens with this row and the shell had no equivalent: the brand lived at the top
 * of the navigation rail and the local-first statement lived one click away in Help
 * (`Help.tsx:230`), so the most distinctive promise of the product was the least visible one on screen.
 * Putting it in the bar is the cheapest real convergence in this round: no layout moves, and the sentence
 * stops being one click away from the work it describes.
 *
 * The middle chip is the honest version of the mockups' `relay-controller · v0.3.0-rc2`. Those values are
 * declared sample data by `03_DESIGN/06_UI_REFERENCE_SCREENS.md`, so nothing here prints them. The chip
 * names the project whose policy is loaded and says nothing when none is, for two reasons: the reference
 * bar shows a project and a version rather than a file, and repeating the selected artifact's name one
 * row above the row that owns it would put the same string on the screen twice with two meanings - the
 * bar's "this is the workspace" and the page's "this is the file I am about to analyze".
 *
 * The reference bar also ends with the application version. This one does not: printing it costs a second
 * `get_app_identity` read every session on top of the one Help already makes, and Help and About remain
 * where the version is stated in full. A bar should not fetch a fact to decorate itself.
 */

import styles from './TopBar.module.css';

export function TopBar({ project }: { readonly project: string | null }) {
  return (
    <header className={styles['bar']}>
      <span className={styles['brand']}>FirmwareSight</span>
      {project === null ? null : (
        <span className={styles['context']} title={project}>
          {project}
        </span>
      )}
      <p className={styles['promise']}>Local workspace — nothing leaves this machine</p>
    </header>
  );
}
