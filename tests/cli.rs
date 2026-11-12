//! Integration tests. Each one uses a real temporary folder.
//!
//! `CARGO_BIN_EXE_rmbin` is the compiled `rmbin` binary. The library tests in
//! `src/` check the pattern maths. These tests check what a person sees.

#![forbid(unsafe_code)]

use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

struct Temp {
    path: PathBuf,
}

impl Temp {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("rmbin-cli-{n}-{nanos}"));
        fs::create_dir_all(&path).expect("temp");
        let path = fs::canonicalize(&path).unwrap_or(path);
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        relax(&self.path);
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn relax(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if path.is_dir() { 0o755 } else { 0o644 };
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
        if let Ok(meta) = fs::symlink_metadata(path) {
            if meta.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        relax(&entry.path());
                    }
                }
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

fn run(dir: &Path, args: &[&str], stdin_bytes: Option<&[u8]>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rmbin"));
    command
        .current_dir(dir)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match stdin_bytes {
        Some(bytes) => {
            command.stdin(Stdio::piped());
            let mut child = command.spawn().expect("spawn rmbin");
            // This version does not read stdin. The child can exit before the
            // write finishes, which is a broken pipe, not a failed run.
            let write_result = child.stdin.as_mut().expect("stdin").write_all(bytes);
            if let Err(error) = write_result {
                if error.kind() != std::io::ErrorKind::BrokenPipe {
                    panic!("write stdin: {error}");
                }
            }
            child.wait_with_output().expect("wait")
        }
        None => {
            command.stdin(Stdio::null());
            command.output().expect("run rmbin")
        }
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn combined(output: &Output) -> String {
    format!("{}{}", text(&output.stdout), text(&output.stderr))
}

#[test]
fn help_and_version() {
    let dir = Temp::new();
    let help = run(dir.path(), &["--help"], None);
    assert!(help.status.success(), "{}", text(&help.stderr));
    let help_text = text(&help.stdout);
    assert!(help_text.contains("--pattern"));
    assert!(help_text.contains("--dry-run"));
    assert!(help_text.contains("--list"));
    assert!(help_text.contains("--yes"));
    assert!(help_text.contains("best-effort"));
    assert!(help_text.contains("DISCLAIMER.md"));
    assert!(help_text.contains("No warranty"));
    assert!(help_text.contains("Not in this version"));
    assert!(help_text.contains("does not overwrite"));

    let version = run(dir.path(), &["--version"], None);
    assert!(version.status.success());
    assert!(text(&version.stdout).contains("rmbin 0.0.9"));
}

#[test]
fn unknown_recycle_flag_deletes_nothing() {
    let dir = Temp::new();
    let file = dir.path().join("notes.txt");
    fs::write(&file, b"secret").expect("write");
    let output = run(
        dir.path(),
        &["--recycle", file.to_str().expect("utf8")],
        None,
    );
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read(&file).expect("still there"), b"secret");
}

#[test]
fn dry_run_changes_nothing_and_prints_the_banner() {
    let dir = Temp::new();
    let file = dir.path().join("notes.txt");
    fs::write(&file, b"secret-bytes").expect("write");
    let output = run(
        dir.path(),
        &["--dry-run", "--yes", file.to_str().expect("utf8")],
        None,
    );
    assert!(output.status.success(), "{}", combined(&output));
    let stdout = text(&output.stdout);
    assert!(stdout.contains("best-effort overwrite"));
    assert!(stdout.contains("APFS"));
    assert!(stdout.contains("Plan. Nothing has been changed yet."));
    assert!(stdout.contains("Pattern: 'lol'"));
    assert!(stdout.contains("Dry run: nothing was changed."));
    assert!(stdout.contains("overwrite 12 bytes"));
    assert_eq!(fs::read(&file).expect("read"), b"secret-bytes");
}

#[test]
fn yes_does_not_delete() {
    let dir = Temp::new();
    let kept = dir.path().join("kept.txt");
    let named = dir.path().join("gone.txt");
    fs::write(&kept, b"stay").expect("write");
    fs::write(&named, b"go").expect("write");

    let declined = run(dir.path(), &[kept.to_str().expect("utf8")], Some(b"no\n"));
    assert!(!declined.status.success());
    assert_eq!(declined.status.code(), Some(2));
    assert!(combined(&declined).contains("Nothing was changed."));
    assert!(combined(&declined).contains("does not delete"));
    assert_eq!(fs::read(&kept).expect("kept"), b"stay");

    let empty = run(dir.path(), &[kept.to_str().expect("utf8")], None);
    assert_eq!(empty.status.code(), Some(2));
    assert_eq!(fs::read(&kept).expect("kept"), b"stay");

    let accepted = run(dir.path(), &[named.to_str().expect("utf8")], Some(b"yes\n"));
    assert_eq!(accepted.status.code(), Some(2), "{}", combined(&accepted));
    assert!(combined(&accepted).contains("does not delete"));
    assert!(!combined(&accepted).contains("securely deleted file"));
    assert_eq!(fs::read(&named).expect("still there"), b"go");
    assert!(kept.exists());
}

#[test]
fn yes_flag_is_ignored_and_the_file_stays() {
    let dir = Temp::new();
    let file = dir.path().join("notes.txt");
    fs::write(&file, b"go").expect("write");
    let output = run(dir.path(), &["--yes", file.to_str().expect("utf8")], None);
    assert_eq!(output.status.code(), Some(2), "{}", combined(&output));
    assert_eq!(fs::read(&file).expect("kept"), b"go");
    let stdout = text(&output.stdout);
    assert!(stdout.contains("Ignoring --yes"));
    assert!(stdout.contains("best-effort"));
    assert!(stdout.contains("Nothing was changed."));
}

#[test]
fn list_is_split_and_positional_names_are_not() {
    let dir = Temp::new();
    fs::write(dir.path().join("a.txt"), b"a").expect("write");
    fs::write(dir.path().join("b.txt"), b"b").expect("write");
    let odd = dir.path().join("odd,name.txt");
    fs::write(&odd, b"odd").expect("write");

    let listed = run(dir.path(), &["--dry-run", "--list", "a.txt;b.txt"], None);
    assert!(listed.status.success(), "{}", combined(&listed));
    let listed_text = text(&listed.stdout);
    assert!(listed_text.contains("a.txt"));
    assert!(listed_text.contains("b.txt"));
    assert!(dir.path().join("a.txt").exists());
    assert!(dir.path().join("b.txt").exists());
    assert_eq!(fs::read(&odd).expect("odd"), b"odd");

    let positional = run(dir.path(), &["--dry-run", "odd,name.txt"], None);
    assert!(positional.status.success(), "{}", combined(&positional));
    assert!(text(&positional.stdout).contains("odd,name.txt"));
    assert_eq!(fs::read(&odd).expect("odd"), b"odd");
    assert!(!dir.path().join("odd").exists());
    assert!(!dir.path().join("name.txt").exists());
}

#[test]
fn list_does_not_treat_a_comma_name_as_two_paths() {
    let dir = Temp::new();
    let odd = dir.path().join("odd,name.txt");
    fs::write(&odd, b"odd").expect("write");
    let output = run(dir.path(), &["--yes", "--list", "odd,name.txt"], None);
    assert!(!output.status.success(), "{}", combined(&output));
    assert_eq!(fs::read(&odd).expect("kept"), b"odd");
    assert!(combined(&output).contains("Nothing was changed."));
}

#[test]
fn folder_is_removed_bottom_up_including_spaces_and_unicode() {
    let dir = Temp::new();
    let root = dir.path().join("project");
    fs::create_dir_all(root.join("sub")).expect("dirs");
    fs::write(root.join("sub").join("b.txt"), b"bbb").expect("write");
    fs::write(root.join("my file.txt"), b"space").expect("write");
    fs::write(root.join("café.txt"), "é".as_bytes()).expect("write");
    fs::create_dir(root.join("empty")).expect("empty");

    let output = run(
        dir.path(),
        &["--dry-run", root.to_str().expect("utf8")],
        None,
    );
    assert!(output.status.success(), "{}", combined(&output));
    let stdout = text(&output.stdout);
    assert!(stdout.contains("b.txt"));
    assert!(stdout.contains("my file.txt"));
    assert!(stdout.contains("would remove folder"));
    assert!(root.exists());
    assert_eq!(fs::read(root.join("sub").join("b.txt")).expect("b"), b"bbb");
}

#[test]
fn symlink_to_a_file_is_removed_and_the_target_stays() {
    let dir = Temp::new();
    let target = dir.path().join("secret.txt");
    fs::write(&target, b"EXAMPLE_TOKEN_123").expect("write");
    let link = dir.path().join("link");
    symlink_file(&target, &link);
    let output = run(
        dir.path(),
        &["--dry-run", link.to_str().expect("utf8")],
        None,
    );
    assert!(output.status.success(), "{}", combined(&output));
    assert!(text(&output.stdout).contains("would delete symlink only"));
    assert!(link.exists());
    assert_eq!(fs::read(&target).expect("target"), b"EXAMPLE_TOKEN_123");
    assert!(!combined(&output).contains("EXAMPLE_TOKEN_123"));
}

#[test]
fn symlink_to_a_directory_is_not_entered() {
    let dir = Temp::new();
    let real = dir.path().join("real");
    fs::create_dir(&real).expect("dir");
    fs::write(real.join("file.txt"), b"inside").expect("write");
    let link = dir.path().join("linkdir");
    symlink_dir(&real, &link);
    let output = run(
        dir.path(),
        &["--dry-run", link.to_str().expect("utf8")],
        None,
    );
    assert!(output.status.success(), "{}", combined(&output));
    assert!(text(&output.stdout).contains("would delete symlink only"));
    assert!(link.exists());
    assert_eq!(fs::read(real.join("file.txt")).expect("inside"), b"inside");
}

#[test]
fn path_through_a_symlink_is_refused() {
    let dir = Temp::new();
    let real = dir.path().join("real");
    fs::create_dir(&real).expect("dir");
    let file = real.join("file.txt");
    fs::write(&file, b"inside").expect("write");
    let link = dir.path().join("linkdir");
    symlink_dir(&real, &link);
    let through = link.join("file.txt");
    let output = run(
        dir.path(),
        &["--yes", through.to_str().expect("utf8")],
        None,
    );
    assert!(!output.status.success(), "{}", combined(&output));
    assert!(combined(&output).contains("refusing to follow symlink"));
    assert_eq!(fs::read(&file).expect("inside"), b"inside");
    assert!(link.exists());
}

#[test]
fn tree_does_not_follow_a_symlink_to_an_outside_file() {
    let dir = Temp::new();
    let outside = dir.path().join("outside.txt");
    fs::write(&outside, b"EXAMPLE_TOKEN_123").expect("write");
    let root = dir.path().join("project");
    fs::create_dir(&root).expect("dir");
    fs::write(root.join("notes.txt"), b"notes").expect("write");
    symlink_file(&outside, &root.join("link"));
    let output = run(
        dir.path(),
        &["--dry-run", root.to_str().expect("utf8")],
        None,
    );
    assert!(output.status.success(), "{}", combined(&output));
    assert!(text(&output.stdout).contains("would delete symlink only"));
    assert!(root.exists());
    assert_eq!(fs::read(&outside).expect("outside"), b"EXAMPLE_TOKEN_123");
    assert!(!combined(&output).contains("EXAMPLE_TOKEN_123"));
}

#[test]
fn protected_root_and_etc_are_refused() {
    let dir = Temp::new();
    #[cfg(unix)]
    {
        let before = fs::read("/etc/passwd").expect("passwd");
        let output = run(dir.path(), &["--dry-run", "--yes", "/etc/passwd"], None);
        assert!(!output.status.success(), "{}", combined(&output));
        assert!(combined(&output).contains("protected"));
        assert!(combined(&output).contains("Nothing was changed."));
        assert!(!combined(&output).contains("Plan."));
        assert_eq!(fs::read("/etc/passwd").expect("passwd"), before);

        let root = run(dir.path(), &["--dry-run", "--yes", "/"], None);
        assert!(!root.status.success());
        let root_text = combined(&root);
        assert!(root_text.contains("protected"));
        assert!(root_text.lines().count() < 40, "{root_text}");
    }
    #[cfg(windows)]
    {
        let output = run(dir.path(), &["--dry-run", "--yes", r"C:\Windows"], None);
        assert!(!output.status.success(), "{}", combined(&output));
        assert!(combined(&output).contains("protected"));
        let root = run(dir.path(), &["--dry-run", "--yes", r"C:\"], None);
        assert!(!root.status.success());
        assert!(combined(&root).contains("protected"));
    }
}

#[test]
fn home_directory_itself_is_refused() {
    let dir = Temp::new();
    let Some(home) = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }) else {
        return;
    };
    let home = PathBuf::from(home);
    let output = run(
        dir.path(),
        &["--dry-run", "--yes", home.to_str().expect("utf8")],
        None,
    );
    assert!(!output.status.success(), "{}", combined(&output));
    assert!(combined(&output).contains("protected"));
    assert!(combined(&output).lines().count() < 40);
}

#[test]
fn missing_path_changes_nothing_else() {
    let dir = Temp::new();
    let file = dir.path().join("keep.txt");
    fs::write(&file, b"stay").expect("write");
    let missing = dir.path().join("missing.txt");
    let output = run(
        dir.path(),
        &[
            "--yes",
            file.to_str().expect("utf8"),
            missing.to_str().expect("utf8"),
        ],
        None,
    );
    assert!(!output.status.success());
    assert_eq!(fs::read(&file).expect("kept"), b"stay");
    assert!(combined(&output).contains("Nothing was changed."));
}

#[test]
fn read_only_file_is_not_deleted() {
    let dir = Temp::new();
    let file = dir.path().join("locked.txt");
    fs::write(&file, b"secret").expect("write");
    let mut permissions = fs::metadata(&file).expect("meta").permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&file, permissions).expect("chmod");
    if OpenOptions::new().write(true).open(&file).is_ok() {
        make_writable(&file);
        return;
    }
    let output = run(dir.path(), &["--yes", file.to_str().expect("utf8")], None);
    assert_eq!(output.status.code(), Some(2), "{}", combined(&output));
    assert!(combined(&output).contains("Nothing was changed."));
    assert!(!combined(&output).contains("securely deleted file"));
    assert_eq!(fs::read(&file).expect("kept"), b"secret");
    make_writable(&file);
}

