-- R5 / D3: the pinned baseline, a measurement reference one project keeps for Compare
-- (`03_DESIGN/08_POST_MVP_PRESENTATION_RULES.md` §8, ADR-0034 D7, `06_DELIVERY/19_R5_HISTORY.md`).
--
-- Additive only. No earlier table is touched, renamed or re-keyed and no existing row changes, so a store
-- upgraded to this version simply has no pin until a person sets one.
--
-- The table is a log, not a slot. Pinning and unpinning each append one row and nothing is ever edited or
-- deleted, so "which build was the reference on that day" stays answerable and an unpin leaves its own
-- record. The current pin of a project is its newest row when that row is a `pin`; a newest `unpin`, or no
-- row at all, means the project has no pinned baseline.
--
-- A pin names a build, never a path or a file (AGENTS.md §7). Two constraints keep a row meaningful:
--
-- * a `pin` names a build and an `unpin` names none, so a row cannot say "unpinned build X";
-- * `seq` is the append order, so "newest" is never decided by two timestamps that tie in one second.

CREATE TABLE baseline_pin_events (
    seq         INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  TEXT NOT NULL REFERENCES projects(id),
    action      TEXT NOT NULL CHECK (action IN ('pin','unpin')),
    build_id    TEXT REFERENCES builds(id),
    recorded_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    CHECK ((action = 'pin' AND build_id IS NOT NULL) OR (action = 'unpin' AND build_id IS NULL))
);

CREATE INDEX idx_baseline_pin_events_project ON baseline_pin_events(project_id, seq);

-- A pin stays inside its project: the build it names must belong to the same project (R5 review S3). The
-- storage API checks this too; the schema says it so no other writer can skip it.
CREATE TRIGGER baseline_pin_events_stay_in_their_project
BEFORE INSERT ON baseline_pin_events
WHEN NEW.build_id IS NOT NULL
 AND (SELECT project_id FROM builds WHERE id = NEW.build_id) IS NOT NEW.project_id
BEGIN
    SELECT RAISE(ABORT, 'a baseline pin must name a build of the same project');
END;

CREATE TRIGGER baseline_pin_events_cannot_be_edited
BEFORE UPDATE ON baseline_pin_events
BEGIN
    SELECT RAISE(ABORT, 'a baseline pin event is a record and cannot be edited; pin or unpin again');
END;

CREATE TRIGGER baseline_pin_events_cannot_be_deleted
BEFORE DELETE ON baseline_pin_events
BEGIN
    SELECT RAISE(ABORT, 'a baseline pin event is a record and cannot be deleted; pin or unpin again');
END;
