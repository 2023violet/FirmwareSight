use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use firmwaresight_project::verify_bundle;

const CONFIG: &str = r#"schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "bin", "hex"]

[release]
release_notes_path = "RELEASE_NOTES.md"
"#;

fn project(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "fwsight-doctor-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).expect("create isolated fixture");
    path
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("apps/cli is <repo>/apps/cli")
        .to_path_buf()
}

fn files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(base: &Path, current: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in std::fs::read_dir(current).expect("read fixture") {
            let path = entry.expect("fixture entry").path();
            if path.is_dir() {
                walk(base, &path, files);
            } else {
                files.push((
                    path.strip_prefix(base).expect("relative").to_path_buf(),
                    std::fs::read(path).expect("read bytes"),
                ));
            }
        }
    }
    let mut result = Vec::new();
    walk(root, root, &mut result);
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fwsight"))
        .args(args)
        .output()
        .expect("run fwsight")
}

fn git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git is available for the temporary fixture");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn doctor_reports_only_observations_and_does_not_write_project_files() {
    let dir = project("valid");
    std::fs::write(dir.join("firmwaresight.toml"), CONFIG).expect("write config");
    std::fs::write(dir.join("RELEASE_NOTES.md"), "Release notes\n").expect("write notes");
    let before = files(&dir);

    let output = run(&[
        "project",
        "doctor",
        dir.to_str().expect("test path is UTF-8"),
    ]);
    let stdout = String::from_utf8(output.stdout).expect("doctor output is UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("doctor diagnostics are UTF-8");

    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert!(stdout.contains("Project readiness"));
    assert!(stdout.contains("motor-controller"));
    assert!(stdout.contains("declared by policy"));
    assert!(stdout.contains("bin"));
    assert!(stdout.contains("hex"));
    assert!(stdout.contains("not checked"));
    assert!(stdout.contains("not a Gate result"));
    assert!(stdout.contains("Unknown"));
    assert!(!stdout.contains(dir.to_str().expect("path")));
    assert!(stderr.is_empty());
    assert_eq!(before, files(&dir));
    std::fs::remove_dir_all(dir).expect("remove isolated fixture");
}

#[test]
fn doctor_config_refusal_uses_exit_two_without_echoing_path_or_config_contents() {
    let dir = project("bad-config");
    std::fs::write(
        dir.join("firmwaresight.toml"),
        "schema_version = 2\nsecret_value = \"sentinel-secret\"\n",
    )
    .expect("write invalid config");
    let output = run(&[
        "project",
        "doctor",
        dir.to_str().expect("test path is UTF-8"),
    ]);
    let stderr = String::from_utf8(output.stderr).expect("diagnostic is UTF-8");

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("ERR-CONFIG"));
    assert!(!stderr.contains(dir.to_str().expect("path")));
    assert!(!stderr.contains("sentinel-secret"));
    std::fs::remove_dir_all(dir).expect("remove isolated fixture");
}

#[test]
fn doctor_missing_and_unsupported_configs_use_exit_two_without_echoing_paths() {
    let dir = project("missing-config");
    let path = dir.to_str().expect("test path is UTF-8");
    let missing = run(&["project", "doctor", path]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(!String::from_utf8_lossy(&missing.stderr).contains(path));

    std::fs::write(dir.join("firmwaresight.toml"), "schema_version = 99\n")
        .expect("write unsupported config");
    let unsupported = run(&["project", "doctor", path]);
    assert_eq!(unsupported.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unsupported.stderr).contains("ERR-CONFIG-7003"));
    assert!(!String::from_utf8_lossy(&unsupported.stderr).contains(path));
    std::fs::remove_dir_all(dir).expect("remove isolated fixture");
}