#[test]
fn a_directory_request_does_not_change_children() {
    let dir = Temp::new();
    let first = dir.path().join("0-first.txt");
    let second = dir.path().join("1-later.txt");
    fs::write(&first, b"keep-first").expect("write");
    fs::write(&second, b"keep-second").expect("write");
    let output = run(
        dir.path(),
        &["--yes", dir.path().to_str().expect("utf8")],
        None,
    );
    assert_eq!(output.status.code(), Some(2), "{}", combined(&output));
    let report = combined(&output);
    assert!(report.contains("Nothing was changed."));
    assert!(!report.contains("not attempted"));
    assert!(!report.contains("securely deleted file"));
    assert_eq!(fs::read(&first).expect("first"), b"keep-first");
    assert_eq!(fs::read(&second).expect("second"), b"keep-second");
}

fn make_writable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o644));
    }
    #[cfg(windows)]
    {
        if let Ok(meta) = fs::metadata(path) {
            let mut permissions = meta.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            {
                permissions.set_readonly(false);
            }
            let _ = fs::set_permissions(path, permissions);
        }
    }
}

#[cfg(unix)]
#[test]
fn a_locked_folder_is_not_a_reason_to_truncate() {
    use std::os::unix::fs::PermissionsExt;
    let dir = Temp::new();
    let folder = dir.path().join("box");
    fs::create_dir(&folder).expect("dir");
    let file = folder.join("notes.txt");
    fs::write(&file, b"hello").expect("write");
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o555)).expect("chmod");
    let output = run(dir.path(), &["--yes", file.to_str().expect("utf8")], None);
    assert_eq!(output.status.code(), Some(2), "{}", combined(&output));
    assert!(combined(&output).contains("Nothing was changed."));
    assert!(file.exists());
    assert_eq!(fs::read(&file).expect("bytes"), b"hello");
}

