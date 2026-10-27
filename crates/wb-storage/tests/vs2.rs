use std::{
    fs,
    os::unix::fs::{symlink, PermissionsExt},
    path::Path,
};
use tempfile::TempDir;
use wb_storage::vs2::{FileStore, Workspace, MAX_BYTES};
fn write(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
}
#[test]
fn vs2_019_atomic_versioned_snapshot_and_cas() {
    let t = TempDir::new().unwrap();
    let path = t.path().join("workspace");
    let store = FileStore::open(&path).unwrap();
    assert!(store.load().unwrap().is_none());
    let first = store.commit(0, &Workspace::default()).unwrap();
    assert_eq!(first.revision, 1);
    let bytes = fs::read(path.join("workspace.json")).unwrap();
    assert!(store.commit(0, &first).is_err());
    assert_eq!(fs::read(path.join("workspace.json")).unwrap(), bytes);
    let second = store.commit(1, &first).unwrap();
    assert_eq!(second.revision, 2);
    drop(store);
    let reopened = FileStore::open(&path).unwrap();
    assert_eq!(reopened.load().unwrap().unwrap().revision, 2);
}
#[test]
fn vs2_020_corruption_is_non_destructive() {
    let t = TempDir::new().unwrap();
    let path = t.path().join("workspace");
    let store = FileStore::open(&path).unwrap();
    store.commit(0, &Workspace::default()).unwrap();
    let original = fs::read(path.join("workspace.json")).unwrap();
    for length in [0, 1, original.len() / 2, original.len() - 1] {
        write(&path.join("workspace.json"), &original[..length]);
        assert!(store.load().is_err());
        assert!(store.commit(0, &Workspace::default()).is_err());
        assert_eq!(
            fs::read(path.join("workspace.json")).unwrap(),
            original[..length]
        );
    }
    write(&path.join("workspace.json"), &original);
    let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
    value["revision"] = 2.into();
    write(&path.join("workspace.json"), value.to_string().as_bytes());
    assert!(matches!(store.load(), Err("SNAPSHOT_CHECKSUM")));
    value["formatVersion"] = 99.into();
    write(&path.join("workspace.json"), value.to_string().as_bytes());
    assert!(matches!(store.load(), Err("UNSUPPORTED_SNAPSHOT_VERSION")));
}
#[test]
fn vs2_021_size_duplicate_unknown_and_symlink_limits() {
    assert!(Workspace::decode(br#"{"formatVersion":1,"formatVersion":1}"#).is_err());
    let t = TempDir::new().unwrap();
    let root = t.path().join("workspace");
    let store = FileStore::open(&root).unwrap();
    write(&root.join("workspace.json"), &vec![b' '; MAX_BYTES + 1]);
    assert!(matches!(store.load(), Err("SNAPSHOT_QUOTA")));
    fs::remove_file(root.join("workspace.json")).unwrap();
    let outside = t.path().join("outside.json");
    write(&outside, b"preserve");
    symlink(&outside, root.join("workspace.json")).unwrap();
    assert!(store.load().is_err());
    assert!(store.commit(0, &Workspace::default()).is_err());
    assert_eq!(fs::read(&outside).unwrap(), b"preserve");
    let link = t.path().join("linked");
    symlink(&root, &link).unwrap();
    assert!(FileStore::open(&link).is_err());
}
#[test]
fn vs2_021_fifo_and_world_readable_paths_fail_without_blocking() {
    use std::{ffi::CString, os::unix::ffi::OsStrExt, time::Instant};
    let t = TempDir::new().unwrap();
    let root = t.path().join("workspace");
    let store = FileStore::open(&root).unwrap();
    let fifo = root.join("workspace.json");
    let c = CString::new(fifo.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
    let now = Instant::now();
    assert!(store.load().is_err());
    assert!(now.elapsed().as_secs() < 1);
    let public = t.path().join("public");
    fs::create_dir(&public).unwrap();
    fs::set_permissions(&public, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(FileStore::open(&public).is_err());
}
#[test]
fn vs2_019_exclusive_writer_and_anchored_directory() {
    let t = TempDir::new().unwrap();
    let root = t.path().join("workspace");
    let store = FileStore::open(&root).unwrap();
    assert!(matches!(FileStore::open(&root), Err("WORKSPACE_BUSY")));
    store.commit(0, &Workspace::default()).unwrap();
    let moved = t.path().join("moved");
    fs::rename(&root, &moved).unwrap();
    fs::create_dir(&root).unwrap();
    store.commit(1, &Workspace::default()).unwrap();
    assert!(!root.join("workspace.json").exists());
    assert_eq!(
        Workspace::decode(&fs::read(moved.join("workspace.json")).unwrap())
            .unwrap()
            .revision,
        2
    );
}

#[test]
fn vs2_021_runtime_snapshot_positive_and_negative_fixture_corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/vslice-002");
    for (folder, expected) in [("valid", true), ("invalid", false)] {
        let mut count = 0;
        for file in fs::read_dir(root.join(folder)).unwrap() {
            let path = file.unwrap().path();
            if path.extension().is_some_and(|v| v == "json") {
                count += 1;
                assert_eq!(
                    Workspace::decode(&fs::read(&path).unwrap()).is_ok(),
                    expected,
                    "{}",
                    path.display()
                );
            }
        }
        assert!(count > 0);
    }
}
