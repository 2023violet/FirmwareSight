# Project Readiness Workflow

Run from the repository root in PowerShell. The project folder and generated bundle live under the system temporary directory. The config and notes are committed to a disposable Git repository so the example Gate can observe a clean workspace and the version tag it requires.

```powershell
$project = Join-Path $env:TEMP 'firmwaresight-readiness-fixture'
New-Item -ItemType Directory -Force $project | Out-Null
Copy-Item 'examples/project-readiness/*' $project
git -C $project init --quiet
git -C $project config user.name 'FirmwareSight Example'
git -C $project config user.email 'example@example.invalid'
git -C $project add firmwaresight.toml RELEASE_NOTES.md
git -C $project commit --quiet -m 'readiness fixture'
git -C $project tag v1.0.0

cargo run -q -p fwsight -- project doctor $project
cargo run -q -p fwsight -- analyze fixtures/elf/p2-diff/target/firmware.elf --map fixtures/elf/p2-diff/target/firmware.map
cargo run -q -p fwsight -- gate --project $project --artifact fixtures/elf/p2-diff/target/firmware.elf --map fixtures/elf/p2-diff/target/firmware.map
cargo run -q -p fwsight -- release prepare --project $project --artifact fixtures/elf/p2-diff/target/firmware.elf --map fixtures/elf/p2-diff/target/firmware.map --out (Join-Path $project 'dist')
```

Doctor reports configuration and workspace observations only. It does not inspect the selected firmware, prove build provenance, evaluate Gate rules, or claim release readiness. Gate is the policy decision; Release repeats that decision before writing the bundle. `fwsight release prepare` leaves its output in the selected temporary project and does not create a database record.