#[cfg(unix)]
#[test]
fn hard_link_is_not_overwritten() {
    let dir = Temp::new();
    let first = dir.path().join("first.txt");
    let second = dir.path().join("second.txt");
    fs::write(&first, b"shared").expect("write");
    fs::hard_link(&first, &second).expect("link");
    let output = run(dir.path(), &["--yes", first.to_str().expect("utf8")], None);
    assert!(!output.status.success(), "{}", combined(&output));
    assert!(combined(&output).contains("NOT securely deleted"));
    assert!(combined(&output).contains("hard link"));
    assert_eq!(fs::read(&first).expect("first"), b"shared");
    assert_eq!(fs::read(&second).expect("second"), b"shared");
    assert!(first.exists());
    assert!(second.exists());
}

#[cfg(unix)]
#[test]
fn fifo_is_skipped() {
    let dir = Temp::new();
    let pipe = dir.path().join("pipe");
    let status = Command::new("mkfifo").arg(&pipe).status().expect("mkfifo");
    assert!(status.success());
    let output = run(dir.path(), &["--yes", pipe.to_str().expect("utf8")], None);
    assert!(!output.status.success(), "{}", combined(&output));
    assert!(combined(&output).contains("NOT securely deleted"));
    assert!(pipe.exists());
}

#[test]
fn zero_byte_file_is_deleted() {
    let dir = Temp::new();
    let file = dir.path().join("empty.txt");
    fs::write(&file, b"").expect("write");
    let output = run(dir.path(), &["--yes", file.to_str().expect("utf8")], None);
    assert_eq!(output.status.code(), Some(2), "{}", combined(&output));
    assert!(file.exists());
    assert_eq!(fs::metadata(&file).expect("meta").len(), 0);
}

