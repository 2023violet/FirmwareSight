use firmwaresight_core::domain::gate::GateFileStatus;
use firmwaresight_project::observe_release_notes;
use std::fs;
#[cfg(windows)]
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "firmwaresight-{label}-{}-{nonce}",
        std::process::id()
    ))
}

#[cfg(windows)]
fn create_junction(link: &std::path::Path, target: &std::path::Path) {
    let output = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()
        .expect("run Windows Junction creation command");
    assert!(
        output.status.success(),
        "mklink /J failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(windows)]
fn remove_junction_when_available(link: &std::path::Path) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        match fs::remove_dir(link) {
            Ok(()) => return,
            Err(error) if error.raw_os_error() == Some(32) => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "open-handle Junction remained non-deletable beyond the test bound"
                );
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Err(error) => panic!("remove Junction itself without traversing target: {error}"),
        }
    }
}

#[cfg(windows)]
#[test]
fn release_notes_junction_cannot_read_an_outside_canary() {
    let root = temp_root("junction-root");
    let outside = temp_root("junction-outside");
    let junction = root.join("docs");
    fs::create_dir_all(&root).expect("create synthetic project");
    fs::create_dir_all(&outside).expect("create outside canary directory");
    let canary = b"external release notes canary\n";
    fs::write(outside.join("RELEASE_NOTES.md"), canary).expect("write outside canary");

    create_junction(&junction, &outside);

    let fact = observe_release_notes(&root, "docs/RELEASE_NOTES.md");
    fs::remove_dir(&junction).expect("remove Junction itself without traversing its target");
    fs::remove_dir_all(&root).expect("remove synthetic project");
    fs::remove_dir_all(&outside).expect("remove synthetic canary directory");

    assert!(
        matches!(fact.status, GateFileStatus::Unreadable { .. }),
        "external Junction must be unreadable, got {:?}",
        fact.status
    );
}

#[cfg(windows)]
#[test]
fn swapping_a_junction_never_exposes_the_outside_canary_digest() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    let root = temp_root("swap-root");
    let inside = temp_root("swap-inside");
    let outside = temp_root("swap-outside");
    let junction = root.join("docs");
    fs::create_dir_all(&root).expect("create synthetic project");
    fs::create_dir_all(&inside).expect("create inside target");
    fs::create_dir_all(&outside).expect("create outside target");
    fs::write(inside.join("RELEASE_NOTES.md"), b"inside release notes\n")
        .expect("write inside notes");
    let outside_bytes = b"outside release notes canary\n";
    fs::write(outside.join("RELEASE_NOTES.md"), outside_bytes).expect("write outside canary");
    create_junction(&junction, &inside);

    let running = Arc::new(AtomicBool::new(true));
    let outside_digest = firmwaresight_project::fingerprint::sha256_hex(outside_bytes);
    let observed = Arc::new(Mutex::new(Vec::new()));
    let reader_root = root.clone();
    let reader_running = Arc::clone(&running);
    let reader_observed = Arc::clone(&observed);
    let reader = std::thread::spawn(move || {
        while reader_running.load(Ordering::Relaxed) {
            let fact = observe_release_notes(&reader_root, "docs/RELEASE_NOTES.md");
            if let GateFileStatus::Present {
                sha256: Some(digest),
            } = fact.status
            {
                reader_observed.lock().expect("result lock").push(digest);
            }
        }
    });

    for iteration in 0..40 {
        remove_junction_when_available(&junction);
        create_junction(
            &junction,
            if iteration % 2 == 0 {
                &outside
            } else {
                &inside
            },
        );
    }
    running.store(false, Ordering::Relaxed);
    reader.join().expect("reader thread completes");
    let observed = Arc::try_unwrap(observed)
        .expect("reader released result handle")
        .into_inner()
        .expect("result lock");

    remove_junction_when_available(&junction);
    fs::remove_dir_all(&root).expect("remove synthetic project");
    fs::remove_dir_all(&inside).expect("remove inside target");
    fs::remove_dir_all(&outside).expect("remove outside canary");

    assert!(
        !observed.iter().any(|digest| digest == &outside_digest),
        "outside canary digest appeared during Junction replacement"
    );
}

#[test]
fn ordinary_nested_release_notes_keep_their_known_digest() {
    let root = temp_root("nested-notes");
    fs::create_dir_all(root.join("docs")).expect("create nested project directory");
    let bytes = b"ordinary nested release notes\n";
    fs::write(root.join("docs/RELEASE_NOTES.md"), bytes).expect("write nested notes");

    let fact = observe_release_notes(&root, "docs/RELEASE_NOTES.md");

    fs::remove_dir_all(&root).expect("remove synthetic project");
    assert!(matches!(
        fact.status,
        GateFileStatus::Present { sha256: Some(digest) }
            if digest == firmwaresight_project::fingerprint::sha256_hex(bytes)
    ));
}

#[test]
fn absent_release_notes_remain_missing() {
    let root = temp_root("missing-notes");
    fs::create_dir_all(&root).expect("create synthetic project");

    let fact = observe_release_notes(&root, "docs/RELEASE_NOTES.md");

    fs::remove_dir_all(&root).expect("remove synthetic project");
    assert_eq!(fact.status, GateFileStatus::Missing);
}

#[test]
fn a_directory_is_unreadable_release_notes() {
    let root = temp_root("directory-notes");
    fs::create_dir_all(root.join("docs/RELEASE_NOTES.md"))
        .expect("create directory at configured notes path");

    let fact = observe_release_notes(&root, "docs/RELEASE_NOTES.md");

    fs::remove_dir_all(&root).expect("remove synthetic project");
    assert!(matches!(fact.status, GateFileStatus::Unreadable { .. }));
}