#[test]
fn doctor_refuses_release_notes_that_escape_the_project_without_disclosing_contents() {
    let parent = project("notes-boundary");
    let dir = parent.join("project");
    std::fs::create_dir(&dir).expect("create nested project");
    std::fs::write(parent.join("outside.md"), "outside-canary-secret\n")
        .expect("write outside canary");
    let config = CONFIG.replace("RELEASE_NOTES.md", "../outside.md");
    std::fs::write(dir.join("firmwaresight.toml"), config).expect("write unsafe config");

    let path = dir.to_str().expect("test path is UTF-8");
    let output = run(&["project", "doctor", path]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("ERR-CONFIG-7005"));
    assert!(!stderr.contains("outside-canary-secret"));
    assert!(!stderr.contains(path));
    std::fs::remove_dir_all(parent).expect("remove isolated fixture");
}

#[test]
fn doctor_requires_an_explicit_project_directory() {
    let output = run(&["project", "doctor"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn prior_json_entry_points_remain_available() {
    let help = run(&["--help"]);
    assert_eq!(help.status.code(), Some(0));
    let stdout = String::from_utf8(help.stdout).expect("help UTF-8");
    for verb in ["analyze", "diff", "gate", "release", "project"] {
        assert!(stdout.contains(verb), "missing {verb} from CLI help");
    }
}

#[test]
fn disposable_fixture_workflow_runs_doctor_analyze_gate_and_release() {
    let world = project("workflow");
    let root = repo_root();
    let example = root.join("examples/project-readiness");
    for name in ["firmwaresight.toml", "RELEASE_NOTES.md"] {
        std::fs::copy(example.join(name), world.join(name)).expect("copy committed workflow input");
    }
    git(&world, &["init", "--quiet"]);
    git(&world, &["config", "user.name", "FirmwareSight Test"]);
    git(&world, &["config", "user.email", "test@example.invalid"]);
    git(&world, &["add", "firmwaresight.toml", "RELEASE_NOTES.md"]);
    git(&world, &["commit", "--quiet", "-m", "workflow fixture"]);
    git(&world, &["tag", "v1.0.0"]);
    let artifact = root.join("fixtures/elf/p2-diff/target/firmware.elf");
    let map = root.join("fixtures/elf/p2-diff/target/firmware.map");
    let artifact = artifact.to_str().expect("fixture path is UTF-8");
    let map = map.to_str().expect("fixture path is UTF-8");
    let dir = world.to_str().expect("temp path is UTF-8");

    let doctor = run(&["project", "doctor", dir]);
    assert_eq!(
        doctor.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&doctor.stderr)
    );
    assert!(String::from_utf8_lossy(&doctor.stdout).contains("not a Gate result"));
    assert!(String::from_utf8_lossy(&doctor.stdout).contains("declared by policy"));

    let analyze = run(&["analyze", artifact, "--map", map]);
    assert_eq!(
        analyze.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&analyze.stderr)
    );

    let gate = run(&[
        "gate",
        "--project",
        dir,
        "--artifact",
        artifact,
        "--map",
        map,
    ]);
    assert_eq!(
        gate.status.code(),
        Some(0),
        "{}\n{}",
        String::from_utf8_lossy(&gate.stdout),
        String::from_utf8_lossy(&gate.stderr)
    );
    assert!(String::from_utf8_lossy(&gate.stdout).contains("PASS"));

    let out = world.join("release-output");
    let out = out.to_str().expect("temp output path is UTF-8");
    let release = run(&[
        "release",
        "prepare",
        "--project",
        dir,
        "--artifact",
        artifact,
        "--map",
        map,
        "--out",
        out,
    ]);
    assert_eq!(
        release.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&release.stderr)
    );
    let bundle = std::fs::read_dir(world.join("release-output"))
        .expect("release output exists")
        .next()
        .expect("one bundle was created")
        .expect("bundle entry")
        .path();
    let verified = verify_bundle(&bundle).expect("Release Bundle self-verifies");
    assert_eq!(verified.artifact_count, 2);
    assert!(verified.carries_release_notes);
    assert!(bundle.join("release-manifest.json").is_file());
    std::fs::remove_dir_all(world).expect("remove disposable workflow");
}