#[test]
fn pattern_bytes_and_length_on_a_real_file() {
    let dir = Temp::new();
    let file = dir.path().join("paint.txt");
    let len = (64 * 1024) + 7;
    fs::write(&file, vec![b'Z'; len]).expect("write");
    let pattern = rmbin::pattern::Pattern::from_text("101").expect("pattern");
    let mut handle = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&file)
        .expect("open");
    let reported =
        rmbin::overwrite::overwrite_in_place(&file, &mut handle, &pattern).expect("paint");
    assert_eq!(reported, len as u64);
    handle.seek(SeekFrom::Start(0)).expect("seek");
    let mut got = Vec::new();
    handle.read_to_end(&mut got).expect("read");
    assert_eq!(got.len(), len);
    let bytes = b"101";
    assert_eq!(got[0], b'1');
    assert_eq!(got[1], b'0');
    assert_eq!(got[2], b'1');
    assert_eq!(got[len - 1], bytes[(len - 1) % 3]);
    assert_eq!(fs::metadata(&file).expect("meta").len(), len as u64);
}

#[test]
fn custom_pattern_flag_is_what_the_plan_shows() {
    let dir = Temp::new();
    let file = dir.path().join("notes.txt");
    fs::write(&file, b"abcd").expect("write");
    let output = run(
        dir.path(),
        &[
            "--dry-run",
            "--yes",
            "--pattern",
            "101",
            file.to_str().expect("utf8"),
        ],
        None,
    );
    assert!(output.status.success(), "{}", combined(&output));
    assert!(text(&output.stdout).contains("Pattern: '101'"));
    assert_eq!(fs::read(&file).expect("unchanged"), b"abcd");
}