#[test]
fn rooted_and_volume_paths_are_refused_without_disclosing_the_root() {
    let root = temp_root("invalid-path");
    fs::create_dir_all(&root).expect("create synthetic project");

    for path in [
        "/outside/RELEASE_NOTES.md",
        "C:/outside/RELEASE_NOTES.md",
        "//server/share/RELEASE_NOTES.md",
    ] {
        let fact = observe_release_notes(&root, path);
        match fact.status {
            GateFileStatus::Unreadable { reason } => {
                assert!(!reason.contains(&root.to_string_lossy().to_string()));
                assert!(!reason.contains(path));
            }
            status => panic!("absolute path was not refused: {status:?}"),
        }
    }

    fs::remove_dir_all(&root).expect("remove synthetic project");
}

#[cfg(windows)]
#[test]
fn a_windows_device_name_is_not_read_as_release_notes() {
    let root = temp_root("device-name");
    fs::create_dir_all(&root).expect("create synthetic project");

    let fact = observe_release_notes(&root, "NUL");

    fs::remove_dir_all(&root).expect("remove synthetic project");
    assert!(
        !matches!(fact.status, GateFileStatus::Present { .. }),
        "Windows device path must not be observed as a regular file"
    );
}

#[cfg(windows)]
#[test]
fn a_windows_file_symlink_to_an_outside_release_notes_file_is_refused_when_available() {
    use std::os::windows::fs::symlink_file;

    let root = temp_root("file-symlink-root");
    let outside = temp_root("file-symlink-outside");
    fs::create_dir_all(root.join("docs")).expect("create synthetic project");
    fs::create_dir_all(&outside).expect("create outside canary directory");
    fs::write(
        outside.join("RELEASE_NOTES.md"),
        b"external file-link canary\n",
    )
    .expect("write outside canary");
    let link = root.join("docs/RELEASE_NOTES.md");
    if let Err(error) = symlink_file(outside.join("RELEASE_NOTES.md"), &link) {
        eprintln!("NOT_VERIFIED: Windows file symlink unavailable: {error}");
        fs::remove_dir_all(&root).expect("remove synthetic project");
        fs::remove_dir_all(&outside).expect("remove outside canary directory");
        return;
    }

    let fact = observe_release_notes(&root, "docs/RELEASE_NOTES.md");

    fs::remove_dir_all(&root).expect("remove synthetic project");
    fs::remove_dir_all(&outside).expect("remove outside canary directory");
    assert!(matches!(fact.status, GateFileStatus::Unreadable { .. }));
}

#[cfg(unix)]
#[test]
fn a_symlink_to_an_outside_release_notes_file_is_unreadable() {
    use std::os::unix::fs::symlink;

    let root = temp_root("symlink-root");
    let outside = temp_root("symlink-outside");
    fs::create_dir_all(root.join("docs")).expect("create synthetic project");
    fs::create_dir_all(&outside).expect("create outside canary directory");
    fs::write(
        outside.join("RELEASE_NOTES.md"),
        b"external symlink canary\n",
    )
    .expect("write outside canary");
    symlink(
        outside.join("RELEASE_NOTES.md"),
        root.join("docs/RELEASE_NOTES.md"),
    )
    .expect("create outside symlink");

    let fact = observe_release_notes(&root, "docs/RELEASE_NOTES.md");

    fs::remove_dir_all(&root).expect("remove synthetic project");
    fs::remove_dir_all(&outside).expect("remove outside canary directory");
    assert!(matches!(fact.status, GateFileStatus::Unreadable { .. }));
}

#[cfg(unix)]
#[test]
fn a_relative_symlink_that_stays_inside_the_project_is_readable() {
    use std::os::unix::fs::symlink;

    let root = temp_root("internal-symlink");
    fs::create_dir_all(root.join("docs")).expect("create synthetic project");
    let bytes = b"internal symlink release notes\n";
    fs::write(root.join("RELEASE_NOTES.md"), bytes).expect("write release notes");
    symlink("../RELEASE_NOTES.md", root.join("docs/RELEASE_NOTES.md"))
        .expect("create relative internal symlink");

    let fact = observe_release_notes(&root, "docs/RELEASE_NOTES.md");

    fs::remove_dir_all(&root).expect("remove synthetic project");
    assert!(matches!(
        fact.status,
        GateFileStatus::Present { sha256: Some(digest) }
            if digest == firmwaresight_project::fingerprint::sha256_hex(bytes)
    ));
}

#[test]
fn a_project_entry_that_is_a_hard_link_remains_readable() {
    let root = temp_root("hardlink-root");
    let outside = temp_root("hardlink-outside");
    fs::create_dir_all(root.join("docs")).expect("create synthetic project");
    fs::create_dir_all(&outside).expect("create outside canary directory");
    let bytes = b"hard-linked release notes\n";
    let outside_file = outside.join("RELEASE_NOTES.md");
    fs::write(&outside_file, bytes).expect("write hard-link source");
    fs::hard_link(&outside_file, root.join("docs/RELEASE_NOTES.md"))
        .expect("create project hard-link entry");

    let fact = observe_release_notes(&root, "docs/RELEASE_NOTES.md");

    fs::remove_dir_all(&root).expect("remove synthetic project");
    fs::remove_dir_all(&outside).expect("remove outside hard-link source");
    assert!(matches!(
        fact.status,
        GateFileStatus::Present { sha256: Some(digest) }
            if digest == firmwaresight_project::fingerprint::sha256_hex(bytes)
    ));
}
