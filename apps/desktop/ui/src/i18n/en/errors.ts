export const errors = {
  /**
   * One sentence per stable code, in the interface's own voice.
   *
   * Rust owns the message and the remediation for a code — the same wording the CLI prints, by
   * `crates/firmwaresight-artifact/src/error.rs`'s own rule that one code has one sentence — and the
   * panel shows both of them unchanged. What lives here is the line a reader who does not work in
   * English can act on: what this code means, said once, keyed by the code and never derived from
   * the engine's sentence at runtime.
   *
   * Where one code covers more than one condition (`ERR-INPUT-0001` is both a path that no longer
   * exists and a session handle that was never issued), the sentence is written for the fact they
   * share, because the engine's own line beside it carries the difference. Where no code is listed,
   * the panel says that plainly rather than guessing.
   */
  'unknown': 'FirmwareSight reported this code. The technical detail below is its own words.',
  'ERR-INPUT-0001': 'The file this window was pointing at is not there to read.',
  'ERR-INPUT-0002': 'The artifact could not be read.',
  'ERR-GUARD-0001': 'The artifact is larger than the full-buffer limit this build is configured to hold.',
  'ERR-FORMAT-0001': 'That file is not a format FirmwareSight can analyze yet.',
  'ERR-PARSE-2001': 'The artifact is malformed, so it was not read as a whole.',
  'ERR-PARSE-2002': 'The artifact could not be parsed as ELF.',
  'ERR-MAP-3001': 'That MAP file is not from a linker this build reads.',
  'ERR-MAP-3002': 'The MAP file could not be parsed.',
  'ERR-DIFF-5001': 'Both sides of the comparison name the same build, so there is nothing to compare.',
  'ERR-DIFF-5002': 'That comparison is no longer held by this session.',
  'ERR-EXPORT-6001': 'The analysis is sound, but the file you asked to write could not be written.',
  'ERR-EXPORT-6002': 'The result on screen changed before the file was written, so nothing was exported.',
  'ERR-STORAGE-4001': "FirmwareSight's own data store could not be opened.",
  'ERR-STORAGE-4002': 'The data store opened, but could not be prepared for use.',
  'ERR-STORAGE-4003': 'A migration of the data store failed, so nothing was read or written.',
  'ERR-STORAGE-4004': 'The data store is at a schema version this build does not understand.',
  'ERR-STORAGE-4005': 'The record this page asked for is not in the data store.',
  'ERR-STORAGE-4006': 'The data store refused the write.',
  'ERR-STORAGE-4007': 'That finding is not part of the stored run.',
  'ERR-STORAGE-4008': 'Only a REVIEW finding can be accepted, and this one is not REVIEW.',
  'ERR-STORAGE-4009': 'That finding already carries an acceptance.',
  'ERR-STORAGE-4010': 'A review acceptance has to name who accepted it and why.',
  'ERR-STORAGE-4011': 'The snapshot the store owed before an upgrade could not be written.',
  'ERR-STORAGE-4012': 'The store could not answer a health question.',
  'ERR-CONFIG-7001': 'The project configuration could not be read.',
  'ERR-CONFIG-7002': 'The project configuration is not valid TOML.',
  'ERR-CONFIG-7003': 'That configuration has a schema version this build does not read.',
  'ERR-CONFIG-7004': 'A configuration value is not one this build accepts.',
  'ERR-CONFIG-7005': 'A configured path points outside the project folder, so it is not followed.',
  'ERR-CONFIG-7006': 'The version pattern in the configuration cannot be used.',
  'ERR-CONFIG-7007': 'The save was refused, because the file holds keys this build does not understand.',
  'ERR-CONFIG-7008': 'The project configuration could not be saved.',
  'ERR-GIT-8001': 'Git workspace facts could not be read.',
  'ERR-BUNDLE-6101': 'This release is not a PASS, so no bundle is written for it.',
  'ERR-BUNDLE-6102': 'The Gate run selected is not the one this project context computes now.',
  'ERR-BUNDLE-6103': 'A source file is no longer the bytes the build recorded.',
  'ERR-BUNDLE-6104': 'The Release Notes are not the bytes the Gate observed.',
  'ERR-BUNDLE-6105': 'That bundle plan is no longer held by this session.',
  'ERR-BUNDLE-6106': 'The chosen folder already holds a bundle under that name.',
  'ERR-BUNDLE-6107': 'The folder that exists there is not a FirmwareSight bundle, so it is not replaced.',
  'ERR-BUNDLE-6108': 'A file could not be written into the bundle.',
  'ERR-BUNDLE-6109': 'What was written does not match what the bundle records about itself.',
  'ERR-BUNDLE-6110': 'The release version could not be resolved from the policy.',
  'ERR-BUNDLE-6111': 'The bundle was written, but its record could not be stored.',
  'ERR-BUNDLE-6112': 'No project policy is loaded, so there is nothing to release under.',
  'ERR-BUNDLE-6113': 'That destination was not chosen in this session.',
  'ERR-BUNDLE-6114': 'That build was not analyzed in this session, so it cannot be packaged.',
  'ERR-BUNDLE-6115': 'A release attachment could not be read.',
  'ERR-BUNDLE-6116': 'That file is not one an attachment may declare.',
  'ERR-BUNDLE-6117': 'An attachment has to hold bytes.',
  'ERR-BUNDLE-6118': 'The attachment set changed after the preview.',
  'ERR-PREFS-8001': 'This application could not reach its own settings folder, so nothing was stored.',
  'ERR-PREFS-8002': 'The settings file could not be written.',
  'ERR-PREFS-8003': 'The settings file could not be replaced.',
  'ERR-PREFS-8004': 'A newer version of FirmwareSight wrote these settings, so this version will not overwrite them.',
  'ERR-PREFS-8005': 'One of the chosen words is not an option this version offers, so nothing was stored.',
  'ERR-INTERNAL-9001': 'FirmwareSight hit an internal error.',
  'ERR-INTERNAL-9002': 'A structural expectation of the data store was violated.',
  'ERR-INTERNAL-9003': 'A stored record names something this build does not understand.',
  'ERR-INTERNAL-9004': 'FirmwareSight hit an internal error while composing the bundle.',
} as const;