#[test]
fn plan_names_later_steps_and_keeps_the_secret() {
    let dir = Temp::new();
    let file = dir.path().join("secret.txt");
    fs::write(&file, b"EXAMPLE_TOKEN_123").expect("write");
    let output = run(
        dir.path(),
        &[
            "--dry-run",
            "--pattern",
            "lol",
            file.to_str().expect("utf8"),
        ],
        None,
    );
    assert!(output.status.success(), "{}", combined(&output));
    let stdout = text(&output.stdout);
    assert!(stdout.contains("would overwrite"));
    assert!(stdout.contains("read back"));
    assert!(stdout.contains("later rename, not done"));
    assert!(stdout.contains("lollolloll"));
    assert!(stdout.contains("id card"));
    assert!(stdout.contains("does not read file contents"));
    assert!(stdout.contains("Nothing was removed"));
    assert!(stdout.contains("free-space fill"));
    assert!(!stdout.contains("EXAMPLE_TOKEN_123"));
    assert_eq!(fs::read(&file).expect("kept"), b"EXAMPLE_TOKEN_123");
    assert!(!dir.path().join("lollolloll").exists());
}

#[cfg(unix)]
#[test]
fn a_sparse_file_is_skipped_in_the_plan_and_left_in_place() {
    let dir = Temp::new();
    let file = dir.path().join("hole.bin");
    let handle = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file)
        .expect("create");
    handle.set_len(1_000_000).expect("len");
    drop(handle);
    let output = run(
        dir.path(),
        &["--dry-run", file.to_str().expect("utf8")],
        None,
    );
    assert!(!output.status.success(), "{}", combined(&output));
    assert!(combined(&output).contains("sparse or compressed"));
    assert!(combined(&output).contains("did not write"));
    assert_eq!(fs::metadata(&file).expect("meta").len(), 1_000_000);
}

#[test]
fn wipe_flag_is_rejected_and_changes_nothing() {
    let dir = Temp::new();
    let file = dir.path().join("notes.txt");
    fs::write(&file, b"secret").expect("write");
    let output = run(
        dir.path(),
        &["--wipe-free-space", file.to_str().expect("utf8")],
        None,
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read(&file).expect("kept"), b"secret");
}

#[test]
fn the_plan_names_the_filesystem_without_writing() {
    let dir = Temp::new();
    let file = dir.path().join("notes.txt");
    fs::write(&file, b"abc").expect("write");
    let output = run(
        dir.path(),
        &["--dry-run", file.to_str().expect("utf8")],
        None,
    );
    assert!(output.status.success(), "{}", combined(&output));
    let stdout = text(&output.stdout);
    assert!(
        stdout.contains("is on ") || stdout.contains("Could not see which filesystem"),
        "{stdout}"
    );
    assert!(stdout.contains("does not change the disk"));
    assert_eq!(fs::read(&file).expect("kept"), b"abc");
}

fn symlink_file(target: &Path, link: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link).expect("symlink");
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(target, link).expect("symlink");
}

fn symlink_dir(target: &Path, link: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link).expect("symlink dir");
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(target, link).expect("symlink dir");
}
