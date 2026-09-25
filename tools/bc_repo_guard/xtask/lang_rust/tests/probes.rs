//! Probes for the six `xtask` guards: `check-conversions`, `check-manifests`,
//! `check-plan-graph`, `check-closure`, `check-placement`, and `check-roster`.
//!
//! A probe plants one defect in a throwaway fixture, runs the guard over that
//! fixture, and asserts the exit code and the line the guard prints. The guard
//! contract is the three exit codes: 0 clean, 1 findings, 2 fail-closed. A
//! probe is correct when the baseline run is green and the planted run is red,
//! so each rule carries a baseline probe beside its planted probe.
//!
//! Every guard item is `pub(crate)` inside a binary target, so a probe reaches
//! the guard the way an operator does: it spawns the binary. No probe reads
//! this repository, and no probe writes outside its own scratch directory.
//!
//! The `closure_` group is the one exception to the second half of that
//! sentence, and the guard makes it one. `CL1c` reads a stored copy of the
//! review back from the plan store through `.claude/plan-coordination/db.sh`,
//! and `check-closure` resolves that launcher from the repository root its own
//! crate compiled in, so a probe of `CL1c` runs the launcher of this
//! repository. Both halves that matter stay throwaway: `PLAN_DB_ROOT` names
//! the probe's own scratch directory, so the store environment is created and
//! removed with the fixture, and the plan name comes from the scratch
//! directory that holds the throwaway document, so no key reaches the plan
//! store of `roadmap/duet-v1`. A probe that mocked the store would prove the
//! mock.
//!
//! `roster_generate_only_never_reaches_a_job` is the second exception, and the
//! rule it holds is why. The rule reads "a gate never takes
//! `--generate-only`", so its denominator is every file under
//! `.github/workflows/` of this repository. The probe reads that directory and
//! writes nothing. It is fail-closed on its own input: a directory that does
//! not open, and a directory that holds no file, are each a failure.
//! `roster_job_line_probe_refuses_a_planted_flag` is its committed positive
//! control, so no reader plants a defect by hand to know that the probe can go
//! red. Every other `roster_` probe reads a synthetic document in its own
//! scratch directory.

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::LazyLock;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{env, fs};

    /// The repository root: the nearest ancestor of this crate that holds the
    /// workspace lockfile. The guard binary resolves its own root the same way.
    fn repository_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .find(|dir| dir.join("Cargo.lock").is_file())
            .expect("an ancestor of the xtask crate holds the workspace lockfile")
            .to_path_buf()
    }

    /// The path of this crate relative to the repository root, with `/` separators.
    fn crate_path_in_repository() -> String {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .strip_prefix(repository_root())
            .expect("the xtask crate sits inside the repository root")
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/")
    }

    /// The CI changed-path filter names the directory that holds this crate.
    ///
    /// The `plan-lint` job runs only when the filter matches a changed path, so a
    /// filter that names a directory which no longer exists skips every guard
    /// probe in CI and still reports green.
    #[test]
    fn gate_filter_ci_names_the_guard_crate() {
        let workflow = fs::read_to_string(repository_root().join(".github/workflows/ci.yml"))
            .expect("the CI workflow opens");
        let crate_path = crate_path_in_repository();
        let filter = workflow
            .lines()
            .find(|line| line.contains("grep -qE '^roadmap/|"))
            .expect("the CI workflow holds the plan changed-path filter");
        let named: Vec<&str> = filter
            .split("grep -qE '")
            .nth(1)
            .and_then(|rest| rest.split('\'').next())
            .expect("the filter holds one quoted pattern")
            .split('|')
            .map(|alternative| alternative.trim_start_matches('^'))
            .collect();
        assert!(
            named
                .iter()
                .any(|prefix| format!("{crate_path}/").starts_with(prefix)),
            "the CI filter {named:?} names no prefix of the guard crate path `{crate_path}`"
        );
    }

    /// The Definition of Done gate runs the reason-text half of `check-conversions` when the
    /// one exempt file changes, so its changed-path pattern names that file.
    #[test]
    fn gate_filter_dod_names_the_exempt_conversion_file() {
        let root = repository_root();
        let dod = fs::read_to_string(root.join("scripts/dod.sh")).expect("the DoD script opens");
        let line = dod
            .lines()
            .find(|line| line.contains("touches '^roadmap/|") && line.contains("convert"))
            .expect("the DoD script holds the conversion changed-path filter");
        let pattern = line
            .split("|^")
            .nth(1)
            .and_then(|rest| rest.split('$').next())
            .expect("the filter names one anchored file after `roadmap/`");
        let file = pattern.replace("\\.", ".");
        assert!(
            root.join(&file).is_file(),
            "the DoD conversion filter names `{file}`, and no such file exists"
        );
    }

    /// A library file with no cast and no suppression.
    const CLEAN_LIB: &str = "//! A probe member.\n";

    /// A library whose line 2 holds one bare cast.
    const CAST_LIB: &str = "//! A probe member.\npub fn probe(x: u64) -> u32 { x as u32 }\n";

    /// A file whose line 1 holds one bare cast.
    const CAST_FIRST_LINE: &str = "pub fn probe(x: u64) -> u32 { x as u32 }\n";

    /// A library whose `use` item carries `as _` across three lines.
    const MULTILINE_USE_LIB: &str = "//! A probe member.
use core::fmt::Write
    as
    _;
pub fn probe(text: &str) -> usize { text.len() }
";

    /// A library whose line 2 holds a `use` item with no terminating semicolon.
    const UNTERMINATED_USE_LIB: &str = "//! A probe member.
use core::fmt::Write
";

    /// A library that carries three cast suppressions and no cast.
    const SUPPRESSIONS_LIB: &str = "#![allow( clippy::as_conversions )]
//! A probe member.

#[expect(
    clippy::as_conversions
)]
pub fn one() {}

#[cfg_attr(test, allow(clippy::as_conversions))]
pub fn two() {}
";

    /// A library in which every cast token sits inside a string or a comment.
    const LEXER_LIB: &str = r##"//! A probe member as a doc comment.
/* a block /* nested as */ comment as */
// a line comment that says as
const OPEN: &str = "/*";
const CLOSE: &str = "*/";
const RAW: &[u8] = br#"say "as" here and x as u32"#;
pub fn probe() -> usize { OPEN.len() + CLOSE.len() + RAW.len() }
"##;

    /// A library whose one cast sits on line 8, inside a comparison pair.
    const QUALIFIED_LIB: &str = "//! A probe member.

/// A qualified path probe.
pub fn qualified(v: i128) -> i64 {
    let _ = <u32 as Default>::default();
    <i64 as TryFrom<i128>>::try_from(v).unwrap_or(0)
}
pub fn compared(a: u32, b: u32, c: u64, d: u32) -> bool { a < b && c as u32 > d }
";

    /// A library that spells a raw identifier the cast scan reads as a cast.
    const RAW_IDENT_LIB: &str = "//! A probe member.
/// A probe.
pub fn probe() -> u32 {
    let r#as = 1u32;
    r#as
}
";

    /// A library that holds one cast and one cast suppression.
    const CAST_AND_SUPPRESSION_LIB: &str = "//! A probe member.
#[allow(clippy::as_conversions)]
pub fn probe(x: u64) -> u32 { x as u32 }
";

    /// One member of a probe workspace.
    #[derive(Debug)]
    struct Member {
        /// The member directory, relative to the workspace root.
        dir: String,
        /// The whole text of the member manifest.
        manifest: String,
        /// Each file the member holds: a member-relative path and its bytes.
        files: Vec<(String, Vec<u8>)>,
    }

    impl Member {
        /// One member whose only target is a library at `src/lib.rs`.
        fn lib(dir: &str, package: &str, files: &[(&str, &str)]) -> Self {
            Self::new(dir, &lib_manifest(package), files)
        }

        /// One member whose manifest the caller states in full.
        fn new(dir: &str, manifest: &str, files: &[(&str, &str)]) -> Self {
            Self {
                dir: dir.to_owned(),
                manifest: manifest.to_owned(),
                files: files
                    .iter()
                    .map(|(name, text)| ((*name).to_owned(), text.as_bytes().to_vec()))
                    .collect(),
            }
        }

        /// The same member plus one file stated as raw bytes.
        fn with_bytes(mut self, name: &str, bytes: &[u8]) -> Self {
            self.files.push((name.to_owned(), bytes.to_vec()));
            self
        }
    }

    /// One comma-separated list of quoted paths.
    fn quoted(items: &[&str]) -> String {
        items
            .iter()
            .map(|item| format!("\"{item}\""))
            .collect::<Vec<String>>()
            .join(", ")
    }

    /// The root manifest of one probe workspace.
    fn root_manifest(members: &[&str], exclude: &[&str]) -> String {
        let listed = quoted(members);
        let skipped = if exclude.is_empty() {
            String::new()
        } else {
            format!("exclude = [{}]\n", quoted(exclude))
        };
        format!(
            r#"[workspace]
resolver = "3"
members = [{listed}]
{skipped}
[workspace.lints.rust]
unsafe_code = "deny"
"#
        )
    }

    /// A member manifest whose one target is a library at `src/lib.rs`.
    fn lib_manifest(package: &str) -> String {
        let ident = package.replace('-', "_");
        format!(
            r#"[package]
name = "{package}"
description = "A probe member."
version = "0.1.0"
edition = "2024"
publish = false

[lib]
name = "{ident}"
path = "src/lib.rs"
"#
        )
    }

    /// A member manifest that meets both conditions the manifest guard asserts.
    fn compliant_manifest(package: &str) -> String {
        format!("{}\n[lints]\nworkspace = true\n", lib_manifest(package))
    }

    /// A member manifest whose `description` is empty.
    fn blank_description_manifest(package: &str) -> String {
        let ident = package.replace('-', "_");
        format!(
            r#"[package]
name = "{package}"
description = ""
version = "0.1.0"
edition = "2024"
publish = false

[lib]
name = "{ident}"
path = "src/lib.rs"

[lints]
workspace = true
"#
        )
    }

    /// A member manifest whose one target sits outside the `src` directory.
    fn outside_src_manifest(package: &str) -> String {
        let ident = package.replace('-', "_");
        format!(
            r#"[package]
name = "{package}"
description = "A probe member."
version = "0.1.0"
edition = "2024"
publish = false

[lib]
name = "{ident}"
path = "other/lib.rs"
"#
        )
    }

    /// The part of a scratch name that no other process of this machine holds.
    ///
    /// The process id separates two probe processes that run at one time, and
    /// the start instant separates a later process that the system gives the
    /// same id.
    static PROCESS_TAG: LazyLock<String> = LazyLock::new(|| {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        format!("{}z{nanos}", std::process::id())
    });

    /// How many names this process has minted.
    static PROBE_SERIAL: AtomicU64 = AtomicU64::new(0);

    /// The next serial of this process. Two calls never return one value.
    fn next_serial() -> u64 {
        PROBE_SERIAL.fetch_add(1, Ordering::Relaxed)
    }

    /// The test that asked, as the harness names the thread it runs on.
    ///
    /// The harness names each test thread after the test item, and the
    /// compiler holds two items of one module apart, so the name is unique. A
    /// run that states no name falls back to the label.
    fn probe_name(label: &str) -> String {
        let thread = std::thread::current();
        match thread.name() {
            Some("main") | None => label.to_owned(),
            Some(named) => named.replace("::", "-"),
        }
    }

    /// A scratch directory this probe owns, outside the repository.
    ///
    /// The name carries the test, the process, and a serial the process never
    /// repeats, so two probes never resolve to one path. Each probe store
    /// lives under this directory, so one path per probe is one store
    /// environment per probe.
    fn scratch(label: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "xtask-probe-{}-{}-{}",
            probe_name(label),
            *PROCESS_TAG,
            next_serial()
        ));
        fs::create_dir_all(&dir).expect("scratch root");
        dir
    }

    /// Write one file and create the directories above it.
    fn write_bytes(path: &Path, bytes: &[u8]) {
        let parent = path.parent().expect("a fixture file has a parent");
        fs::create_dir_all(parent).expect("fixture directory");
        fs::write(path, bytes).expect("fixture file");
    }

    /// Remove one scratch directory.
    fn clean(root: &Path) {
        fs::remove_dir_all(root).expect("scratch cleanup");
    }

    /// Write one throwaway cargo workspace and return its root directory.
    fn workspace(label: &str, manifest: &str, members: &[Member]) -> PathBuf {
        let root = scratch(label);
        write_bytes(&root.join("Cargo.toml"), manifest.as_bytes());
        for member in members {
            let base = root.join(&member.dir);
            write_bytes(&base.join("Cargo.toml"), member.manifest.as_bytes());
            for (name, bytes) in &member.files {
                write_bytes(&base.join(name), bytes);
            }
        }
        root
    }

    /// Run the `xtask` binary in one directory and return `(exit code, stdout)`.
    fn xtask(dir: &Path, args: &[&str]) -> (i32, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(args)
            .current_dir(dir)
            .output()
            .expect("spawn xtask");
        let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
        (output.status.code().unwrap_or(-1), stdout)
    }

    /// Build one conversion fixture, run the guard inside it, and clean up.
    fn conversions(label: &str, manifest: &str, members: &[Member]) -> (i32, String) {
        let root = workspace(label, manifest, members);
        let report = xtask(&root, &["check-conversions"]);
        clean(&root);
        report
    }

    /// Build one manifest fixture, run the guard inside it, and clean up.
    fn manifests(label: &str, manifest: &str, members: &[Member]) -> (i32, String) {
        let root = workspace(label, manifest, members);
        let report = xtask(&root, &["check-manifests"]);
        clean(&root);
        report
    }

    /// How many lines of one report carry one token.
    fn count_lines(report: &str, token: &str) -> usize {
        report.lines().filter(|line| line.contains(token)).count()
    }

    /// The whole architecture document of one probe plan directory.
    ///
    /// Each row is a phase number, the manifest-owner cell, and the cell that
    /// names the line chunks of that phase. Each link is one section 13.4 cell.
    fn architecture(rows: &[(u32, &str, &str)], links: &[&str]) -> String {
        use std::fmt::Write as _;

        let mut out = String::from(
            "# A probe plan\n\n### 13.3 The phases\n\n\
             | Phase | Manifest owner | Also written | Line chunks | Width |\n\
             |---|---|---|---|---|\n",
        );
        for (phase, owner, running) in rows {
            writeln!(out, "| {phase} | {owner} | none | {running} | 1 |").expect("row text");
        }
        out.push_str(
            "\n### 13.4 Every serial link and its reason\n\n| Link | Reason |\n|---|---|\n",
        );
        for link in links {
            writeln!(out, "| {link} | A probe reason. |").expect("link text");
        }
        out.push_str("\n## 14 Completion\n");
        out
    }

    /// One chunk file body with the front matter the plan-graph guard reads.
    fn chunk(id: &str, line: &str, depends_on: &[&str], write_scope: &[&str]) -> String {
        let deps = depends_on.join(", ");
        let scope = write_scope.join(", ");
        format!(
            "---\nid: {id}\nline: {line}\ndepends_on: [{deps}]\nwrite_scope: [{scope}]\n\
             parallelism: independent\ncompletion: cargo nextest run passes\n---\n\n# {id}\n"
        )
    }

    /// Write one throwaway plan directory and return its root.
    fn plan(label: &str, arch: &str, chunks: &[(&str, String)]) -> PathBuf {
        let root = scratch(label);
        write_bytes(&root.join("architecture.md"), arch.as_bytes());
        for (name, text) in chunks {
            write_bytes(&root.join(name), text.as_bytes());
        }
        root
    }

    /// Build one plan fixture, run the guard over it, and clean up.
    fn plan_graph(label: &str, arch: &str, chunks: &[(&str, String)]) -> (i32, String) {
        let root = plan(label, arch, chunks);
        let named = root.display().to_string();
        let report = xtask(&root, &["check-plan-graph", named.as_str()]);
        clean(&root);
        report
    }

    /// Run the plan-graph guard over one standing plan directory.
    ///
    /// A manifest probe runs the guard twice over one fixture, once to write
    /// `plan-graph.md` and once to read it, so the fixture outlives the run
    /// and the probe cleans it up itself.
    fn plan_graph_run(root: &Path, flags: &[&str]) -> (i32, String) {
        let named = root.display().to_string();
        let mut args = vec!["check-plan-graph", named.as_str()];
        args.extend_from_slice(flags);
        xtask(root, &args)
    }

    /// The architecture document every manifest probe reads.
    fn manifest_architecture() -> String {
        architecture(&[(1, "none", "A1"), (2, "none", "B2")], &["A1 before B2"])
    }

    /// The two chunk files every manifest probe writes.
    fn manifest_chunks(scope: &str) -> Vec<(&'static str, String)> {
        vec![
            (
                "a1.md",
                chunk("A1", "core", &[], &["crates/bc_x/alpha/lang_rust/src/a.rs"]),
            ),
            ("b2.md", chunk("B2", "core", &["A1"], &[scope])),
        ]
    }

    #[test]
    fn conversion_clean_workspace_exits_zero() {
        let (code, report) = conversions(
            "clean",
            &root_manifest(&["crates/*"], &[]),
            &[
                Member::lib("crates/alpha", "alpha", &[("src/lib.rs", CLEAN_LIB)]),
                Member::lib("crates/beta", "beta", &[("src/lib.rs", CLEAN_LIB)]),
            ],
        );
        assert_eq!(code, 0, "a clean workspace is clean: {report}");
        assert!(
            report.contains(
                "MEMBERS: 2   FILES: 2   FINDINGS: 0   EXPECTED MEMBERS: 2   EXPECTED FILES: 2"
            ),
            "the summary line states the derived sets: {report}"
        );
    }

    #[test]
    fn conversion_cg3_bare_cast_is_a_finding() {
        let (code, report) = conversions(
            "cg3-cast",
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/alpha",
                "alpha",
                &[("src/lib.rs", CAST_LIB)],
            )],
        );
        assert_eq!(code, 1, "a bare cast is a finding: {report}");
        assert!(
            report.contains("  CAST: crates/alpha/src/lib.rs: line 2"),
            "the finding names the file and the line: {report}"
        );
    }

    #[test]
    fn conversion_cg4_multiline_use_is_dropped() {
        let (code, report) = conversions(
            "cg4-use",
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/alpha",
                "alpha",
                &[("src/lib.rs", MULTILINE_USE_LIB)],
            )],
        );
        assert_eq!(code, 0, "a `use` item with `as _` is dropped: {report}");
        assert_eq!(
            count_lines(&report, "CAST"),
            0,
            "no cast is reported: {report}"
        );
    }

    #[test]
    fn conversion_cg4b_unterminated_use_fails_closed() {
        let (code, report) = conversions(
            "cg4b-use",
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/alpha",
                "alpha",
                &[("src/lib.rs", UNTERMINATED_USE_LIB)],
            )],
        );
        assert_eq!(
            code, 2,
            "a `use` with no semicolon is fail-closed: {report}"
        );
        assert!(
            report.contains(
                "FAIL: crates/alpha/src/lib.rs: a `use` at line 2 has no `;`; the guard is fail-closed."
            ),
            "the guard names the file and the line: {report}"
        );
    }

    #[test]
    fn conversion_cg6_every_suppression_form_is_a_finding() {
        let (code, report) = conversions(
            "cg6-suppress",
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/alpha",
                "alpha",
                &[("src/lib.rs", SUPPRESSIONS_LIB)],
            )],
        );
        assert_eq!(code, 1, "a cast suppression is a finding: {report}");
        assert_eq!(
            count_lines(&report, "SUPPRESSION"),
            3,
            "the inner `allow`, the multi-line `expect`, and the `cfg_attr` wrapper each report: {report}"
        );
        for line in ["line 1", "line 4", "line 9"] {
            assert!(
                report.contains(&format!("  SUPPRESSION: crates/alpha/src/lib.rs: {line}")),
                "the guard names {line}: {report}"
            );
        }
    }

    #[test]
    fn conversion_cg2_lexer_keeps_a_cast_token_inert() {
        let (code, report) = conversions(
            "cg2-lexer",
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/alpha",
                "alpha",
                &[("src/lib.rs", LEXER_LIB)],
            )],
        );
        assert_eq!(code, 0, "a cast token outside code is inert: {report}");
        assert_eq!(
            count_lines(&report, "CAST"),
            0,
            "no cast is reported: {report}"
        );
    }

    #[test]
    fn conversion_cg5_qualified_path_is_exempt_and_a_comparison_is_not() {
        let (code, report) = conversions(
            "cg5-qualified",
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/alpha",
                "alpha",
                &[("src/lib.rs", QUALIFIED_LIB)],
            )],
        );
        assert_eq!(code, 1, "a comparison pair is not exempt: {report}");
        assert_eq!(
            count_lines(&report, "CAST"),
            1,
            "only the comparison reports: {report}"
        );
        assert!(
            report.contains("  CAST: crates/alpha/src/lib.rs: line 8"),
            "the finding names the comparison line: {report}"
        );
    }

    #[test]
    fn conversion_cg3b_raw_identifier_is_the_stated_false_red() {
        let (code, report) = conversions(
            "cg3b-raw-ident",
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/alpha",
                "alpha",
                &[("src/lib.rs", RAW_IDENT_LIB)],
            )],
        );
        assert_eq!(code, 1, "a raw identifier reports a cast: {report}");
        assert_eq!(
            count_lines(&report, "CAST"),
            2,
            "the name and its use both report: {report}"
        );
        for line in ["line 4", "line 5"] {
            assert!(
                report.contains(&format!("  CAST: crates/alpha/src/lib.rs: {line}")),
                "the guard names {line}: {report}"
            );
        }
    }

    #[test]
    fn conversion_cg1b_excluded_member_fails_closed() {
        let (code, report) = conversions(
            "cg1b-exclude",
            &root_manifest(&["crates/*", "tools/*"], &["tools/beta"]),
            &[
                Member::lib("crates/alpha", "alpha", &[("src/lib.rs", CLEAN_LIB)]),
                Member::lib("tools/beta", "beta", &[("src/lib.rs", CAST_LIB)]),
            ],
        );
        assert_eq!(code, 2, "an excluded member is fail-closed: {report}");
        assert!(
            report.contains(
                "FAIL: tools/beta holds a Cargo.toml and the scan does not cover it; the guard is fail-closed."
            ),
            "the guard names the member it does not cover: {report}"
        );
    }

    #[test]
    fn conversion_cg1b_dot_directory_member_fails_closed() {
        let (code, report) = conversions(
            "cg1b-dotdir",
            &root_manifest(&["crates/*"], &["crates/.fixture"]),
            &[
                Member::lib("crates/alpha", "alpha", &[("src/lib.rs", CLEAN_LIB)]),
                Member::lib("crates/.fixture", "fixture", &[("src/lib.rs", CAST_LIB)]),
            ],
        );
        assert_eq!(
            code, 2,
            "a member whose name opens with a dot is fail-closed: {report}"
        );
        assert!(
            report.contains(
                "FAIL: crates/.fixture holds a Cargo.toml and the scan does not cover it; the guard is fail-closed."
            ),
            "the guard names the member it does not cover: {report}"
        );
    }

    #[test]
    fn conversion_cg1b_top_level_sibling_fails_closed() {
        let (code, report) = conversions(
            "cg1b-sibling",
            &root_manifest(&["alpha"], &["beta"]),
            &[
                Member::lib("alpha", "alpha", &[("src/lib.rs", CLEAN_LIB)]),
                Member::lib("beta", "beta", &[("src/lib.rs", CAST_LIB)]),
            ],
        );
        assert_eq!(
            code, 2,
            "an excluded top-level sibling is fail-closed: {report}"
        );
        assert!(
            report.contains(
                "FAIL: beta holds a Cargo.toml and the scan does not cover it; the guard is fail-closed."
            ),
            "the guard names the member it does not cover: {report}"
        );
    }

    #[test]
    fn conversion_cg1b_source_outside_the_target_directory_fails_closed() {
        let (code, report) = conversions(
            "cg1b-outside-src",
            &root_manifest(&["crates/*"], &[]),
            &[Member::new(
                "crates/alpha",
                &outside_src_manifest("alpha"),
                &[("other/lib.rs", CLEAN_LIB), ("src/hidden.rs", CAST_LIB)],
            )],
        );
        assert_eq!(
            code, 2,
            "a source file the walk misses is fail-closed: {report}"
        );
        assert!(
            report.contains(
                "FAIL: crates/alpha/src/hidden.rs sits under a scanned member's src and the scan does not hold it; the guard is fail-closed."
            ),
            "the guard names the file it does not hold: {report}"
        );
    }

    #[test]
    fn conversion_cg8_bytes_that_are_not_utf8_fail_closed() {
        let (code, report) = conversions(
            "cg8-not-utf8",
            &root_manifest(&["crates/*"], &[]),
            &[
                Member::lib("crates/alpha", "alpha", &[("src/lib.rs", CLEAN_LIB)])
                    .with_bytes("src/bad.rs", b"//! \xc3\x28 a probe\n"),
            ],
        );
        assert_eq!(
            code, 2,
            "a member file that is not UTF-8 is fail-closed: {report}"
        );
        assert!(
            report.contains(
                "FAIL: crates/alpha/src/bad.rs: the bytes are not UTF-8; the guard is fail-closed."
            ),
            "the guard names the file it cannot decode: {report}"
        );
    }

    #[test]
    fn conversion_cg8_unreadable_file_fails_closed() {
        use std::os::unix::fs::PermissionsExt as _;

        let root = workspace(
            "cg8-unreadable",
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/alpha",
                "alpha",
                &[("src/lib.rs", CLEAN_LIB), ("src/shut.rs", CLEAN_LIB)],
            )],
        );
        let shut = root
            .join("crates")
            .join("alpha")
            .join("src")
            .join("shut.rs");
        fs::set_permissions(&shut, fs::Permissions::from_mode(0o000)).expect("set mode 000");
        if fs::read(&shut).is_ok() {
            println!("skipped: this process reads a mode-000 file, so it runs as root");
            clean(&root);
            return;
        }
        let (code, report) = xtask(&root, &["check-conversions"]);
        clean(&root);
        assert_eq!(
            code, 2,
            "a member file that does not open is fail-closed: {report}"
        );
        assert!(
            report.contains(
                "FAIL: crates/alpha/src/shut.rs: the file does not open; the guard is fail-closed."
            ),
            "the guard names the file it cannot open: {report}"
        );
    }

    #[test]
    fn conversion_cg8_refused_metadata_fails_closed() {
        let root = workspace("cg8-refused", "[workspace\nmembers = [\n", &[]);
        let named = fs::canonicalize(&root).expect("canonical scratch root");
        let (code, report) = xtask(&root, &["check-conversions"]);
        clean(&root);
        assert_eq!(
            code, 2,
            "a workspace cargo refuses is fail-closed: {report}"
        );
        assert!(
            report.contains(&format!(
                "FAIL: `cargo metadata --no-deps` refused {}; the guard is fail-closed.",
                named.display()
            )),
            "the guard names the directory cargo refused: {report}"
        );
    }

    #[test]
    fn conversion_cg1_source_module_named_target_is_scanned() {
        let (code, report) = conversions(
            "cg1-target-module",
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/alpha",
                "alpha",
                &[
                    ("src/lib.rs", "//! A probe member.\npub mod target;\n"),
                    ("src/target/mod.rs", CAST_FIRST_LINE),
                ],
            )],
        );
        assert_eq!(
            code, 1,
            "a source module named target stays in the scan: {report}"
        );
        assert!(
            report.contains("  CAST: crates/alpha/src/target/mod.rs: line 1"),
            "the finding names the module file: {report}"
        );
    }

    #[test]
    fn conversion_cg1_uppercase_extension_is_scanned() {
        let (code, report) = conversions(
            "cg1-uppercase-ext",
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/alpha",
                "alpha",
                &[("src/lib.rs", CLEAN_LIB), ("src/Other.RS", CAST_FIRST_LINE)],
            )],
        );
        assert_eq!(
            code, 1,
            "a Rust extension matches without regard to case: {report}"
        );
        assert!(
            report.contains("  CAST: crates/alpha/src/Other.RS: line 1"),
            "the finding names the uppercase file: {report}"
        );
    }

    #[test]
    fn conversion_cg7_only_the_exempt_file_may_hold_a_cast() {
        let (code, report) = conversions(
            "cg7-exempt",
            &root_manifest(&["crates/*/*/lang_rust"], &[]),
            &[Member::lib(
                "crates/bc_time/duet-time/lang_rust",
                "duet-time",
                &[
                    ("src/lib.rs", CLEAN_LIB),
                    ("src/convert.rs", CAST_LIB),
                    ("src/other.rs", CAST_LIB),
                ],
            )],
        );
        assert_eq!(
            code, 1,
            "a cast beside the exempt file is a finding: {report}"
        );
        assert_eq!(
            count_lines(&report, "CAST"),
            1,
            "only the sibling reports: {report}"
        );
        assert!(
            report.contains("  CAST: crates/bc_time/duet-time/lang_rust/src/other.rs: line 2"),
            "the finding names the sibling file: {report}"
        );
    }

    #[test]
    fn conversion_cg7_a_suppression_inside_the_exempt_file_is_exempt() {
        let (code, report) = conversions(
            "cg7-exempt-suppression",
            &root_manifest(&["crates/*/*/lang_rust"], &[]),
            &[Member::lib(
                "crates/bc_time/duet-time/lang_rust",
                "duet-time",
                &[
                    ("src/lib.rs", CLEAN_LIB),
                    ("src/convert.rs", CAST_AND_SUPPRESSION_LIB),
                ],
            )],
        );
        assert_eq!(
            code, 0,
            "the exempt file carries its own suppression: {report}"
        );
        assert_eq!(
            count_lines(&report, "SUPPRESSION"),
            0,
            "no suppression is reported: {report}"
        );
    }

    /// The `b1-convert` cell of the `CG9` green control.
    ///
    /// The cell carries a quotation mark and the markdown escape `\|`, so the
    /// control proves that the guard reads the cell a reader sees.
    const CONTROL_CELL: &str = "\"a \"quoted\" word and a \\| bar\"";

    /// The exempt-file source whose reason text matches [`CONTROL_CELL`].
    ///
    /// The literal carries a `\"` escape and breaks across two lines with a
    /// backslash continuation, so the control proves that the guard decodes
    /// the literal and never compares its raw source token.
    const CONTROL_CONVERT: &str = "//! A probe member.
#[expect(
    clippy::as_conversions,
    reason = \"a \\\"quoted\\\" word and a | \\
        bar\"
)]
pub fn alpha(value: u64) -> u64 { value }
";

    /// A synthetic appendix whose `b1-convert` block states the given rows.
    ///
    /// Each pair is a site and the whole cell three of its row, quotation
    /// marks included, so a probe states the cell exactly as an author writes
    /// it in the markdown table.
    fn b1_appendix(rows: &[(&str, &str)]) -> String {
        use std::fmt::Write as _;

        let mut out = String::from(
            "# A probe appendix\n\n#### Conversion suppressions\n\n\
             <!-- GUARD BLOCK id=b1-convert rows>=1 -->\n\
             | Site | Lints the attribute names | Reason text that the code must carry |\n\
             |---|---|---|\n",
        );
        for (site, cell) in rows {
            let _written = writeln!(out, "| `{site}` | `as_conversions` | {cell} |");
        }
        out.push('\n');
        out
    }

    /// An exempt-file source that declares one `fn` per given pair.
    ///
    /// Each pair is a function name and the reason text its `#[expect]`
    /// carries, stated as the source spells it between the quotation marks.
    fn convert_source(sites: &[(&str, &str)]) -> String {
        use std::fmt::Write as _;

        let mut out = String::from("//! A probe member.\n");
        for (name, text) in sites {
            let _written = writeln!(
                out,
                "#[expect(\n    clippy::as_conversions,\n    reason = \"{text}\"\n)]\n\
                 pub fn {name}(value: u64) -> u64 {{ value }}\n"
            );
        }
        out
    }

    /// The one member every `CG9` probe builds: the exempt file and its crate.
    fn exempt_member(convert: &str) -> Member {
        Member::lib(
            "crates/bc_time/duet-time/lang_rust",
            "duet-time",
            &[("src/lib.rs", CLEAN_LIB), ("src/convert.rs", convert)],
        )
    }

    /// Build one `CG9` fixture, run the guard inside it, and clean up.
    ///
    /// The appendix path is absolute and names a document under the scratch
    /// root, so no probe of this group reads this repository. A `None`
    /// appendix writes no document, which leaves the named path absent.
    fn conversions_with_appendix(
        label: &str,
        convert: &str,
        appendix: Option<&str>,
    ) -> (i32, String) {
        let root = workspace(
            label,
            &root_manifest(&["crates/*/*/lang_rust"], &[]),
            &[exempt_member(convert)],
        );
        let document = root.join("roadmap").join("duet-v1").join("architecture.md");
        if let Some(text) = appendix {
            write_bytes(&document, text.as_bytes());
        }
        let named = document.display().to_string();
        let report = xtask(&root, &["check-conversions", "--appendix", &named]);
        clean(&root);
        report
    }

    #[test]
    fn conversions_reason_text_that_differs_is_a_finding() {
        let (code, report) = conversions_with_appendix(
            "cg9-differs",
            &convert_source(&[("alpha", "the divisor is 2^23")]),
            Some(&b1_appendix(&[("alpha", "\"the divisor is 2^24\"")])),
        );
        assert_eq!(code, 1, "a reason text that differs is a finding: {report}");
        assert_eq!(
            count_lines(&report, "  REASON TEXT: "),
            1,
            "one line names the one divergence: {report}"
        );
        assert!(
            report.contains("REASON TEXTS:    1     REASON TEXT BAD: 1"),
            "the counter states the denominator and the failures: {report}"
        );
        assert!(
            report.contains("  REASON TEXT: alpha:"),
            "the line names the site: {report}"
        );
    }

    #[test]
    fn conversions_reason_row_with_no_site_is_a_finding() {
        let (code, report) = conversions_with_appendix(
            "cg9-row-no-site",
            &convert_source(&[("alpha", "the divisor is 2^23")]),
            Some(&b1_appendix(&[
                ("alpha", "\"the divisor is 2^23\""),
                ("ghost", "\"a site no file declares\""),
            ])),
        );
        assert_eq!(code, 1, "a row with no site is a finding: {report}");
        assert_eq!(
            count_lines(&report, "  REASON TEXT: "),
            1,
            "one line names the one row: {report}"
        );
        assert!(
            report.contains("REASON TEXTS:    2     REASON TEXT BAD: 1"),
            "the counter states both sites and the one failure: {report}"
        );
        assert!(
            report.contains("  REASON TEXT: ghost:"),
            "the line names the row site: {report}"
        );
    }

    #[test]
    fn conversions_reason_site_with_no_row_is_a_finding() {
        let (code, report) = conversions_with_appendix(
            "cg9-site-no-row",
            &convert_source(&[
                ("alpha", "the divisor is 2^23"),
                ("extra", "a reason no row names"),
            ]),
            Some(&b1_appendix(&[("alpha", "\"the divisor is 2^23\"")])),
        );
        assert_eq!(code, 1, "a site with no row is a finding: {report}");
        assert_eq!(
            count_lines(&report, "  REASON TEXT: "),
            1,
            "one line names the one site: {report}"
        );
        assert!(
            report.contains("REASON TEXTS:    2     REASON TEXT BAD: 1"),
            "the counter states both sites and the one failure: {report}"
        );
        assert!(
            report.contains("  REASON TEXT: extra:"),
            "the line names the code site: {report}"
        );
    }

    #[test]
    fn conversions_appendix_that_does_not_open_fails_closed() {
        let (code, report) = conversions_with_appendix(
            "cg9-absent",
            &convert_source(&[("alpha", "the divisor is 2^23")]),
            None,
        );
        assert_eq!(code, 2, "an absent appendix fails closed: {report}");
        assert!(
            report.contains("the appendix does not open; the guard is fail-closed."),
            "the named line states the fail-closed reason: {report}"
        );
        assert!(
            report.contains("architecture.md"),
            "the named line names the document: {report}"
        );
        assert_eq!(
            count_lines(&report, "REASON TEXT BAD: "),
            0,
            "a fail-closed run prints no counter: {report}"
        );
    }

    #[test]
    fn conversions_reason_text_that_matches_is_clean() {
        let (code, report) = conversions_with_appendix(
            "cg9-matches",
            CONTROL_CONVERT,
            Some(&b1_appendix(&[("alpha", CONTROL_CELL)])),
        );
        assert_eq!(code, 0, "a matching pair is clean: {report}");
        assert!(
            report.contains("REASON TEXTS:    1     REASON TEXT BAD: 0"),
            "the counter states one text and no failure: {report}"
        );
        assert_eq!(
            count_lines(&report, "  REASON TEXT: "),
            0,
            "a clean run names no site: {report}"
        );
    }

    #[test]
    fn manifests_clean_workspace_exits_zero() {
        let (code, report) = manifests(
            "manifests-clean",
            &root_manifest(&["crates/*"], &[]),
            &[
                Member::new(
                    "crates/alpha",
                    &compliant_manifest("alpha"),
                    &[("src/lib.rs", CLEAN_LIB)],
                ),
                Member::new(
                    "crates/beta",
                    &compliant_manifest("beta"),
                    &[("src/lib.rs", CLEAN_LIB)],
                ),
            ],
        );
        assert_eq!(code, 0, "a compliant workspace is clean: {report}");
        assert!(
            report.contains("MEMBERS: 2   FINDINGS: 0"),
            "the summary line counts every member: {report}"
        );
    }

    #[test]
    fn manifests_member_with_no_lints_table_is_a_finding() {
        let (code, report) = manifests(
            "manifests-lints",
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/alpha",
                "alpha",
                &[("src/lib.rs", CLEAN_LIB)],
            )],
        );
        assert_eq!(
            code, 1,
            "a member outside the lint policy is a finding: {report}"
        );
        assert!(
            report.contains("crates/alpha/Cargo.toml: no `[lints] workspace = true`"),
            "the finding names the member path and the condition: {report}"
        );
    }

    #[test]
    fn manifests_member_with_an_empty_description_is_a_finding() {
        let (code, report) = manifests(
            "manifests-description",
            &root_manifest(&["crates/*"], &[]),
            &[Member::new(
                "crates/alpha",
                &blank_description_manifest("alpha"),
                &[("src/lib.rs", CLEAN_LIB)],
            )],
        );
        assert_eq!(
            code, 1,
            "a member with no description is a finding: {report}"
        );
        assert!(
            report.contains("crates/alpha/Cargo.toml: no non-empty `description`"),
            "the finding names the member path and the condition: {report}"
        );
    }

    #[test]
    fn manifests_refused_metadata_fails_closed() {
        let (code, report) = manifests("manifests-refused", "[workspace\nmembers = [\n", &[]);
        assert_eq!(
            code, 2,
            "a workspace cargo refuses is fail-closed: {report}"
        );
        assert!(
            report.contains("FAIL: `cargo metadata` exited with"),
            "the guard states that cargo refused the workspace: {report}"
        );
    }

    #[test]
    fn plan_graph_well_formed_plan_exits_zero() {
        let arch = architecture(&[(1, "none", "A1"), (2, "none", "B2")], &["A1 before B2"]);
        let (code, report) = plan_graph(
            "pg-clean",
            &arch,
            &[
                (
                    "a1.md",
                    chunk("A1", "core", &[], &["crates/bc_x/alpha/lang_rust/src/a.rs"]),
                ),
                (
                    "b2.md",
                    chunk(
                        "B2",
                        "core",
                        &["A1"],
                        &["crates/bc_x/beta/lang_rust/src/b.rs"],
                    ),
                ),
            ],
        );
        assert_eq!(code, 0, "a well-formed plan is clean: {report}");
        assert!(
            report.contains("CHUNKS: 2   PHASES: 2   LINKS 13.4: 1   FINDINGS: 0"),
            "the summary line states every derived count: {report}"
        );
    }

    #[test]
    fn plan_graph_unknown_dependency_is_a_finding() {
        let arch = architecture(&[(1, "none", "A1")], &[]);
        let (code, report) = plan_graph(
            "pg-unknown",
            &arch,
            &[(
                "a1.md",
                chunk(
                    "A1",
                    "core",
                    &["Z9"],
                    &["crates/bc_x/alpha/lang_rust/src/a.rs"],
                ),
            )],
        );
        assert_eq!(code, 1, "an unknown dependency is a finding: {report}");
        assert!(
            report.contains("FINDING: A1 depends on Z9, which has no chunk file"),
            "the finding names the chunk and the missing id: {report}"
        );
    }

    #[test]
    fn plan_graph_vocabulary_duplicate_chunk_id_is_a_finding() {
        let arch = architecture(&[(1, "none", "A1")], &[]);
        let body = chunk("A1", "core", &[], &["crates/bc_x/alpha/lang_rust/src/a.rs"]);
        let (code, report) = plan_graph(
            "pg-duplicate",
            &arch,
            &[("a1.md", body.clone()), ("a1-copy.md", body)],
        );
        assert_eq!(code, 1, "two files that state one id give exit 1: {report}");
        assert!(
            report
                .lines()
                .any(|line| line.starts_with("FINDING: duplicate chunk id")),
            "the report opens the duplicate-id line with the finding word: {report}"
        );
        assert!(
            report.contains("FINDING: duplicate chunk id A1 in a1.md and a1-copy.md"),
            "the finding names the id and both files: {report}"
        );
    }

    #[test]
    fn plan_graph_vocabulary_empty_directory_fails_closed() {
        let arch = architecture(&[(1, "none", "A1")], &[]);
        let (code, report) = plan_graph("pg-empty", &arch, &[]);
        assert_eq!(
            code, 2,
            "a plan directory with no chunk file gives exit 2: {report}"
        );
        assert!(
            report.contains("FAIL: no chunk file found; the guard is fail-closed."),
            "the guard states that it read no chunk and is fail-closed: {report}"
        );
    }

    #[test]
    fn plan_graph_cycle_is_a_finding() {
        let arch = architecture(&[(1, "none", "A1, B1")], &[]);
        let (code, report) = plan_graph(
            "pg-cycle",
            &arch,
            &[
                (
                    "a1.md",
                    chunk(
                        "A1",
                        "core",
                        &["B1"],
                        &["crates/bc_x/alpha/lang_rust/src/a.rs"],
                    ),
                ),
                (
                    "b1.md",
                    chunk(
                        "B1",
                        "edge",
                        &["A1"],
                        &["crates/bc_x/beta/lang_rust/src/b.rs"],
                    ),
                ),
            ],
        );
        assert_eq!(code, 1, "a dependency cycle is a finding: {report}");
        assert!(
            report.contains("FINDING: cycle: A1 -> B1 -> A1"),
            "the finding names the chain: {report}"
        );
    }

    #[test]
    fn plan_graph_backward_link_is_a_finding() {
        let arch = architecture(&[(1, "none", "A1"), (2, "none", "B2")], &[]);
        let (code, report) = plan_graph(
            "pg-backward",
            &arch,
            &[
                (
                    "a1.md",
                    chunk(
                        "A1",
                        "core",
                        &["B2"],
                        &["crates/bc_x/alpha/lang_rust/src/a.rs"],
                    ),
                ),
                (
                    "b2.md",
                    chunk("B2", "edge", &[], &["crates/bc_x/beta/lang_rust/src/b.rs"]),
                ),
            ],
        );
        assert_eq!(code, 1, "a backward link is a finding: {report}");
        assert!(
            report.contains("FINDING: A1 (phase 1) depends on B2 (phase 2), a backward link"),
            "the finding names both chunks and both phases: {report}"
        );
    }

    #[test]
    fn plan_graph_shared_write_scope_inside_one_phase_is_a_finding() {
        let arch = architecture(&[(1, "none", "A1, B1")], &[]);
        let shared = "crates/bc_x/shared/lang_rust/src/lib.rs";
        let (code, report) = plan_graph(
            "pg-scope",
            &arch,
            &[
                ("a1.md", chunk("A1", "core", &[], &[shared])),
                ("b1.md", chunk("B1", "edge", &[], &[shared])),
            ],
        );
        assert_eq!(
            code, 1,
            "a shared write scope inside one phase is a finding: {report}"
        );
        assert!(
            report.contains(&format!(
                "FINDING: phase 1: A1 and B1 both write {shared} / {shared}"
            )),
            "the finding names both chunks and the shared path: {report}"
        );
    }

    #[test]
    fn plan_graph_two_chunks_of_one_line_in_one_phase_is_a_finding() {
        let arch = architecture(&[(1, "none", "A1, B1")], &[]);
        let (code, report) = plan_graph(
            "pg-line",
            &arch,
            &[
                (
                    "a1.md",
                    chunk("A1", "core", &[], &["crates/bc_x/alpha/lang_rust/src/a.rs"]),
                ),
                (
                    "b1.md",
                    chunk("B1", "core", &[], &["crates/bc_x/beta/lang_rust/src/b.rs"]),
                ),
            ],
        );
        assert_eq!(
            code, 1,
            "one line with two chunks in one phase is a finding: {report}"
        );
        assert!(
            report.contains("FINDING: phase 1: line core has two chunks, A1 and B1"),
            "the finding names the line and both chunks: {report}"
        );
    }

    #[test]
    fn plan_graph_phase_row_with_no_chunk_file_is_a_finding() {
        let arch = architecture(&[(1, "none", "A1, C9")], &[]);
        let (code, report) = plan_graph(
            "pg-row",
            &arch,
            &[(
                "a1.md",
                chunk("A1", "core", &[], &["crates/bc_x/alpha/lang_rust/src/a.rs"]),
            )],
        );
        assert_eq!(
            code, 1,
            "a phase row with no chunk file is a finding: {report}"
        );
        assert!(
            report.contains("FINDING: C9 has a row in section 13.3 and no chunk file"),
            "the finding names the chunk of the row: {report}"
        );
    }

    #[test]
    fn plan_graph_link_that_no_dependency_carries_is_a_finding() {
        let arch = architecture(&[(1, "none", "A1"), (2, "none", "B2")], &["A1 before B2"]);
        let (code, report) = plan_graph(
            "pg-link",
            &arch,
            &[
                (
                    "a1.md",
                    chunk("A1", "core", &[], &["crates/bc_x/alpha/lang_rust/src/a.rs"]),
                ),
                (
                    "b2.md",
                    chunk("B2", "edge", &[], &["crates/bc_x/beta/lang_rust/src/b.rs"]),
                ),
            ],
        );
        assert_eq!(
            code, 1,
            "a link with no dependency edge is a finding: {report}"
        );
        assert!(
            report
                .contains("FINDING: section 13.4 link A1 before B2 is not a depends_on edge of B2"),
            "the finding names both chunks of the link: {report}"
        );
    }

    #[test]
    fn plan_graph_directory_that_does_not_open_fails_closed() {
        let root = scratch("pg-absent");
        let absent = root.join("no-such-plan");
        let named = absent.display().to_string();
        let (code, report) = xtask(&root, &["check-plan-graph", named.as_str()]);
        clean(&root);
        assert_eq!(
            code, 2,
            "a plan directory that does not open is fail-closed: {report}"
        );
        assert!(
            report.contains("FAIL: cannot open"),
            "the guard states that it cannot open the plan: {report}"
        );
    }

    #[test]
    fn plan_graph_manifest_that_the_front_matter_generates_is_clean() {
        let root = plan(
            "pg-manifest-clean",
            &manifest_architecture(),
            &manifest_chunks("crates/bc_x/beta/lang_rust/src/b.rs"),
        );
        let (written, write_report) = plan_graph_run(&root, &["--write-manifest"]);
        let (code, report) = plan_graph_run(&root, &["--check-manifest"]);
        clean(&root);
        assert_eq!(written, 0, "the write run is clean: {write_report}");
        assert!(
            write_report.contains("MANIFEST: plan-graph.md written"),
            "the write run states that it wrote the manifest: {write_report}"
        );
        assert_eq!(
            code, 0,
            "a manifest the front matter generates is clean: {report}"
        );
        assert!(
            report.contains("FINDINGS: 0"),
            "the summary line counts no finding: {report}"
        );
    }

    #[test]
    fn plan_graph_manifest_that_a_changed_chunk_leaves_behind_is_a_finding() {
        let root = plan(
            "pg-manifest-drift",
            &manifest_architecture(),
            &manifest_chunks("crates/bc_x/beta/lang_rust/src/b.rs"),
        );
        let (written, write_report) = plan_graph_run(&root, &["--write-manifest"]);
        write_bytes(
            &root.join("b2.md"),
            chunk(
                "B2",
                "core",
                &["A1"],
                &["crates/bc_x/beta/lang_rust/src/moved.rs"],
            )
            .as_bytes(),
        );
        let (code, report) = plan_graph_run(&root, &["--check-manifest"]);
        clean(&root);
        assert_eq!(written, 0, "the write run is clean: {write_report}");
        assert_eq!(
            code, 1,
            "a manifest the front matter no longer generates is a finding: {report}"
        );
        assert!(
            report.contains("FINDING:") && report.contains("plan-graph.md is out of step"),
            "the finding names the file and the front matter: {report}"
        );
        assert!(
            report.contains("FINDINGS: 1"),
            "the summary line counts the drift: {report}"
        );
    }

    #[test]
    fn plan_graph_manifest_edited_by_hand_is_a_finding() {
        let root = plan(
            "pg-manifest-edited",
            &manifest_architecture(),
            &manifest_chunks("crates/bc_x/beta/lang_rust/src/b.rs"),
        );
        let (written, write_report) = plan_graph_run(&root, &["--write-manifest"]);
        let manifest = root.join("plan-graph.md");
        let text = fs::read_to_string(&manifest).expect("the write run left a manifest");
        write_bytes(
            &manifest,
            format!("{text}A hand-written line.\n").as_bytes(),
        );
        let (code, report) = plan_graph_run(&root, &["--check-manifest"]);
        clean(&root);
        assert_eq!(written, 0, "the write run is clean: {write_report}");
        assert_eq!(code, 1, "a hand-edited manifest is a finding: {report}");
        assert!(
            report.contains("plan-graph.md is out of step"),
            "the finding names the file: {report}"
        );
    }

    #[test]
    fn plan_graph_manifest_that_is_not_there_is_a_finding() {
        let root = plan(
            "pg-manifest-absent",
            &manifest_architecture(),
            &manifest_chunks("crates/bc_x/beta/lang_rust/src/b.rs"),
        );
        let (code, report) = plan_graph_run(&root, &["--check-manifest"]);
        clean(&root);
        assert_eq!(
            code, 1,
            "a manifest that is not there is a finding: {report}"
        );
        assert!(
            report.contains("plan-graph.md is out of step"),
            "the finding names the file the run expected: {report}"
        );
    }

    #[test]
    fn plan_graph_a_finding_of_the_rules_hides_the_manifest_check() {
        let arch = architecture(&[(1, "none", "A1")], &[]);
        let root = plan(
            "pg-manifest-after-finding",
            &arch,
            &[(
                "a1.md",
                chunk(
                    "A1",
                    "core",
                    &["Z9"],
                    &["crates/bc_x/alpha/lang_rust/src/a.rs"],
                ),
            )],
        );
        let (code, report) = plan_graph_run(&root, &["--check-manifest"]);
        clean(&root);
        assert_eq!(code, 1, "a rule finding is a finding: {report}");
        assert!(
            report.contains("FINDING: A1 depends on Z9, which has no chunk file"),
            "the rules report first: {report}"
        );
        assert!(
            !report.contains("out of step"),
            "a run that already has a finding reads no manifest: {report}"
        );
    }

    #[test]
    fn plan_graph_both_manifest_flags_fail_closed() {
        let root = plan(
            "pg-manifest-both",
            &manifest_architecture(),
            &manifest_chunks("crates/bc_x/beta/lang_rust/src/b.rs"),
        );
        let (code, report) = plan_graph_run(&root, &["--write-manifest", "--check-manifest"]);
        let written = root.join("plan-graph.md").exists();
        clean(&root);
        assert_eq!(code, 2, "two manifest flags are fail-closed: {report}");
        assert!(
            report.contains("FAIL: give --write-manifest or --check-manifest, not both"),
            "the guard states the usage rule on stdout: {report}"
        );
        assert!(!written, "a refused run writes no manifest: {report}");
    }

    #[test]
    fn plan_graph_front_matter_the_parser_rejects_is_a_finding() {
        let arch = architecture(&[(1, "none", "A1")], &[]);
        let (code, report) = plan_graph(
            "pg-front-matter",
            &arch,
            &[
                (
                    "a1.md",
                    chunk("A1", "core", &[], &["crates/bc_x/alpha/lang_rust/src/a.rs"]),
                ),
                (
                    "broken.md",
                    "---\nid: B1\nline: edge\n\n# B1 opens a fence it never closes\n".to_owned(),
                ),
            ],
        );
        assert_eq!(
            code, 1,
            "a front-matter fence the parser rejects is a finding: {report}"
        );
        assert!(
            report.contains("FINDING: broken.md opens front matter the guard cannot parse"),
            "the finding names the file: {report}"
        );
        assert!(
            report.contains("CHUNKS: 1   PHASES: 1   LINKS 13.4: 0   FINDINGS: 1"),
            "the malformed file counts as a finding and as no chunk: {report}"
        );
    }

    #[test]
    fn plan_graph_document_front_matter_stays_silent() {
        let arch = architecture(&[(1, "none", "A1")], &[]);
        let (code, report) = plan_graph(
            "pg-document-front-matter",
            &arch,
            &[
                (
                    "a1.md",
                    chunk("A1", "core", &[], &["crates/bc_x/alpha/lang_rust/src/a.rs"]),
                ),
                (
                    "notes.md",
                    "---\ntitle: Notes\nauthor: A probe\n---\n\n# Notes\n".to_owned(),
                ),
                (
                    "rule.md",
                    "---\n\nA horizontal rule opens this file.\n".to_owned(),
                ),
            ],
        );
        assert_eq!(
            code, 0,
            "a fence that states no chunk key breaks no chunk rule: {report}"
        );
        assert!(
            report.contains("CHUNKS: 1   PHASES: 1   LINKS 13.4: 0   FINDINGS: 0"),
            "the guard reads both files and says nothing about them: {report}"
        );
    }

    #[test]
    fn plan_graph_chunk_key_in_a_rejected_fence_is_a_finding() {
        let arch = architecture(&[(1, "none", "A1")], &[]);
        let (code, report) = plan_graph(
            "pg-partial-chunk-key",
            &arch,
            &[
                (
                    "a1.md",
                    chunk("A1", "core", &[], &["crates/bc_x/alpha/lang_rust/src/a.rs"]),
                ),
                (
                    "partial.md",
                    "---\nid: B1\ntitle: A chunk that states four keys too few\n---\n\n# B1\n"
                        .to_owned(),
                ),
            ],
        );
        assert_eq!(
            code, 1,
            "a fence that states a chunk key is held to the chunk contract: {report}"
        );
        assert!(
            report.contains("FINDING: partial.md opens front matter the guard cannot parse"),
            "the finding names the file: {report}"
        );
        assert!(
            report.contains("CHUNKS: 1   PHASES: 1   LINKS 13.4: 0   FINDINGS: 1"),
            "the rejected file counts as a finding and as no chunk: {report}"
        );
    }

    #[test]
    fn plan_graph_file_with_no_front_matter_stays_silent() {
        let arch = architecture(&[(1, "none", "A1")], &[]);
        let (code, report) = plan_graph(
            "pg-no-front-matter",
            &arch,
            &[
                (
                    "a1.md",
                    chunk("A1", "core", &[], &["crates/bc_x/alpha/lang_rust/src/a.rs"]),
                ),
                (
                    "notes.md",
                    "# Notes\n\nThis file is no chunk and opens no fence.\n".to_owned(),
                ),
            ],
        );
        assert_eq!(code, 0, "a file with no fence is no chunk: {report}");
        assert!(
            report.contains("CHUNKS: 1   PHASES: 1   LINKS 13.4: 0   FINDINGS: 0"),
            "the guard reads the file and says nothing about it: {report}"
        );
    }

    /// How many chunks the deep-chain probe writes.
    ///
    /// A recursive walk of this chain overflows the process stack and aborts
    /// with a status outside the three the guard contract states. The walk the
    /// guard runs holds its own stack, so the depth reaches a verdict.
    const DEEP_CHAIN: usize = 40_000;

    /// Run the plan-graph guard over one chunk whose write scope holds one path.
    fn plan_graph_scope(label: &str, path: &str) -> (i32, String) {
        let arch = architecture(&[(1, "none", "A1")], &[]);
        plan_graph(
            label,
            &arch,
            &[("a1.md", chunk("A1", "core", &[], &[path]))],
        )
    }

    /// The finding check 9 prints for one chunk and one path.
    fn scope_shape_finding(path: &str) -> String {
        format!(
            "FINDING: A1 writes {path}, which is not `<crates|tools>/bc_<context>/<package>/` \
followed by `lang_rust/` or a unit sibling"
        )
    }

    #[test]
    fn plan_graph_check9_path_under_lang_rust_is_clean() {
        let (code, report) = plan_graph_scope("pg9-rust", "crates/bc_x/duet-aa/lang_rust/src/x.rs");
        assert_eq!(
            code, 0,
            "a path under `lang_rust/` is in the unit shape: {report}"
        );
    }

    #[test]
    fn plan_graph_check9_unit_sibling_is_clean() {
        let (code, report) = plan_graph_scope("pg9-asset", "crates/bc_x/duet-aa/assets/y.otf");
        assert_eq!(
            code, 0,
            "a path under `assets/` is a unit sibling: {report}"
        );
    }

    #[test]
    fn plan_graph_check9_path_with_no_context_is_a_finding() {
        let path = "crates/duet-aa/src/x.rs";
        let (code, report) = plan_graph_scope("pg9-old", path);
        assert_eq!(
            code, 1,
            "a path with no context directory is a finding: {report}"
        );
        assert!(
            report.contains(&scope_shape_finding(path)),
            "the finding names the chunk and the path: {report}"
        );
    }

    #[test]
    fn plan_graph_check9_source_outside_lang_rust_is_a_finding() {
        let path = "crates/bc_x/duet-aa/src/x.rs";
        let (code, report) = plan_graph_scope("pg9-src", path);
        assert_eq!(
            code, 1,
            "a source file beside `lang_rust/` is a finding: {report}"
        );
        assert!(
            report.contains(&scope_shape_finding(path)),
            "the finding names the chunk and the path: {report}"
        );
    }

    #[test]
    fn plan_graph_check9_tool_with_no_context_is_a_finding() {
        let path = "tools/xtask/src/x.rs";
        let (code, report) = plan_graph_scope("pg9-tool", path);
        assert_eq!(
            code, 1,
            "a tool path with no context directory is a finding: {report}"
        );
        assert!(
            report.contains(&scope_shape_finding(path)),
            "the finding names the chunk and the path: {report}"
        );
    }

    #[test]
    fn plan_graph_deep_dependency_chain_reaches_a_verdict() {
        let root = scratch("pg-deep");
        write_bytes(
            &root.join("architecture.md"),
            architecture(&[(1, "none", "A1")], &[]).as_bytes(),
        );
        for index in 0..DEEP_CHAIN {
            let next = index + 1;
            let depends = if next < DEEP_CHAIN {
                format!("D{next}")
            } else {
                String::new()
            };
            let scope = format!("crates/bc_x/c{index}/lang_rust/src/lib.rs");
            let text = format!(
                "---\nid: D{index}\nline: core\ndepends_on: [{depends}]\n\
                 write_scope: [{scope}]\nparallelism: independent\n\
                 completion: cargo nextest run passes\n---\n\n# D{index}\n"
            );
            fs::write(root.join(format!("d{index:07}.md")), text).expect("chain chunk file");
        }
        let (code, report) = plan_graph_run(&root, &[]);
        clean(&root);
        let summary = report.lines().last().unwrap_or_default().to_owned();
        assert_eq!(
            code, 1,
            "a chain of {DEEP_CHAIN} chunks reaches a verdict: {summary}"
        );
        assert!(
            summary.starts_with(&format!("CHUNKS: {DEEP_CHAIN}")),
            "the summary line counts every chunk of the chain: {summary}"
        );
    }

    /// The throwaway review every closure probe writes, byte for byte.
    ///
    /// The id generator reads the headings alone, so the file needs no body.
    /// The three digests below are the md5 of exactly these bytes, so a change
    /// of one character here turns every closure probe red.
    const CLOSURE_REVIEW: &str = "# Engineering Critic, specification review, revision 99

**Counts: 2 Criticals, 1 Warnings, 1 Concerns.**

## CRITICAL 1. A probe

## CRITICAL 2. A probe

## WARNING 1. A probe

## CONCERN 1. A probe
";

    /// The md5 of the review file, which the block records (`CL5`).
    const CLOSURE_FILE_DIGEST: &str = "8bf1ef7216b17e25853e7b0bcf327869";

    /// The md5 of the review with its trailing newlines removed (`CL1c`).
    const CLOSURE_CONTENT_DIGEST: &str = "4b78ae2b0f7a7556e28f8d92c7dedf88";

    /// The md5 of the doctored copy with its trailing newlines removed.
    const CLOSURE_DOCTORED_DIGEST: &str = "da764fb9652cb2803ab6197619ca7238";

    /// The count sentence the throwaway review generates (`CL3`).
    const CLOSURE_SENTENCE: &str = "It returned 2 Criticals, 1 Warnings, and 1 Concerns, and this block holds one row for each of the 4.";

    /// The registered block every closure probe runs.
    const CLOSURE_BLOCK: &str = "closure-r16";

    /// The parent the guard puts before the plan directory of a fixture.
    ///
    /// The guard reads the directory that holds the document, so each fixture
    /// names that directory after its own token. The plan KEY therefore
    /// differs per probe, and two probes cannot resolve to one store
    /// environment even if `PLAN_DB_ROOT` reaches neither of them.
    const CLOSURE_PLAN_PARENT: &str = "roadmap";

    /// A plan-store key that carries the suffix of another review (`CL1c`).
    const CLOSURE_OTHER_KEY: &str = "36304kihouge4-review-r21-inner";

    /// The plan-store suffix one recorded `check-closure` run carries (`CL1d`).
    const CLOSURE_RUN_SUFFIX: &str = "closure-run";

    /// A key that carries the run suffix and that no store answers (`CL1d`).
    const CLOSURE_ABSENT_RUN_KEY: &str = "11111absentrun1-closure-run";

    /// The revision a review states when it sits below the `CL1d` cut-off.
    ///
    /// The review bytes stay the ones the three digest constants record, and
    /// the file name alone states the revision, so the id prefix and the store
    /// suffix both follow the name.
    const CLOSURE_BELOW_REVISION: &str = "23";

    /// The recorded body of one `check-closure` run, as `CL1d` shapes it.
    ///
    /// The body opens with `COMMAND: `, `EXIT: ` and `TREE: `, one per line,
    /// and then holds the stdout of the run.
    fn recorded_run(exit: i32) -> String {
        format!(
            "COMMAND: cargo xtask check-closure architecture.md review.md {CLOSURE_BLOCK}\n\
             EXIT: {exit}\n\
             TREE: 0123456789abcdef0123456789abcdef01234567\n\
             REVIEW: a probe   BLOCK: {CLOSURE_BLOCK}   CLOSURE BAD: 0\n"
        )
    }

    /// The one line that binds the block to the run that verified it (`CL1d`).
    fn run_key_line(key: &str) -> String {
        format!(
            "The `check-closure` run that verified this block is recorded at plan-store key `{key}`.\n"
        )
    }

    /// The repository the closure guard resolves at compile time.
    ///
    /// The guard reads `.claude/plan-coordination/db.sh` of the repository
    /// that holds the workspace lockfile above its own crate, so a probe of
    /// `CL1c` resolves the same launcher the same way.
    fn guard_repo() -> PathBuf {
        repository_root()
    }

    /// A run token no other closure probe shares.
    ///
    /// The token opens with a letter after the revision number, because the id
    /// generator reads a digit run there as a longer revision.
    fn run_token() -> String {
        format!("r99z{}z{}", *PROCESS_TAG, next_serial())
    }

    /// Append one value to the throwaway store and return the key it minted.
    fn store_append(root: &Path, plan: &str, suffix: &str, value: &str) -> String {
        let launcher = guard_repo()
            .join(".claude")
            .join("plan-coordination")
            .join("db.sh");
        let output = Command::new("bash")
            .arg(&launcher)
            .args(["append", plan, suffix, value])
            .current_dir(guard_repo())
            .env("PLAN_DB_ROOT", root.join("store"))
            .output()
            .expect("spawn the plan-store launcher");
        assert!(
            output.status.success(),
            "the throwaway store takes the review: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("utf-8 key")
            .trim()
            .to_owned()
    }

    /// Run the `xtask` binary against one throwaway store.
    fn xtask_with_store(root: &Path, args: &[&str]) -> (i32, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(args)
            .current_dir(root)
            .env("PLAN_DB_ROOT", root.join("store"))
            .output()
            .expect("spawn xtask");
        let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
        (output.status.code().unwrap_or(-1), stdout)
    }

    /// The one sentence that binds the block to the review file (`CL5`).
    fn digest_sentence() -> String {
        format!(
            "The review file this block records has md5 `{CLOSURE_FILE_DIGEST}` ({CLOSURE_BLOCK})."
        )
    }

    /// The whole throwaway closure section, which records the minted keys.
    ///
    /// The `run` line is the whole `CL1d` run-key line, or an empty text for a
    /// section that records no run.
    fn closure_block(key: &str, run: &str) -> String {
        format!(
            "### 1.9 A probe section\n\n\
             #### The revision-16 review, over the frozen document\n\n\
             {CLOSURE_SENTENCE}\n\n\
             {}\n\
             The review file this block records is stored at plan-store key `{key}` ({CLOSURE_BLOCK}).\n\
             {run}\n\
             <!-- GUARD BLOCK id={CLOSURE_BLOCK} rows>=4 -->\n\
             | Id | Finding | State | Section and mechanism |\n\
             |---|---|---|---|\n\
             | C99-1 | A probe | **CLOSED** | 1.9 |\n\
             | C99-2 | A probe | **CLOSED** | 1.9 |\n\
             | C99-W1 | A probe | **CLOSED** | 1.9 |\n\
             | N99-1 | A probe | **CLOSED** | 1.9 |\n",
            digest_sentence()
        )
    }

    /// One throwaway closure fixture: its own store, document, and review.
    #[derive(Debug)]
    struct Closure {
        /// The scratch root this fixture owns.
        root: PathBuf,
        /// The token the review file name and the store suffix carry.
        token: String,
        /// The plan-store key the store minted for the review.
        key: String,
        /// The plan-store key the store minted for the recorded run (`CL1d`).
        run_key: String,
    }

    impl Closure {
        /// Write the review, append it to a throwaway store, and mint its key.
        ///
        /// The fixture also records one `check-closure` run that exited 0, so
        /// the baseline section cites a run the store answers.
        fn new(label: &str) -> Self {
            let root = scratch(label);
            let token = run_token();
            let review = root.join("review").join(format!("critic-spec-{token}.md"));
            write_bytes(&review, CLOSURE_REVIEW.as_bytes());
            let plan = format!("{CLOSURE_PLAN_PARENT}/plan-{token}");
            let key = store_append(&root, &plan, &format!("review-{token}"), CLOSURE_REVIEW);
            let run_key = store_append(&root, &plan, CLOSURE_RUN_SUFFIX, &recorded_run(0));
            Self {
                root,
                token,
                key,
                run_key,
            }
        }

        /// The plan directory this fixture owns, under its scratch root.
        fn plan_dir(&self) -> String {
            format!("plan-{}", self.token)
        }

        /// The plan name `db.sh` reads for this fixture.
        fn plan(&self) -> String {
            format!("{CLOSURE_PLAN_PARENT}/{}", self.plan_dir())
        }

        /// The review file this fixture wrote.
        fn review(&self) -> PathBuf {
            self.root
                .join("review")
                .join(format!("critic-spec-{}.md", self.token))
        }

        /// The plan-store suffix the review file name decides.
        fn suffix(&self) -> String {
            format!("review-{}", self.token)
        }

        /// The baseline closure section, which records the minted keys.
        fn block(&self) -> String {
            closure_block(&self.key, &run_key_line(&self.run_key))
        }

        /// Append one more copy of the review under this fixture's suffix.
        fn append(&self, value: &str) -> String {
            store_append(&self.root, &self.plan(), &self.suffix(), value)
        }

        /// Record one more `check-closure` run and return the key it minted.
        fn append_run(&self, body: &str) -> String {
            store_append(&self.root, &self.plan(), CLOSURE_RUN_SUFFIX, body)
        }

        /// A second review of this fixture, below the `CL1d` cut-off.
        ///
        /// The bytes are the ones the digest constants record and the file
        /// name states the revision, so the id prefix reads `C23` and the
        /// store suffix reads `review-r23`. The fixture owns its own scratch
        /// root and its own plan, so one name for every probe is still one
        /// store environment per probe.
        fn below_cut_off(&self) -> (PathBuf, String) {
            let name = format!("critic-spec-r{CLOSURE_BELOW_REVISION}.md");
            let path = self.root.join("below").join(name);
            write_bytes(&path, CLOSURE_REVIEW.as_bytes());
            let suffix = format!("review-r{CLOSURE_BELOW_REVISION}");
            let key = store_append(&self.root, &self.plan(), &suffix, CLOSURE_REVIEW);
            (path, key)
        }

        /// Run the guard over one document text and this fixture's review.
        fn guard(&self, document: &str) -> (i32, String) {
            self.guard_with(document, &self.review(), CLOSURE_BLOCK)
        }

        /// Run the guard over one document text, review path, and block id.
        fn guard_with(&self, document: &str, review: &Path, block: &str) -> (i32, String) {
            let path = self.root.join(self.plan_dir()).join("architecture.md");
            write_bytes(&path, document.as_bytes());
            let named = path.display().to_string();
            let read = review.display().to_string();
            xtask_with_store(
                &self.root,
                &["check-closure", named.as_str(), read.as_str(), block],
            )
        }
    }

    /// Plant one document shape over a fresh fixture, run it, and clean up.
    fn planted(label: &str, plant: impl Fn(&str) -> String) -> (i32, String) {
        let fixture = Closure::new(label);
        let report = fixture.guard(&plant(&fixture.block()));
        clean(&fixture.root);
        report
    }

    #[test]
    fn fixture_scratch_never_repeats_one_path_for_one_label() {
        let first = scratch("fixture-repeat");
        let second = scratch("fixture-repeat");
        assert_ne!(
            first, second,
            "two scratch calls under one label give two paths"
        );
        clean(&first);
        clean(&second);
    }

    #[test]
    fn fixture_closure_plan_key_carries_the_token_of_its_own_probe() {
        let first = Closure::new("fixture-plan-first");
        let second = Closure::new("fixture-plan-second");
        let names = (first.plan(), second.plan());
        let same_root = first.root == second.root;
        clean(&first.root);
        clean(&second.root);
        assert_ne!(
            names.0, names.1,
            "two fixtures give two plan names, so two probes cannot resolve to one store"
        );
        assert!(
            !same_root,
            "two fixtures give two scratch roots, so PLAN_DB_ROOT names two parents"
        );
    }

    #[test]
    fn closure_baseline_is_clean() {
        let (code, report) = planted("cl-base", str::to_owned);
        assert_eq!(code, 0, "a well-formed throwaway pair is clean: {report}");
        assert!(
            report.starts_with("REVIEW: critic-spec-r99z"),
            "the summary names the review this run read: {report}"
        );
        assert!(
            report.contains(
                "BLOCK: closure-r16   GENERATED: 4   ROWS: 4   STORE: matches   STORE COPIES: 1   CLOSURE BAD: 0   RUN KEY: verified"
            ),
            "the summary states the generated set, the rows, the stored copy, and the recorded run: {report}"
        );
    }

    #[test]
    fn closure_pp32_missing_row_is_a_finding() {
        let (code, report) = planted("cl-missing", |block| {
            block
                .replace("| C99-2 | A probe | **CLOSED** | 1.9 |\n", "")
                .replace("rows>=4", "rows>=3")
        });
        assert_eq!(code, 1, "a generated id with no row is a finding: {report}");
        assert!(
            report.contains("  CLOSURE:   C99-2: the review states it and the block holds no row"),
            "the finding names the id the block drops: {report}"
        );
    }

    #[test]
    fn closure_pp32_extra_row_is_a_finding() {
        let (code, report) = planted("cl-extra", |block| {
            block
                .replace(
                    "| N99-1 | A probe | **CLOSED** | 1.9 |",
                    "| N99-1 | A probe | **CLOSED** | 1.9 |\n| C99-9 | A probe | **CLOSED** | 1.9 |",
                )
                .replace("rows>=4", "rows>=5")
        });
        assert_eq!(code, 1, "a row for no finding is a finding: {report}");
        assert!(
            report.contains(
                "  CLOSURE:   C99-9: the block holds a row and the review states no such finding"
            ),
            "the finding names the id the review does not state: {report}"
        );
    }

    #[test]
    fn closure_pp32_wrong_count_sentence_is_a_finding() {
        let (code, report) = planted("cl-count", |block| {
            block.replace("It returned 2 Criticals", "It returned 1 Criticals")
        });
        assert_eq!(code, 1, "a count the review does not generate: {report}");
        assert!(
            report.contains(&format!(
                "  COUNT:     the document does not hold the count sentence this review generates: {CLOSURE_SENTENCE}"
            )),
            "the finding prints the sentence the review generates: {report}"
        );
    }

    #[test]
    fn closure_pp32_closed_row_with_an_empty_section_is_a_finding() {
        let (code, report) = planted("cl-nosection", |block| {
            block.replace(
                "| C99-1 | A probe | **CLOSED** | 1.9 |",
                "| C99-1 | A probe | **CLOSED** |  |",
            )
        });
        assert_eq!(
            code, 1,
            "a CLOSED row with no section is a finding: {report}"
        );
        assert!(
            report.contains("  CLOSURE:   C99-1: the row says CLOSED and names no section"),
            "the finding names the row that closes on nothing: {report}"
        );
    }

    #[test]
    fn closure_pp32_wrong_digest_is_a_finding() {
        let (code, report) = planted("cl-digest", |block| {
            block.replace(CLOSURE_FILE_DIGEST, &"0".repeat(32))
        });
        assert_eq!(code, 1, "a digest of another file is a finding: {report}");
        assert!(
            report.contains(&format!(
                "  DIGEST:    the document states no md5 `{CLOSURE_FILE_DIGEST}` for {CLOSURE_BLOCK} beside the block, so nothing binds this block to the file this run read"
            )),
            "the finding prints the md5 of the file this run read: {report}"
        );
    }

    #[test]
    fn closure_pp32_state_outside_the_three_is_a_finding() {
        let (code, report) = planted("cl-state", |block| {
            block.replace(
                "| C99-2 | A probe | **CLOSED** | 1.9 |",
                "| C99-2 | A probe | **PENDING** | 1.9 |",
            )
        });
        assert_eq!(code, 1, "a fourth state word is a finding: {report}");
        assert!(
            report.contains(
                "  CLOSURE:   C99-2: the state is `PENDING` and the three states are CLOSED, PARTIAL, OPEN"
            ),
            "the finding prints the state and the three it accepts: {report}"
        );
    }

    #[test]
    fn closure_pp32_blank_row_is_a_finding() {
        let (code, report) = planted("cl-blank", |block| {
            block.replace(
                "| C99-W1 | A probe | **CLOSED** | 1.9 |",
                "| C99-W1 |  |  |  |",
            )
        });
        assert_eq!(code, 1, "a row that says nothing is a finding: {report}");
        assert!(
            report.contains("  CLOSURE:   C99-W1: the row states no finding"),
            "the finding names the empty row: {report}"
        );
    }

    #[test]
    fn closure_pp32_unknown_section_is_a_finding() {
        let (code, report) = planted("cl-section", |block| {
            block.replace(
                "| N99-1 | A probe | **CLOSED** | 1.9 |",
                "| N99-1 | A probe | **CLOSED** | 99.99 |",
            )
        });
        assert_eq!(
            code, 1,
            "a section no heading states is a finding: {report}"
        );
        assert!(
            report.contains(
                "  CLOSURE:   N99-1: the row names section 99.99 and no heading of this document states it"
            ),
            "the finding names the section the document lacks: {report}"
        );
    }

    #[test]
    fn closure_pp32_duplicate_row_is_a_finding() {
        let (code, report) = planted("cl-duplicate", |block| {
            block.replace(
                "| C99-W1 | A probe | **CLOSED** | 1.9 |",
                "| C99-W1 | A probe | **CLOSED** | 1.9 |\n| C99-W1 | A probe | **OPEN** | 1.9 |",
            )
        });
        assert_eq!(
            code, 1,
            "a second row for one finding is a finding: {report}"
        );
        assert!(
            report.contains(
                "  CLOSURE:   C99-W1: the block holds 2 rows and PG32 states one closure row per finding"
            ),
            "the finding counts the rows the id carries: {report}"
        );
    }

    #[test]
    fn closure_pp32_prose_section_cell_is_a_finding() {
        let (code, report) = planted("cl-prose", |block| {
            block.replace(
                "| C99-1 | A probe | **CLOSED** | 1.9 |",
                "| C99-1 | A probe | **CLOSED** | fixed in the tool |",
            )
        });
        assert_eq!(
            code, 1,
            "a cell that names no section is a finding: {report}"
        );
        assert!(
            report.contains(
                "  CLOSURE:   C99-1: the row says CLOSED and its section cell names no section of this document"
            ),
            "the finding names the row whose cell is prose: {report}"
        );
    }

    #[test]
    fn closure_pp32_moved_digest_sentence_is_a_finding() {
        let (code, report) = planted("cl-moved", |block| {
            let sentence = digest_sentence();
            format!(
                "{}\n### A section far away\n\n{sentence}\n",
                block.replace(&format!("{sentence}\n"), "")
            )
        });
        assert_eq!(
            code, 1,
            "a digest sentence outside the block's section is a finding: {report}"
        );
        assert!(
            report.contains(&format!(
                "  DIGEST:    the document states no md5 `{CLOSURE_FILE_DIGEST}` for {CLOSURE_BLOCK} beside the block, so nothing binds this block to the file this run read"
            )),
            "the guard reads the block's own section alone: {report}"
        );
    }

    #[test]
    fn closure_pp32_deleted_store_key_is_a_finding() {
        let (code, report) = planted("cl-nokey", |block| {
            let kept: Vec<&str> = block
                .lines()
                .filter(|line| !line.contains("is stored at plan-store key"))
                .collect();
            format!("{}\n", kept.join("\n"))
        });
        assert_eq!(
            code, 1,
            "a block that names no stored copy is a finding: {report}"
        );
        assert!(
            report.contains(&format!(
                "  STORE:     the section states no plan-store key for {CLOSURE_BLOCK}, so nothing binds this block to a copy outside the Architect's write scope (CL1c)"
            )),
            "the finding states that CL1c has no second source: {report}"
        );
    }

    #[test]
    fn closure_pp32_key_of_another_suffix_is_a_finding() {
        let fixture = Closure::new("cl-otherkey");
        let planted_block = fixture.block().replace(
            &format!("`{}`", fixture.key),
            &format!("`{CLOSURE_OTHER_KEY}`"),
        );
        let (code, report) = fixture.guard(&planted_block);
        let suffix = fixture.suffix();
        clean(&fixture.root);
        assert_eq!(
            code, 1,
            "a key outside the review's suffix is a finding: {report}"
        );
        assert!(
            report.contains(&format!(
                "  STORE:     the section names key `{CLOSURE_OTHER_KEY}`, whose suffix is not `{suffix}`; the review file name decides the suffix and the Architect does not (CL1c)"
            )),
            "the finding states that the file name decides the suffix: {report}"
        );
    }

    #[test]
    fn closure_pp32_second_stored_copy_is_a_finding() {
        let fixture = Closure::new("cl-reappend");
        let doctored =
            CLOSURE_REVIEW.replace("## CONCERN 1. A probe", "## CONCERN 1. A doctored probe");
        let second = fixture.append(&doctored);
        let (code, report) = fixture.guard(&fixture.block());
        clean(&fixture.root);
        assert_eq!(
            code, 1,
            "a second stored copy that differs is a finding: {report}"
        );
        assert!(
            report.contains(&format!(
                "  STORE:     1 of 2 stored copies of this review differ from the file this run read, `{CLOSURE_CONTENT_DIGEST}`; the first is key `{second}` at md5 `{CLOSURE_DOCTORED_DIGEST}` (CL1c)"
            )),
            "the finding names the doctored copy and both digests: {report}"
        );
    }

    #[test]
    fn closure_run_key_absent_is_a_finding() {
        let fixture = Closure::new("cl-run-absent");
        let planted_block = closure_block(&fixture.key, "");
        let (code, report) = fixture.guard(&planted_block);
        clean(&fixture.root);
        assert_eq!(
            code, 1,
            "a section at the cut-off that cites no run is a finding: {report}"
        );
        assert!(
            report.contains(
                "  RUN:       the section states no plan-store key for the `check-closure` run that verified it, so the verdict is a claim in prose that no later party can reproduce (CL1d)"
            ),
            "the finding states that nothing records the run: {report}"
        );
        assert!(
            report.contains("RUN KEY: unrecorded"),
            "the summary names the state of the run half: {report}"
        );
    }

    #[test]
    fn closure_run_key_wrong_suffix_is_a_finding() {
        let fixture = Closure::new("cl-run-suffix");
        let planted_block = closure_block(&fixture.key, &run_key_line(&fixture.key));
        let (code, report) = fixture.guard(&planted_block);
        let named = fixture.key.clone();
        clean(&fixture.root);
        assert_eq!(
            code, 1,
            "a run key outside the run suffix is a finding: {report}"
        );
        assert!(
            report.contains(&format!(
                "  RUN:       the section names key `{named}`, whose suffix is not `{CLOSURE_RUN_SUFFIX}`; the store mints that suffix for a recorded run and for no other record (CL1d)"
            )),
            "the finding names the key and the suffix a run carries: {report}"
        );
    }

    #[test]
    fn closure_run_key_store_does_not_answer_is_a_finding() {
        let fixture = Closure::new("cl-run-silent");
        let planted_block = closure_block(&fixture.key, &run_key_line(CLOSURE_ABSENT_RUN_KEY));
        let (code, report) = fixture.guard(&planted_block);
        clean(&fixture.root);
        assert_eq!(
            code, 1,
            "a run key the store does not hold is a finding: {report}"
        );
        assert!(
            report.contains(&format!("  RUN:       key `{CLOSURE_ABSENT_RUN_KEY}`:"))
                && report.contains("; CL1d is fail-closed"),
            "the finding names the key the store does not answer: {report}"
        );
    }

    #[test]
    fn closure_run_key_failed_run_is_a_finding() {
        let fixture = Closure::new("cl-run-failed");
        let failed = fixture.append_run(&recorded_run(1));
        let planted_block = closure_block(&fixture.key, &run_key_line(&failed));
        let (code, report) = fixture.guard(&planted_block);
        clean(&fixture.root);
        assert_eq!(code, 1, "a recorded run that failed is a finding: {report}");
        assert!(
            report.contains(&format!(
                "  RUN:       key `{failed}`: the recorded run states `EXIT: 1`, and a closure section cites a run that exited 0 (CL1d)"
            )),
            "the finding states the exit the record holds: {report}"
        );
    }

    #[test]
    fn closure_run_key_below_the_cut_off_is_clean() {
        let fixture = Closure::new("cl-run-below");
        let (review, stored) = fixture.below_cut_off();
        let planted_block = closure_block(&stored, "")
            .replace("| C99-", "| C23-")
            .replace("| N99-", "| N23-");
        let (code, report) = fixture.guard_with(&planted_block, &review, CLOSURE_BLOCK);
        clean(&fixture.root);
        assert_eq!(
            code, 0,
            "a section below the cut-off cites no run and is clean: {report}"
        );
        assert!(
            report.contains("CLOSURE BAD: 0   RUN KEY: absent"),
            "the summary states that the cut-off holds the run half back: {report}"
        );
    }

    #[test]
    fn closure_pp32_hidden_warning_review_fails_closed() {
        let fixture = Closure::new("cl-format");
        let hidden = format!(
            "{}\n### Warning two\n\nA probe.\n",
            CLOSURE_REVIEW.replace(
                "**Counts: 2 Criticals, 1 Warnings, 1 Concerns.**",
                "**Counts: 2 Criticals, 2 Warnings, 1 Concerns.**",
            )
        );
        let review = fixture.root.join("hidden").join("critic-spec-r99.md");
        write_bytes(&review, hidden.as_bytes());
        let (code, report) = fixture.guard_with(&fixture.block(), &review, CLOSURE_BLOCK);
        clean(&fixture.root);
        assert_eq!(
            code, 2,
            "a heading no form of the parser reads is fail-closed: {report}"
        );
        assert!(
            report.contains(
                ": the review states 2 Warnings and the heading scan finds 1; a heading this parser cannot see is the one failure a second source exists to catch; the guard is fail-closed."
            ),
            "the guard states what the second source caught: {report}"
        );
    }

    #[test]
    fn closure_review_that_does_not_open_fails_closed() {
        let fixture = Closure::new("cl-absent");
        let absent = fixture.root.join("review").join("no-such-review.md");
        let (code, report) = fixture.guard_with(&fixture.block(), &absent, CLOSURE_BLOCK);
        clean(&fixture.root);
        assert_eq!(
            code, 2,
            "a review that does not open is fail-closed: {report}"
        );
        assert!(
            report.contains("FAIL: cannot open")
                && report.contains("no-such-review.md; the guard is fail-closed."),
            "the guard names the file it cannot open: {report}"
        );
    }

    #[test]
    fn closure_unregistered_block_fails_closed() {
        let fixture = Closure::new("cl-unregistered");
        let (code, report) = fixture.guard_with(&fixture.block(), &fixture.review(), "closure-r99");
        clean(&fixture.root);
        assert_eq!(
            code, 2,
            "a block id no register holds is fail-closed: {report}"
        );
        assert!(
            report
                .contains("FAIL: `closure-r99` is no registered block; the guard is fail-closed."),
            "the guard names the block it does not register: {report}"
        );
    }

    #[test]
    fn closure_review_that_states_no_revision_fails_closed() {
        let fixture = Closure::new("cl-norevision");
        let unnamed = fixture.root.join("review").join("notes.md");
        write_bytes(
            &unnamed,
            b"# Engineering Critic, specification review\n\n## CRITICAL 1. A probe\n",
        );
        let (code, report) = fixture.guard_with(&fixture.block(), &unnamed, CLOSURE_BLOCK);
        clean(&fixture.root);
        assert_eq!(
            code, 2,
            "a review that names no revision is fail-closed: {report}"
        );
        assert!(
            report.contains(
                ": the file name states no review and the title states no revision; the guard is fail-closed."
            ),
            "the guard states that no id prefix can be generated: {report}"
        );
    }
    /// Every crate the synthetic placement document states in section 1.2.
    fn placement_crates() -> Vec<String> {
        let mut found = vec!["duet".to_owned()];
        found.extend(
            "abcdefghijklmno"
                .chars()
                .map(|mark| format!("duet-a{mark}")),
        );
        found
    }

    /// Every third-party crate a section 1.2 dependency cell of the fixture names.
    fn placement_third_party() -> Vec<String> {
        "abcdefghijklm"
            .chars()
            .map(|mark| format!("tpa{mark}"))
            .collect()
    }

    /// One list of generated names, from a prefix, a width, and a count.
    fn placement_series(prefix: &str, width: usize, count: usize) -> Vec<String> {
        (0..count)
            .map(|index| format!("{prefix}{index:0width$}"))
            .collect()
    }

    /// Every external name the section 1.9 verdict table of the fixture decides.
    fn placement_externals() -> Vec<String> {
        placement_series("Ext", 1, 34)
    }

    /// Every framework name section 1.3 of the fixture declares.
    fn placement_framework() -> Vec<String> {
        placement_series("Fw", 1, 7)
    }

    /// Every third-party type name the section 1.2 name map of the fixture maps.
    fn placement_mapped() -> Vec<String> {
        placement_series("Qm", 2, 23)
    }

    /// The candidate drop list of the fixture.
    fn placement_dropped() -> Vec<String> {
        [
            "Clone",
            "Copy",
            "Debug",
            "Default",
            "Hash",
            "Ord",
            "PartialEq",
            "PartialOrd",
        ]
        .iter()
        .map(|one| (*one).to_owned())
        .collect()
    }

    /// Every primitive the section 1.9 trait and size blocks of the fixture name.
    fn placement_primitives() -> Vec<String> {
        [
            "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
            "f32", "f64", "bool", "char",
        ]
        .iter()
        .map(|one| (*one).to_owned())
        .collect()
    }

    /// Every chunk line of section 13 of the fixture, one letter each.
    fn placement_lines() -> Vec<String> {
        "ACDEFGHJKLMNOPQR"
            .chars()
            .map(|mark| mark.to_string())
            .collect()
    }

    /// The path every line chunk of the fixture writes, in the unit shape.
    const PLACEMENT_SCOPE: &str = "`crates/bc_app/duet/lang_rust/src/lib.rs`";

    /// The chunk row A1 of the fixture, as the baseline states it.
    const PLACEMENT_A1_ROW: &str = "| A1 | 0 | a synthetic chunk | `crates/bc_app/duet/lang_rust/src/lib.rs` | a synthetic gate |";

    /// Every crate of the fixture and the bounded context that holds it.
    fn placement_context_map() -> Vec<String> {
        placement_crates()
            .iter()
            .map(|name| {
                let context = if name == "duet" { "app" } else { "synth" };
                format!("{name} {context}")
            })
            .collect()
    }

    /// Every chunk id of section 13 of the fixture, four to a line.
    fn placement_chunks() -> Vec<String> {
        let mut found = Vec::new();
        for line in placement_lines() {
            found.extend((1..5_usize).map(|index| format!("{line}{index}")));
        }
        found
    }

    /// Every audio-owned root the fixture declares and marks.
    fn placement_roots() -> Vec<String> {
        let mut found = vec!["EngineProcess".to_owned()];
        found.extend((1..44_usize).map(|index| format!("Aud{index:02}")));
        found
    }

    /// Every audio-reachable leaf the fixture declares.
    fn placement_leaves() -> Vec<String> {
        placement_series("Leaf", 2, 46)
    }

    /// Every carrier holder the fixture declares.
    fn placement_carriers() -> Vec<String> {
        placement_series("Car", 2, 25)
    }

    /// Every type a Rust block of the fixture declares.
    fn placement_declared() -> Vec<String> {
        let mut found = placement_roots();
        found.extend(placement_leaves());
        found.extend(placement_carriers());
        found
    }

    /// Every budget the fixture cites from a declaration line, with its value.
    fn placement_budgets() -> Vec<(String, usize, String)> {
        (1..10_usize)
            .map(|index| (format!("B{index}"), 3 + index, format!("CAP_{index:03}")))
            .collect()
    }

    /// One markdown row, from its cells.
    fn md_row(cells: &[&str]) -> String {
        format!("| {} |", cells.join(" | "))
    }

    /// One markdown table, from its header and its rows.
    fn md_table(header: &[&str], rows: Vec<String>) -> Vec<String> {
        let ruler = format!("|{}|", vec!["---"; header.len()].join("|"));
        let mut found = vec![md_row(header), ruler];
        found.extend(rows);
        found
    }

    /// One filler markdown table whose first cell carries a generated label.
    fn md_filler(count: usize, columns: usize, prefix: &str) -> Vec<String> {
        let header: Vec<String> = (0..columns).map(|index| format!("Head{index}")).collect();
        let names: Vec<&str> = header.iter().map(String::as_str).collect();
        let rows = (0..count)
            .map(|index| {
                let head = format!("{prefix}{index}");
                let mut cells = vec![head];
                cells.extend(vec!["text".to_owned(); columns.saturating_sub(1)]);
                let borrowed: Vec<&str> = cells.iter().map(String::as_str).collect();
                md_row(&borrowed)
            })
            .collect();
        md_table(&names, rows)
    }

    /// One comma-separated list of code spans.
    fn md_spans(items: &[String]) -> String {
        items
            .iter()
            .map(|one| format!("`{one}`"))
            .collect::<Vec<String>>()
            .join(", ")
    }

    /// One registered block of the synthetic placement document.
    struct PlacementBlock {
        /// The id the marker line states.
        id: &'static str,
        /// The `####` heading the block sits under.
        heading: &'static str,
        /// Whether the block is a table, a text fence, or a Rust fence.
        kind: &'static str,
        /// The floor the marker states, which is the row count the block holds.
        minimum: usize,
        /// The membership rule the `block-members` block gives the block.
        member: &'static str,
        /// Every row the block holds.
        body: Vec<String>,
    }

    /// One entry of the fixture block register.
    fn pblock(
        id: &'static str,
        heading: &'static str,
        kind: &'static str,
        member: &'static str,
        body: Vec<String>,
    ) -> PlacementBlock {
        let minimum = body
            .len()
            .saturating_sub(if kind == "table" { 2 } else { 0 });
        PlacementBlock {
            id,
            heading,
            kind,
            minimum,
            member,
            body,
        }
    }

    /// The section 1.2 crate table of the fixture.
    fn placement_crate_table() -> Vec<String> {
        let third = placement_third_party();
        let joined = third
            .iter()
            .map(|one| format!("`{one}`"))
            .collect::<Vec<String>>()
            .join(" ");
        let rows = placement_crates()
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let deps = if index == 0 {
                    joined.clone()
                } else {
                    "none".to_owned()
                };
                md_row(&[&format!("`{name}`"), "member", "a synthetic member", &deps])
            })
            .collect();
        md_table(&["Crate", "Kind", "What it does", "Depends on"], rows)
    }

    /// The section 1.5 ownership table of the fixture.
    fn placement_ownership_table() -> Vec<String> {
        let placed = md_spans(&placement_declared());
        let mut rows = vec![md_row(&["`duet`", &placed, "9.1"])];
        rows.extend(
            placement_crates()
                .iter()
                .skip(1)
                .map(|name| md_row(&[&format!("`{name}`"), "none", "9.1"])),
        );
        md_table(&["Crate", "Types", "Declared in"], rows)
    }

    /// The section 1.3 edge list of the fixture.
    fn placement_edge_list() -> Vec<String> {
        let crates = placement_crates();
        let reached = crates.get(1..5).unwrap_or_default().join(" ");
        let mut found = vec![format!("duet -> {reached}")];
        found.extend(crates.iter().skip(1).map(|name| format!("{name} ->")));
        found.push("the graph ends here".to_owned());
        found.push("no further target".to_owned());
        found
    }

    /// The section 1.6 constant block of the fixture.
    fn placement_constants() -> Vec<String> {
        let mut found = vec!["// duet-ab".to_owned()];
        for (budget, value, constant) in placement_budgets() {
            found.push(format!("/// {budget} capacity."));
            found.push(format!("pub const {constant}: usize = {value};"));
        }
        let mut index = 0_usize;
        while found.len() < 122 {
            found.push(format!("pub const SYN_{index:03}: usize = 1;"));
            index = index.saturating_add(1);
        }
        found
    }

    /// The section 1.6 budget table of the fixture.
    fn placement_budget_table() -> Vec<String> {
        let mut rows: Vec<String> = placement_budgets()
            .iter()
            .map(|(budget, value, _constant)| {
                md_row(&[budget, &value.to_string(), "a synthetic bound", ""])
            })
            .collect();
        rows.extend(
            (10..149_usize)
                .map(|index| md_row(&[&format!("B{index}"), "-", "a synthetic bound", ""])),
        );
        md_table(&["Id", "Value", "What it bounds", "Used by"], rows)
    }

    /// The section 1.9 probe table of the fixture.
    fn placement_probe_table() -> Vec<String> {
        let mut rows = vec![
            md_row(&[
                "`PG2`",
                "a document that does not open",
                "`PP2`",
                "synthetic",
                "exit 2",
            ]),
            md_row(&[
                "`CG1`",
                "a cast outside the one file",
                "`CP1`",
                "synthetic",
                "exit 1",
            ]),
        ];
        while rows.len() < 65 {
            rows.push(md_row(&["-", "a filler row", "-", "synthetic", "exit 0"]));
        }
        md_table(
            &["Rule", "What it plants", "Probe", "How", "Recorded"],
            rows,
        )
    }

    /// The section 1.9 external-verdict table of the fixture.
    fn placement_external_verdicts() -> Vec<String> {
        let rows = placement_externals()
            .iter()
            .map(|name| {
                md_row(&[
                    &format!("`{name}`"),
                    "`not Copy`",
                    "`Default`",
                    "a synthetic entry",
                ])
            })
            .collect();
        md_table(&["Name", "Verdict", "Traits", "Why"], rows)
    }

    /// The section 1.9 external-path block of the fixture.
    fn placement_external_paths() -> Vec<String> {
        let mut names = placement_externals();
        names.extend(placement_mapped());
        names.extend(placement_framework());
        names.extend(placement_dropped());
        names
            .iter()
            .take(70)
            .map(|one| format!("{one} tpaaa::{}", one.to_lowercase()))
            .collect()
    }

    /// The section 1.9 recorded-size block of the fixture.
    fn placement_recorded_sizes() -> Vec<String> {
        let mut names = placement_externals();
        names.extend(placement_framework());
        names.extend(placement_primitives());
        names
            .iter()
            .take(51)
            .map(|one| format!("{one} 4 4"))
            .collect()
    }

    /// The section 1.9 primitive trait block of the fixture.
    fn placement_primitive_traits() -> Vec<String> {
        let names = placement_primitives();
        let traits = "Copy Default PartialEq Eq Hash Ord PartialOrd Serialize Deserialize";
        [(0_usize, 6_usize), (6, 12), (12, 16)]
            .iter()
            .map(|(from, upto)| {
                let listed = names.get(*from..*upto).unwrap_or_default().join(" ");
                format!("{listed} : {traits}")
            })
            .collect()
    }

    /// The section 5.7 carrier table of the fixture.
    fn placement_carrier_table() -> Vec<String> {
        let rows = placement_carriers()
            .iter()
            .enumerate()
            .map(|(index, name)| {
                md_row(&[
                    &format!("`msg{index:02}`"),
                    &format!("`{name}.w`"),
                    "triple_buffer",
                    &format!("`{name}.r`"),
                    "Audio -> Ui",
                    "the writer overwrites",
                ])
            })
            .collect();
        md_table(
            &[
                "Message",
                "Write end",
                "Carrier",
                "Read end",
                "Threads",
                "Overflow",
            ],
            rows,
        )
    }

    /// The section 13.3 phase table of the fixture.
    fn placement_phase_table() -> Vec<String> {
        let lines = placement_lines();
        let rows = (0..16_usize)
            .map(|phase| {
                let listed = lines
                    .iter()
                    .map(|line| format!("{line}{}", phase.saturating_add(1)))
                    .collect::<Vec<String>>()
                    .join(" ");
                let (chunks, width) = if phase < 4 {
                    (listed, "16")
                } else {
                    ("none".to_owned(), "0")
                };
                md_row(&[
                    &phase.to_string(),
                    "a synthetic phase",
                    "runs",
                    &chunks,
                    width,
                ])
            })
            .collect();
        md_table(&["Phase", "What it does", "State", "Chunks", "Width"], rows)
    }

    /// The section 13.3 phase-pair exemption block of the fixture.
    fn placement_phase_pair_exempt() -> Vec<String> {
        let chunks = placement_chunks();
        (0..24_usize)
            .map(|index| {
                let first = chunks.get(index).cloned().unwrap_or_default();
                let second = chunks
                    .get(index.saturating_add(1))
                    .cloned()
                    .unwrap_or_default();
                format!("{first} {second} the pair {first} and {second} shares one line")
            })
            .collect()
    }

    /// One closure-review table of the fixture, with the stated row count.
    fn placement_closure_rows(count: usize) -> Vec<String> {
        let rows = (0..count)
            .map(|index| {
                md_row(&[
                    &format!("C{}-W1", index.saturating_add(1)),
                    "a synthetic finding",
                    "closed",
                    "9.1",
                ])
            })
            .collect();
        md_table(&["Id", "Finding", "State", "Section"], rows)
    }

    /// The section 9.1 Rust block of the fixture.
    fn placement_declarations() -> Vec<String> {
        let mut found = vec![
            "/// A synthetic function that carries a suppression.".to_owned(),
            "pub fn probe_fn() -> u32 { 1 }".to_owned(),
            String::new(),
        ];
        let mut reachable = placement_roots().get(1..30).unwrap_or_default().to_vec();
        reachable.extend(placement_leaves());
        let fields = reachable
            .iter()
            .enumerate()
            .map(|(index, name)| format!("f{index}: {name}"))
            .collect::<Vec<String>>()
            .join(", ");
        found.push("/// The audio root. **Audio-owned**".to_owned());
        found.push(format!("pub struct EngineProcess {{ {fields} }}"));
        found.push(String::new());
        for name in placement_roots().iter().skip(1) {
            found.push("/// A synthetic root. **Audio-owned**".to_owned());
            found.push(format!("pub struct {name} {{ v: u32 }}"));
            found.push(String::new());
        }
        for name in placement_leaves() {
            found.push("/// A synthetic leaf.".to_owned());
            found.push(format!("pub struct {name} {{ v: u32 }}"));
            found.push(String::new());
        }
        for name in placement_carriers() {
            found.push("/// A synthetic carrier holder.".to_owned());
            found.push(format!(
                "pub struct {name} {{ w: triple_buffer::Input<u32>, r: triple_buffer::Output<u32> }}"
            ));
            found.push(String::new());
        }
        found
    }

    /// The registered blocks of sections 1.2, 1.3, 1.5, and 1.6 of the fixture.
    fn placement_blocks_plan() -> Vec<PlacementBlock> {
        let third = placement_third_party();
        let mapped = placement_mapped();
        let name_map = (0..23_usize)
            .map(|index| {
                let name = mapped.get(index).cloned().unwrap_or_default();
                let owner = third.get(index % third.len()).cloned().unwrap_or_default();
                format!("{name} {owner}")
            })
            .collect();
        let limits = placement_budgets()
            .iter()
            .take(5)
            .map(|(_budget, _value, constant)| format!("{constant} duet-ab duet-ab"))
            .collect();
        vec![
            pblock(
                "crate-table",
                "The crate dependency table",
                "table",
                "not-declared",
                placement_crate_table(),
            ),
            pblock(
                "carrier-table",
                "Every cross-thread carrier, and the two ends it needs",
                "table",
                "not-declared",
                placement_carrier_table(),
            ),
            pblock(
                "name-map",
                "The third-party name map",
                "text",
                "not-declared",
                name_map,
            ),
            pblock(
                "edge-list",
                "The internal edge list",
                "text",
                "not-declared",
                placement_edge_list(),
            ),
            pblock(
                "framework-types",
                "Framework types",
                "text",
                "external-name",
                placement_framework(),
            ),
            pblock(
                "ownership-table",
                "The type ownership table",
                "table",
                "not-declared",
                placement_ownership_table(),
            ),
            pblock(
                "drop-list",
                "The candidate drop list",
                "text",
                "drop-name",
                placement_dropped(),
            ),
            pblock(
                "constants",
                "Every workspace constant",
                "rust",
                "not-declared",
                placement_constants(),
            ),
            pblock(
                "shared-limits",
                "Every shared limit and its enforcers",
                "text",
                "not-declared",
                limits,
            ),
            pblock(
                "budget-table",
                "Every budget and bound",
                "table",
                "not-declared",
                placement_budget_table(),
            ),
        ]
    }

    /// The registered blocks of section 1.9 of the fixture.
    fn placement_blocks_registers() -> Vec<PlacementBlock> {
        let mut found = placement_blocks_tables();
        found.extend(placement_blocks_names());
        found
    }

    /// The register blocks of section 1.9 the guard reads as tables and lists.
    fn placement_blocks_tables() -> Vec<PlacementBlock> {
        vec![
            pblock(
                "probe-table",
                "Every rule, its probe, and the recorded result",
                "table",
                "not-declared",
                placement_probe_table(),
            ),
            pblock(
                "external-verdicts",
                "Every external type, and its verdict",
                "table",
                "not-declared",
                placement_external_verdicts(),
            ),
            pblock(
                "external-paths",
                "Where every external name comes from",
                "text",
                "not-declared",
                placement_external_paths(),
            ),
            pblock(
                "pins",
                "The external crate pins the roster compile uses",
                "text",
                "not-declared",
                placement_third_party()
                    .iter()
                    .map(|one| format!("{one} 1.0"))
                    .collect(),
            ),
            pblock(
                "recorded-sizes",
                "Every size the guard records",
                "text",
                "not-declared",
                placement_recorded_sizes(),
            ),
            pblock(
                "substitutions",
                "Every substitution the roster compile applies",
                "text",
                "sub-id",
                (1..9_usize)
                    .map(|index| format!("S{index} a synthetic substitution"))
                    .collect(),
            ),
            pblock(
                "drop-impls",
                "Every declaration with a hand-written Drop impl",
                "text",
                "declared-name",
                placement_leaves().get(..3).unwrap_or_default().to_vec(),
            ),
            pblock(
                "impl-sites",
                "Every impl block the roster compiles",
                "text",
                "not-declared",
                (0..65_usize)
                    .map(|index| format!("site{index:02} duet"))
                    .collect(),
            ),
        ]
    }

    /// The name-set blocks of section 1.9 of the fixture.
    fn placement_blocks_names() -> Vec<PlacementBlock> {
        let heap = [
            "Box", "Vec", "String", "Arc", "Rc", "BTreeMap", "BTreeSet", "HashMap", "HashSet",
            "VecDeque", "Cow", "PathBuf",
        ];
        let grow = [
            "Vec", "String", "HashMap", "HashSet", "BTreeMap", "BTreeSet", "VecDeque", "Cow",
            "PathBuf", "SmallVec",
        ];
        let lock = [
            "Mutex",
            "RwLock",
            "ReentrantLock",
            "Condvar",
            "Barrier",
            "OnceLock",
            "LazyLock",
            "MutexGuard",
            "RwLockReadGuard",
            "RwLockWriteGuard",
        ];
        let owned =
            |names: &[&str]| -> Vec<String> { names.iter().map(|one| (*one).to_owned()).collect() };
        vec![
            pblock(
                "heap-names",
                "Every heap-owning name the audio rules refuse",
                "text",
                "not-declared",
                owned(&heap),
            ),
            pblock(
                "grow-names",
                "Every growable name the audio rules refuse",
                "text",
                "not-declared",
                owned(&grow),
            ),
            pblock(
                "lock-names",
                "Every lock name the audio rules refuse",
                "text",
                "not-declared",
                owned(&lock),
            ),
            pblock(
                "primitive-traits",
                "Every primitive trait set the guard uses",
                "text",
                "not-declared",
                placement_primitive_traits(),
            ),
            pblock(
                "primitive-sizes",
                "Every primitive size the guard uses",
                "text",
                "not-declared",
                placement_primitives()
                    .iter()
                    .map(|one| format!("{one} 4 4"))
                    .collect(),
            ),
            pblock(
                "justified-unknowns",
                "The justified unknowns",
                "table",
                "not-declared",
                md_filler(2, 4, "unknown"),
            ),
            pblock(
                "block-members",
                "Every registered block and its membership rule",
                "text",
                "block-id",
                Vec::new(),
            ),
            pblock(
                "rule-blocks",
                "Which rule reads which block",
                "table",
                "not-declared",
                Vec::new(),
            ),
        ]
    }

    /// The registered blocks of sections 3.5, 5.7, and Appendix B.1 of the fixture.
    fn placement_blocks_audio() -> Vec<PlacementBlock> {
        let convert = {
            let mut rows = vec![md_row(&[
                "`probe_fn`",
                "a synthetic reason",
                "when the port lands",
            ])];
            rows.extend((0..6_usize).map(|index| {
                md_row(&[
                    &format!("site {index}"),
                    "a synthetic reason",
                    "when the port lands",
                ])
            }));
            md_table(&["Site", "Why", "Removal"], rows)
        };
        vec![
            pblock(
                "vr1-table",
                "Every VR1 derive and its use",
                "table",
                "not-declared",
                md_filler(27, 3, "vr1"),
            ),
            pblock(
                "audio-owned",
                "Every audio-owned declaration",
                "text",
                "declared-name",
                placement_roots(),
            ),
            pblock(
                "audio-exempt",
                "Every audio-owned field the rule exempts",
                "text",
                "not-declared",
                vec!["Aud01.v Ext0 a synthetic exemption".to_owned()],
            ),
            pblock(
                "audio-reachable-leaf",
                "Every reachable leaf the closure rule allows",
                "text",
                "declared-name",
                placement_leaves(),
            ),
            pblock(
                "audio-asserted",
                "Every audio-owned root the closure does not reach",
                "text",
                "declared-name",
                placement_roots().get(30..).unwrap_or_default().to_vec(),
            ),
            pblock(
                "snapshot-table",
                "High-rate traffic: latest value, lock free, no event",
                "table",
                "not-declared",
                md_filler(7, 3, "snap"),
            ),
            pblock(
                "b1-convert",
                "Conversion suppressions",
                "table",
                "not-declared",
                convert,
            ),
            pblock(
                "b1-complexity",
                "Complexity suppressions",
                "table",
                "not-declared",
                md_filler(6, 3, "cx"),
            ),
            pblock(
                "b1-copy",
                "Expectations for missing_copy_implementations",
                "table",
                "not-declared",
                md_filler(4, 3, "cp"),
            ),
            pblock(
                "b1-variant",
                "Expectations for variant_size_differences",
                "table",
                "not-declared",
                md_filler(3, 3, "vs"),
            ),
        ]
    }

    /// The registered blocks of sections 13 and 14 and Appendix C of the fixture.
    fn placement_blocks_plan_graph() -> Vec<PlacementBlock> {
        let reviews: [(&'static str, &'static str, usize); 9] = [
            (
                "closure-r16",
                "The revision-16 review, over the frozen document",
                46,
            ),
            (
                "closure-r17",
                "The revision-17 review, over the frozen document",
                26,
            ),
            (
                "closure-r18",
                "The revision-18 review, over the frozen document",
                20,
            ),
            (
                "closure-r19",
                "The revision-19 review, over the frozen document",
                45,
            ),
            (
                "closure-r20",
                "The revision-20 review, over the frozen document",
                23,
            ),
            (
                "closure-r21-inner",
                "The revision-21 inner review, over the frozen document",
                13,
            ),
            (
                "closure-r21",
                "The revision-21 external review, over the frozen document",
                45,
            ),
            (
                "closure-r22-inner",
                "The revision-22 inner review, over the frozen document",
                35,
            ),
            (
                "closure-r23-inner",
                "The revision-23 inner review, over the frozen document",
                31,
            ),
        ];
        let mut found = vec![
            pblock(
                "phase-table",
                "The phase table",
                "table",
                "not-declared",
                placement_phase_table(),
            ),
            pblock(
                "selected-tests",
                "Every test this section selects by name",
                "table",
                "not-declared",
                md_filler(8, 6, "sel"),
            ),
            pblock(
                "line-map",
                "Every chunk line and the crate it owns",
                "text",
                "not-declared",
                placement_lines()
                    .iter()
                    .map(|one| format!("{one} duet"))
                    .collect(),
            ),
            pblock(
                "context-map",
                "The bounded context map",
                "text",
                "crate-name",
                placement_context_map(),
            ),
            pblock(
                "phase-pair-exempt",
                "Every same-phase crate edge that does not bind",
                "text",
                "not-declared",
                placement_phase_pair_exempt(),
            ),
            pblock(
                "gate-defects",
                "Six planted gate defects, and the rule that catches each one",
                "table",
                "not-declared",
                md_filler(6, 3, "gate"),
            ),
            pblock(
                "fault-messages",
                "Every engine fault and the line the user reads",
                "table",
                "not-declared",
                md_filler(16, 3, "fault"),
            ),
        ];
        found.extend(reviews.iter().map(|(id, heading, count)| {
            pblock(
                id,
                heading,
                "table",
                "not-declared",
                placement_closure_rows(*count),
            )
        }));
        found
    }

    /// The row floor the guard register states for the `rule-blocks` block.
    const RULE_BLOCK_FLOOR: usize = 34;

    /// Every registered block of the fixture, with its two self-describing rows.
    fn placement_register() -> Vec<PlacementBlock> {
        let mut found = placement_blocks_plan();
        found.extend(placement_blocks_registers());
        found.extend(placement_blocks_audio());
        found.extend(placement_blocks_plan_graph());
        let members: Vec<String> = found
            .iter()
            .map(|entry| format!("{} {}", entry.id, entry.member))
            .collect();
        let ids: Vec<String> = found.iter().map(|entry| entry.id.to_owned()).collect();
        let groups: Vec<String> = ids.chunks(2).map(md_spans).collect();
        let rows: Vec<String> = (0..RULE_BLOCK_FLOOR)
            .map(|index| {
                let cell = groups
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| "`crate-table`".to_owned());
                md_row(&[&format!("rule {index}"), &cell])
            })
            .collect();
        for entry in &mut found {
            if entry.id == "block-members" {
                entry.body = members.clone();
                entry.minimum = members.len();
            }
            if entry.id == "rule-blocks" {
                entry.body = md_table(&["Rule", "Blocks"], rows.clone());
                entry.minimum = RULE_BLOCK_FLOOR;
            }
        }
        found
    }

    /// One registered block of the fixture, as the document writes it.
    fn placement_render(blocks: &[PlacementBlock], id: &str) -> String {
        let entry = blocks
            .iter()
            .find(|entry| entry.id == id)
            .unwrap_or_else(|| panic!("the fixture registers `{id}`"));
        let mut out = vec![
            format!("#### {}", entry.heading),
            String::new(),
            format!("<!-- GUARD BLOCK id={id} rows>={} -->", entry.minimum),
        ];
        if entry.kind == "table" {
            out.extend(entry.body.clone());
        } else {
            out.push(format!("```{}", entry.kind));
            out.extend(entry.body.clone());
            out.push("```".to_owned());
        }
        out.push(String::new());
        format!("{}\n", out.join("\n"))
    }

    /// The section 1.7 rule index of the fixture.
    fn placement_index_table() -> String {
        let rows = vec![
            md_row(&["PG2", "opens the document", "1.5"]),
            md_row(&["PP2", "probes the open", "1.9"]),
            md_row(&["CG1", "refuses one cast", "1.5"]),
            md_row(&["CP1", "probes the cast", "1.9"]),
        ];
        format!(
            "{}\n",
            md_table(&["Id", "Rule, in three words", "Stated in"], rows).join("\n")
        )
    }

    /// The section 5.7 thread table of the fixture.
    fn placement_thread_table() -> String {
        let rows = vec![
            md_row(&["Audio", "the engine", "the graph", "allocate"]),
            md_row(&["Ui", "the shell", "the views", "block"]),
        ];
        format!(
            "{}\n",
            md_table(&["Thread", "Owner", "Owns", "Never does"], rows).join("\n")
        )
    }

    /// The section 13.1 chunk table of the fixture.
    fn placement_chunk_table() -> String {
        let pins = placement_third_party()
            .iter()
            .map(|one| format!("`{one}`"))
            .collect::<Vec<String>>()
            .join(" ");
        let manifest = format!("`[workspace.dependencies]` only ({pins})");
        let rows = placement_chunks()
            .iter()
            .map(|chunk| {
                let phase = chunk
                    .chars()
                    .last()
                    .and_then(|mark| mark.to_digit(10))
                    .unwrap_or(1);
                let scope = if chunk == "M1" {
                    manifest.clone()
                } else {
                    PLACEMENT_SCOPE.to_owned()
                };
                md_row(&[
                    chunk,
                    &phase.saturating_sub(1).to_string(),
                    "a synthetic chunk",
                    &scope,
                    "a synthetic gate",
                ])
            })
            .collect();
        format!(
            "{}\n",
            md_table(&["Chunk", "Phase", "What", "Writes", "Gate"], rows).join("\n")
        )
    }

    /// The two appendix pin tables of the fixture.
    fn placement_pin_tables() -> (String, String) {
        let third = placement_third_party();
        let short = third
            .iter()
            .map(|one| md_row(&[&format!("`{one}`"), "a synthetic pin", "M1"]))
            .collect();
        let long = third
            .iter()
            .map(|one| {
                md_row(&[
                    &format!("`{one}`"),
                    "a synthetic pin",
                    "the workspace",
                    "M1",
                ])
            })
            .collect();
        (
            format!(
                "{}\n",
                md_table(&["Pin", "What", "Owner"], short).join("\n")
            ),
            format!(
                "{}\n",
                md_table(&["Pin", "What", "Where", "Owner"], long).join("\n")
            ),
        )
    }

    /// Sections 1.2 to 1.9 of the fixture document.
    fn placement_part_one(blocks: &[PlacementBlock]) -> Vec<String> {
        let render = |id: &str| placement_render(blocks, id);
        let mut out = vec![
            "# A synthetic architecture document\n".to_owned(),
            "## 1. The plan\n".to_owned(),
            "### 1.2 The crates\n".to_owned(),
            render("crate-table"),
            render("name-map"),
            "### 1.3 The graph\n".to_owned(),
            render("edge-list"),
            render("framework-types"),
            "### 1.5 The placement table\n".to_owned(),
            render("ownership-table"),
            render("drop-list"),
            "### 1.6 The budgets\n".to_owned(),
            render("budget-table"),
            render("shared-limits"),
            "### 1.7 The rule index\n".to_owned(),
            placement_index_table(),
            render("constants"),
            "### 1.9 The registers\n".to_owned(),
        ];
        for id in [
            "probe-table",
            "external-verdicts",
            "external-paths",
            "pins",
            "recorded-sizes",
            "substitutions",
            "drop-impls",
            "impl-sites",
            "heap-names",
            "grow-names",
            "lock-names",
            "primitive-traits",
            "primitive-sizes",
            "justified-unknowns",
            "block-members",
            "rule-blocks",
        ] {
            out.push(render(id));
        }
        out
    }

    /// Sections 3.5, 5.7, 9.1, and 2.3 of the fixture document.
    fn placement_part_two(blocks: &[PlacementBlock]) -> Vec<String> {
        let render = |id: &str| placement_render(blocks, id);
        vec![
            "### 3.5 The value rules\n".to_owned(),
            render("vr1-table"),
            "### 5.7 The threads\n".to_owned(),
            placement_thread_table(),
            render("audio-owned"),
            render("audio-exempt"),
            render("audio-reachable-leaf"),
            render("audio-asserted"),
            render("carrier-table"),
            render("snapshot-table"),
            "### 9.1 The declarations\n".to_owned(),
            "```rust".to_owned(),
            placement_declarations().join("\n"),
            "```\n".to_owned(),
            "## 2. The crates in detail\n".to_owned(),
            "### 2.3 The conversion crate\n".to_owned(),
            "one functions carry a suppression: `probe_fn`. The rest carry none.\n".to_owned(),
        ]
    }

    /// Sections 13 and 14 of the fixture document.
    fn placement_part_three(blocks: &[PlacementBlock]) -> Vec<String> {
        let render = |id: &str| placement_render(blocks, id);
        let zeros = vec!["0"; 16].join(", ");
        vec![
            "## 13. The plan graph\n".to_owned(),
            "### 13.1 The chunk table\n".to_owned(),
            placement_chunk_table(),
            "### 13.3 The phases\n".to_owned(),
            render("phase-table"),
            render("line-map"),
            render("context-map"),
            render("phase-pair-exempt"),
            format!("**Per-phase `Cargo.lock` writer counts, phases 0 to 15: {zeros}**\n"),
            "**The plan is longer and narrower than revision 5's.** sixteen phases replace \
four, and the widest phase falls from twenty to sixteen. The alternative rule is stated \
elsewhere.\n"
                .to_owned(),
            render("gate-defects"),
            render("fault-messages"),
            "## 14. The measurable completion outcome for version one\n".to_owned(),
            "The run is measured here.\n".to_owned(),
            render("selected-tests"),
        ]
    }

    /// The appendices of the fixture document.
    fn placement_part_four(blocks: &[PlacementBlock]) -> Vec<String> {
        let render = |id: &str| placement_render(blocks, id);
        let (short, long) = placement_pin_tables();
        let mut out = vec![
            "## Appendix A. The glossary\n".to_owned(),
            "A synthetic glossary.\n".to_owned(),
            "## Appendix B. The registers\n".to_owned(),
            "### B.1 The suppressions\n".to_owned(),
            "Appendix B.1 holds seven suppressions in `duet-time::convert`, six complexity \
suppressions, four `missing_copy_implementations` expectations, and three \
`variant_size_differences` expectations.\n"
                .to_owned(),
            render("b1-convert"),
            render("b1-complexity"),
            render("b1-copy"),
            render("b1-variant"),
            "### B.3 The pins\n".to_owned(),
            short,
            "### B.4 The features\n".to_owned(),
            "A synthetic list.\n".to_owned(),
            "### B.5 The second pin table\n".to_owned(),
            long,
            "#### Every timeout the plan states\n".to_owned(),
            "A synthetic list.\n".to_owned(),
            "## Appendix C. The closures\n".to_owned(),
        ];
        let reviews = [
            "r16",
            "r17",
            "r18",
            "r19",
            "r20",
            "r21-inner",
            "r21",
            "r22-inner",
            "r23-inner",
        ];
        for (index, label) in reviews.iter().enumerate() {
            out.push(format!(
                "### C.{} The review {label}\n",
                index.saturating_add(1)
            ));
            out.push(render(&format!("closure-{label}")));
        }
        out
    }

    /// The whole synthetic architecture document the placement guard reads clean.
    fn placement_document() -> String {
        let blocks = placement_register();
        let mut parts = placement_part_one(&blocks);
        parts.extend(placement_part_two(&blocks));
        parts.extend(placement_part_three(&blocks));
        parts.extend(placement_part_four(&blocks));
        parts.join("\n")
    }

    /// The four plan-tool files `PG29` reads for their own rule ids.
    fn placement_tools() -> Vec<(&'static str, String)> {
        vec![
            ("placement_check.py", "# PG2 rule\n".to_owned()),
            ("conversion_check.py", "# CG1 rule\n".to_owned()),
            ("roster_compile.sh", "# PG2 rule\n".to_owned()),
            ("closure_check.py", "# PG2 rule\n".to_owned()),
        ]
    }

    /// Run the placement guard over one synthetic document, and clean up.
    ///
    /// The document and the sibling `tools` directory the `PG29` rule reads are
    /// both written into a scratch directory this probe owns, so no probe reads
    /// this repository and no probe reads the plan.
    fn placement_run(
        label: &str,
        text: &str,
        tools: &[(&'static str, String)],
        write: bool,
    ) -> (i32, String) {
        let root = scratch(label);
        let document = root.join("architecture.md");
        for (name, body) in tools {
            write_bytes(&root.join("tools").join(name), body.as_bytes());
        }
        if write {
            write_bytes(&document, text.as_bytes());
        } else {
            fs::create_dir_all(&root).expect("scratch root");
        }
        let report = xtask(&root, &["check-placement", &document.display().to_string()]);
        clean(&root);
        report
    }

    /// Run the placement guard over one synthetic document with the stated tools.
    fn placement(label: &str, text: &str) -> (i32, String) {
        placement_run(label, text, &placement_tools(), true)
    }

    /// The fixture document with one exact span replaced once.
    fn plant(text: &str, old: &str, new: &str) -> String {
        assert!(
            text.contains(old),
            "the synthetic document holds the span `{old}` a probe plants into"
        );
        text.replacen(old, new, 1)
    }

    /// The fixture with one declaration added and its names placed by section 1.5.
    fn plant_declaration(text: &str, declaration: &str, names: &[&str]) -> String {
        let placed: Vec<String> = names.iter().map(|one| format!("`{one}`")).collect();
        let text = plant(
            text,
            "/// A synthetic carrier holder.",
            &format!("{declaration}\n/// A synthetic carrier holder."),
        );
        plant(&text, "`Car00`", &format!("{}, `Car00`", placed.join(", ")))
    }

    /// The fixture with three container names decided and dropped.
    ///
    /// A container name a field states is a used name, so the drop list has to
    /// carry it, and the section 1.9 table has to decide it, or `PG4` and `PG18`
    /// answer before the audio rule the probe is aimed at.
    fn plant_containers(text: &str) -> String {
        let text = plant(
            text,
            "| `Ext31` | `not Copy` | `Default` | a synthetic entry |",
            "| `Box` | `not Copy` | `Default` | a synthetic entry |",
        );
        let text = plant(
            &text,
            "| `Ext32` | `not Copy` | `Default` | a synthetic entry |",
            "| `Vec` | `not Copy` | `Default` | a synthetic entry |",
        );
        let text = plant(
            &text,
            "| `Ext33` | `not Copy` | `Default` | a synthetic entry |",
            "| `Mutex` | `not Copy` | `Default` | a synthetic entry |",
        );
        plant(&text, "Debug\nDefault\nHash", "Box\nVec\nMutex")
    }

    /// The fixture with one external name left undecided and dropped.
    fn plant_undecided(text: &str) -> String {
        let text = plant(
            text,
            "| `Ext33` | `not Copy` |",
            "| `Ext33` | `undecided` |",
        );
        plant(&text, "Debug\n", "Ext33\n")
    }

    #[test]
    fn placement_baseline_reads_the_synthetic_document_clean() {
        let (code, report) = placement("pg-base", &placement_document());
        assert_eq!(code, 0, "the synthetic document holds no breach: {report}");
        assert!(
            report.contains("BLOCKS:          52     BLOCK BAD: 0"),
            "every registered block reads: {report}"
        );
        assert!(
            report.contains("UNPLACED:        0     DUPLICATED:   0"),
            "the placement rules find nothing: {report}"
        );
        assert!(
            report.contains("MEMBER ROWS: 1381     MEMBER BAD: 0"),
            "every row of every block names a referent: {report}"
        );
        assert!(
            report.contains("CARRIERS:        25     CARRIER FIELDS: 50     CARRIER BAD: 0"),
            "every carrier row has its two declared ends: {report}"
        );
    }

    #[test]
    fn placement_pg1_no_argument_exits_two() {
        let root = scratch("pg-noargument");
        let (code, _report) = xtask(&root, &["check-placement"]);
        clean(&root);
        assert_eq!(code, 2, "a run with no document argument is fail-closed");
    }

    #[test]
    fn placement_pg2_document_that_does_not_open_fails_closed() {
        let (code, report) = placement_run("pg2", "", &placement_tools(), false);
        assert_eq!(
            code, 2,
            "a document that does not open is fail-closed: {report}"
        );
        assert!(
            report.contains("FAIL: cannot open")
                && report.contains("architecture.md; the guard is fail-closed."),
            "the guard names the document it cannot open: {report}"
        );
    }

    /// The fixture document with the body of the section 9.1 fence removed.
    ///
    /// The parser reads a declaration and a used name from that one Rust fence,
    /// so a document with an empty fence states neither, and the candidate set
    /// the placement rules decide over is empty. Every guard block, marker,
    /// heading and list fence stays, so the run reaches the candidate count
    /// instead of one of the earlier fail-closed rules.
    fn without_declarations(text: &str) -> String {
        const DECLARATION_HEADING: &str = "### 9.1 The declarations";
        let mut out = String::new();
        let mut at_section = false;
        let mut inside = false;
        for line in text.lines() {
            if inside && line != "```" {
                continue;
            }
            if inside {
                inside = false;
                at_section = false;
            } else if at_section && line == "```rust" {
                inside = true;
            } else if line == DECLARATION_HEADING {
                at_section = true;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    #[test]
    fn placement_vocabulary_empty_candidate_set_fails_closed() {
        let (code, report) = placement_run(
            "pg-nocandidate",
            &without_declarations(&placement_document()),
            &placement_tools(),
            true,
        );
        assert_eq!(
            code, 2,
            "a document the parser reads to zero candidates gives exit 2: {report}"
        );
        assert!(
            report.contains("FAIL: the candidate set is empty, so the parse is broken."),
            "the guard states that the parse is broken: {report}"
        );
    }

    #[test]
    fn placement_pg3_comment_where_a_field_belongs_is_a_finding() {
        let text = plant(
            &placement_document(),
            "pub struct Leaf00 { v: u32 }",
            "pub struct Leaf00 { /* private */ }",
        );
        let (code, report) = placement("pg3", &text);
        assert_eq!(
            code, 1,
            "a comment where a field belongs is a finding: {report}"
        );
        assert!(
            report.contains("PLACEHOLDER: duet::Leaf00 has a comment where a field belongs"),
            "the guard names the declaration whose body states no field: {report}"
        );
    }

    #[test]
    fn placement_pg4_declared_type_the_table_omits_is_a_finding() {
        let text = plant(&placement_document(), "`Leaf45`, ", "");
        let (code, report) = placement("pg4", &text);
        assert_eq!(
            code, 1,
            "a declared type section 1.5 omits is a finding: {report}"
        );
        assert!(
            report.contains("UNPLACED:   Leaf45"),
            "the guard names the declared type no row places: {report}"
        );
    }

    #[test]
    fn placement_pg4b_placed_name_no_block_declares_is_a_finding() {
        let text = plant(&placement_document(), "`Car00`", "`Ghost`, `Car00`");
        let (code, report) = placement("pg4b", &text);
        assert_eq!(
            code, 1,
            "a placed name with no declaration is a finding: {report}"
        );
        assert!(
            report.contains("UNDECLARED: Ghost is placed by 1.5 and no Rust block declares it"),
            "the guard names the placed name no Rust block declares: {report}"
        );
    }

    #[test]
    fn placement_pg5_type_two_blocks_declare_is_a_finding() {
        let text = plant(
            &placement_document(),
            "/// A synthetic carrier holder.",
            "/// A second declaration.\npub struct Leaf45 { v: u32 }\n\n\
/// A synthetic carrier holder.",
        );
        let (code, report) = placement("pg5", &text);
        assert_eq!(code, 1, "one type declared twice is a finding: {report}");
        assert!(
            report.contains("DUPLICATED: Leaf45"),
            "the guard names the type two Rust blocks declare: {report}"
        );
    }

    #[test]
    fn placement_pg6_framework_name_claimed_by_another_crate_is_a_finding() {
        let text = plant(&placement_document(), "Fw6\n```", "Car00\n```");
        let (code, report) = placement("pg6", &text);
        assert_eq!(
            code, 1,
            "a framework name a non-application crate claims is a finding: {report}"
        );
        assert!(
            report.contains("MISCLAIMED: Car00 claimed by duet"),
            "the guard names the framework name and the crate that claims it: {report}"
        );
    }

    #[test]
    fn placement_pg7_framework_type_in_a_non_application_declaration_is_a_finding() {
        let text = plant(&placement_document(), "Fw6\n```", "Ext0\n```");
        let text = plant(
            &text,
            "pub struct Leaf45 { v: u32 }",
            "pub struct Leaf45 { v: u32, g: Ext0 }",
        );
        let (code, report) = placement("pg7", &text);
        assert_eq!(
            code, 1,
            "a framework type inside a non-application declaration is a finding: {report}"
        );
        assert!(
            report.contains("FRAMEWORK:  duet::Leaf45 holds Ext0"),
            "the guard names the declaration and the framework type it holds: {report}"
        );
    }

    #[test]
    fn placement_pg8_an_enum_arm_name_is_masked_inside_its_own_enum_only() {
        let masked = plant_declaration(
            &placement_document(),
            "/// A synthetic enum.\npub enum Kind { Ghost, Other }\n",
            &["Kind"],
        );
        let (clean_code, clean_report) = placement("pg8-arm", &masked);
        assert_eq!(
            clean_code, 0,
            "an arm name is no type of its own enum: {clean_report}"
        );
        let shadowed = plant_declaration(
            &placement_document(),
            "/// A synthetic enum.\npub enum Kind { Ghosty, Other }\n\n\
/// A synthetic struct named like an arm.\npub struct Ghosty { v: u32 }\n",
            &["Kind"],
        );
        let (code, report) = placement("pg8-struct", &shadowed);
        assert_eq!(
            code, 1,
            "a struct named like an arm is still a candidate: {report}"
        );
        assert!(
            report.contains("UNPLACED:   Ghosty"),
            "the mask is local to the enum that declares the arm: {report}"
        );
    }

    #[test]
    fn placement_pg9_all_copy_type_with_no_copy_derive_is_a_finding() {
        let text = plant(
            &placement_document(),
            "| `Ext0` | `not Copy` |",
            "| `Ext0` | `Copy` |",
        );
        let text = plant(&text, "Debug\n", "Ext0\n");
        let text = plant_declaration(
            &text,
            "/// An all-Copy body with no Copy derive.\npub struct Cap { a: Ext0 }\n",
            &["Cap"],
        );
        let (code, report) = placement("pg9", &text);
        assert_eq!(
            code, 1,
            "an all-Copy type that derives no Copy is a finding: {report}"
        );
        assert!(
            report.contains("COPY:       duet::Cap needs Copy or an #[expect]"),
            "the guard names the type that needs the derive or the expectation: {report}"
        );
    }

    #[test]
    fn placement_pg10_copy_derive_over_a_field_that_is_not_copy_is_a_finding() {
        let text = plant_declaration(
            &placement_document(),
            "/// A Copy derive over a field that is not Copy.\n#[derive(Copy)]\n\
pub struct Cap { a: Leaf01 }\n",
            &["Cap"],
        );
        let (code, report) = placement("pg10", &text);
        assert_eq!(
            code, 1,
            "a Copy derive over a field that is not Copy is a finding: {report}"
        );
        assert!(
            report.contains("NOT COPY:   duet::Cap derives Copy over Leaf01"),
            "the guard names the derive and the field that refutes it: {report}"
        );
    }

    #[test]
    fn placement_pg10b_copy_derive_over_an_undecided_field_is_a_finding() {
        let text = plant_declaration(
            &plant_undecided(&placement_document()),
            "/// A Copy derive over an undecided field.\n#[derive(Copy)]\n\
pub struct Cap { a: Ext33 }\n",
            &["Cap"],
        );
        let (code, report) = placement("pg10b", &text);
        assert_eq!(
            code, 1,
            "a Copy derive over an undecided field is a finding: {report}"
        );
        assert!(
            report.contains("NOT DECIDED:duet::Cap derives Copy over Ext33"),
            "the guard names the field whose Copy-ness no row decides: {report}"
        );
    }

    #[test]
    fn placement_pg11_prose_edge_the_list_does_not_carry_is_a_finding() {
        let text = plant(
            &placement_document(),
            "A synthetic glossary.",
            "The crate `duet-ae` uses `duet-af` here.",
        );
        let (code, report) = placement("pg11", &text);
        assert_eq!(
            code, 1,
            "a prose edge the 1.3 list omits is a finding: {report}"
        );
        assert!(
            report.contains("EDGE MISS:  duet-ae -> duet-af"),
            "the guard names the two crates of the claim: {report}"
        );
    }

    #[test]
    fn placement_pg12_dependency_row_that_omits_a_used_crate_is_a_finding() {
        let text = plant(&placement_document(), " `tpam` |", " |");
        let text = plant(
            &text,
            "| `duet-aa` | member | a synthetic member | none |",
            "| `duet-aa` | member | a synthetic member | `tpam` |",
        );
        let text = plant(
            &text,
            "pub struct Leaf45 { v: u32 }",
            "pub struct Leaf45 { v: u32, q: tpam::Qm12 }",
        );
        let (code, report) = placement("pg12", &text);
        assert_eq!(
            code, 1,
            "a 1.2 row that omits a used crate is a finding: {report}"
        );
        assert!(
            report.contains("DEP MISS:   duet uses tpam through Leaf45"),
            "the guard names the crate, the dependency, and the declaration: {report}"
        );
    }

    #[test]
    fn placement_pg13_published_snapshot_that_is_not_declared_is_a_finding() {
        let text = plant(
            &placement_document(),
            "| snap0 | text | text |",
            "| Audio | text | triple_buffer::Output<Ghost> |",
        );
        let (code, report) = placement("pg13", &text);
        assert_eq!(
            code, 1,
            "a published type no block declares is a finding: {report}"
        );
        assert!(
            report.contains("SNAPSHOT:   Ghost: no Rust block declares it"),
            "the guard names the published type it cannot find: {report}"
        );
    }

    #[test]
    fn placement_pg14_undecidable_field_is_counted_as_unknown() {
        let (base_code, base_report) = placement("pg14-base", &placement_document());
        assert_eq!(
            base_code, 0,
            "the baseline decides every field: {base_report}"
        );
        assert!(
            base_report.contains("UNKNOWN: 0"),
            "the baseline counts no undecidable field: {base_report}"
        );
        let text = plant_declaration(
            &plant_undecided(&placement_document()),
            "/// An undecidable field.\npub struct Cap { a: Ext33 }\n",
            &["Cap"],
        );
        let text = plant(
            &text,
            "| unknown0 | text | text | text |",
            "| unknown0 | text | `Ext33` | text |",
        );
        let (code, report) = placement("pg14", &text);
        assert_eq!(
            code, 0,
            "a justified unknown is a count and not a breach: {report}"
        );
        assert!(
            report.contains("UNKNOWN: 1     JUSTIFIED: 1     UNJUSTIFIED: 0"),
            "the guard counts the undecidable field of any declaration: {report}"
        );
    }

    #[test]
    fn placement_pg15_declared_in_cell_that_omits_a_section_is_a_finding() {
        let text = plant(
            &placement_document(),
            ", `Car24` | 9.1 |",
            ", `Car24` | 9.2 |",
        );
        let (code, report) = placement("pg15", &text);
        assert_eq!(
            code, 1,
            "a `Declared in` cell that omits a section is a finding: {report}"
        );
        assert!(
            report.contains("REGISTER:   duet declares Aud01 in 9.1, which its cell omits"),
            "the guard names the crate, the type, and the section: {report}"
        );
    }

    #[test]
    fn placement_pg16_selected_test_no_chunk_writes_is_a_finding() {
        let text = plant(
            &placement_document(),
            "The run is measured here.",
            "The run selects test(ghost_test) here.",
        );
        let (code, report) = placement("pg16", &text);
        assert_eq!(
            code, 1,
            "a selected test no chunk writes is a finding: {report}"
        );
        assert!(
            report.contains("TEST MISS:  ghost_test: no row in the selected-test table"),
            "the guard names the test section 14 selects: {report}"
        );
    }

    #[test]
    fn placement_pg17_undecidable_field_with_no_justified_row_is_a_finding() {
        let text = plant_declaration(
            &plant_undecided(&placement_document()),
            "/// An undecidable field.\npub struct Cap { a: Ext33 }\n",
            &["Cap"],
        );
        let (code, report) = placement("pg17", &text);
        assert_eq!(code, 1, "an unjustified unknown is a finding: {report}");
        assert!(
            report.contains("UNJUSTIFIED:Cap.Ext33 is in no section 1.9 row"),
            "the guard names the declaration and the undecidable field: {report}"
        );
    }

    #[test]
    fn placement_pg18_external_token_the_table_omits_is_a_finding() {
        let text = plant(
            &placement_document(),
            "pub struct Leaf45 { v: u32 }",
            "pub struct Leaf45 { v: u32, g: Ghost }",
        );
        let (code, report) = placement("pg18", &text);
        assert_eq!(
            code, 1,
            "an external token no 1.9 row decides is a finding: {report}"
        );
        assert!(
            report.contains("EXTERNAL:   Leaf45 names Ghost, which no 1.9 row decides"),
            "the guard names the declaration and the token: {report}"
        );
    }

    #[test]
    fn placement_pg19_derive_closure_break_is_a_finding() {
        let text = plant_declaration(
            &placement_document(),
            "/// A derive over a field with no such trait.\n#[derive(Default)]\n\
pub struct Cap { a: Leaf01 }\n",
            &["Cap"],
        );
        let (code, report) = placement("pg19", &text);
        assert_eq!(code, 1, "a derive-closure break is a finding: {report}");
        assert!(
            report.contains("CLOSURE:    duet::Cap derives Default over a: Leaf01 has no Default"),
            "the guard names the trait, the field, and the expression: {report}"
        );
    }

    #[test]
    fn placement_pg20_field_type_in_an_unreached_crate_is_a_finding() {
        let text = plant(&placement_document(), "`Leaf01`, ", "");
        let text = plant(
            &text,
            "| `duet-ae` | none | 9.1 |",
            "| `duet-ae` | `Leaf01` | 9.1 |",
        );
        let (code, report) = placement("pg20", &text);
        assert_eq!(
            code, 1,
            "a field type in an unreached crate is a finding: {report}"
        );
        assert!(
            report.contains("REACH:      duet::EngineProcess holds Leaf01, which lives in duet-ae"),
            "the guard names the declaration, the type, and the crate: {report}"
        );
    }

    #[test]
    fn placement_pg20b_constant_in_an_unreached_crate_is_a_finding() {
        let text = plant(
            &placement_document(),
            "pub const SYN_000: usize = 1;",
            "// duet-ae",
        );
        let text = plant(
            &text,
            "pub const SYN_001: usize = 1;",
            "pub const CAP_FAR: usize = 2;",
        );
        let text = plant(
            &text,
            "pub struct Leaf45 { v: u32 }",
            "pub struct Leaf45 { v: [u32; CAP_FAR] }",
        );
        let (code, report) = placement("pg20b", &text);
        assert_eq!(
            code, 1,
            "a constant in an unreached crate is a finding: {report}"
        );
        assert!(
            report.contains("CONST:      duet::Leaf45.v names CAP_FAR, which lives in duet-ae"),
            "the guard names the field and the crate that declares the constant: {report}"
        );
    }

    #[test]
    fn placement_pg21_expectation_appendix_b1_omits_is_a_finding() {
        let text = plant_declaration(
            &placement_document(),
            "/// An expectation Appendix B.1 does not list.\n\
#[expect(missing_copy_implementations, reason = \"a synthetic reason\")]\n\
pub struct Cap { v: u32 }\n",
            &["Cap"],
        );
        let (code, report) = placement("pg21", &text);
        assert_eq!(
            code, 1,
            "an expectation Appendix B.1 omits is a finding: {report}"
        );
        assert!(
            report.contains(
                "B.1:        Cap carries a missing_copy_implementations expectation and \
Appendix B.1 omits it"
            ),
            "the guard names the site and the lint: {report}"
        );
    }

    #[test]
    fn placement_pg22_hash_derive_with_no_vr1_row_is_a_finding() {
        let text = plant_declaration(
            &placement_document(),
            "/// A Hash derive with no VR1 row.\n#[derive(Hash)]\npub struct Cap { v: u32 }\n",
            &["Cap"],
        );
        let (code, report) = placement("pg22", &text);
        assert_eq!(
            code, 1,
            "a Hash derive the VR1 table omits is a finding: {report}"
        );
        assert!(
            report.contains("VR1:        duet::Cap derives Hash with no VR1 row"),
            "the guard names the declaration and the derive: {report}"
        );
    }

    #[test]
    fn placement_pg23_partial_eq_with_no_eq_is_a_finding() {
        let text = plant_declaration(
            &placement_document(),
            "/// A PartialEq derive with no Eq.\n#[derive(PartialEq)]\npub struct Cap { v: u32 }\n",
            &["Cap"],
        );
        let (code, report) = placement("pg23", &text);
        assert_eq!(
            code, 1,
            "a PartialEq derive with no Eq is a finding: {report}"
        );
        assert!(
            report.contains("EQ:         duet::Cap derives PartialEq and every field supplies Eq"),
            "the guard names the declaration the nursery lint refuses: {report}"
        );
    }

    #[test]
    fn placement_pg24_wide_arm_spread_is_a_finding() {
        let text = plant_declaration(
            &placement_document(),
            "/// An enum with a wide arm spread.\npub enum Big { Wide([u32; 64]), Narrow(u32) }\n",
            &["Big"],
        );
        let (code, report) = placement("pg24", &text);
        assert_eq!(
            code, 1,
            "an arm spread the lint refuses is a finding: {report}"
        );
        assert!(
            report
                .contains("VARIANT:    duet::Big: arm Wide is 256 bytes and the next largest is 4"),
            "the guard names the arm, its size, and the next largest: {report}"
        );
    }

    #[test]
    fn placement_pg26_heap_inside_audio_owned_state_is_a_finding() {
        let text = plant(
            &plant_containers(&placement_document()),
            "pub struct Aud01 { v: u32 }",
            "pub struct Aud01 { v: Box<u32> }",
        );
        let (code, report) = placement("pg26", &text);
        assert_eq!(
            code, 1,
            "a heap allocation inside audio-owned state is a finding: {report}"
        );
        assert!(
            report.contains("HEAP:       duet::Aud01 holds Box<u32> through v"),
            "the guard names the root, the expression, and the field path: {report}"
        );
    }

    #[test]
    fn placement_pg26b_container_that_grows_inside_audio_state_is_a_finding() {
        let text = plant(
            &plant_containers(&placement_document()),
            "pub struct Aud02 { v: u32 }",
            "pub struct Aud02 { v: Vec<u32> }",
        );
        let (code, report) = placement("pg26b", &text);
        assert_eq!(
            code, 1,
            "a container that grows inside audio state is a finding: {report}"
        );
        assert!(
            report.contains("GROW:       duet::Aud02 holds Vec<u32> through v"),
            "the guard names the root and the container that grows: {report}"
        );
    }

    #[test]
    fn placement_pg26c_lock_inside_audio_owned_state_is_a_finding() {
        let text = plant(
            &plant_containers(&placement_document()),
            "pub struct Aud03 { v: u32 }",
            "pub struct Aud03 { v: Mutex<u32> }",
        );
        let (code, report) = placement("pg26c", &text);
        assert_eq!(
            code, 1,
            "a lock inside audio-owned state is a finding: {report}"
        );
        assert!(
            report.contains("LOCK:       duet::Aud03 holds Mutex<u32> through v"),
            "the guard names the root and the lock it holds: {report}"
        );
    }

    #[test]
    fn placement_pg26d_root_block_and_marker_set_disagree_is_a_finding() {
        let text = plant(
            &placement_document(),
            "/// A synthetic root. **Audio-owned**\npub struct Aud01",
            "/// A synthetic root.\npub struct Aud01",
        );
        let (code, report) = placement("pg26d", &text);
        assert_eq!(code, 1, "a root with no marker is a finding: {report}");
        assert!(
            report.contains(
                "ROOT:       Aud01: the block names it and its declaration carries no marker"
            ),
            "the guard names the root the two sources disagree about: {report}"
        );
    }

    #[test]
    fn placement_pg26e_reachable_declaration_that_is_no_root_or_leaf_is_a_finding() {
        let text = plant(&placement_document(), "Leaf45\n```", "Car00\n```");
        let (code, report) = placement("pg26e", &text);
        assert_eq!(
            code, 1,
            "a break in the audio closure is a finding: {report}"
        );
        assert!(
            report.contains(
                "CLOSURE:    Leaf45: EngineProcess reaches it through a declared field and it \
is neither an audio-owned root nor an audio-reachable leaf"
            ),
            "the guard names the declaration the closure reaches: {report}"
        );
    }

    #[test]
    fn placement_pg26f_asserted_block_and_root_set_disagree_is_a_finding() {
        let text = plant(&placement_document(), "```text\nAud30", "```text\nLeaf00");
        let (code, report) = placement("pg26f", &text);
        assert_eq!(
            code, 1,
            "an asserted root the block omits is a finding: {report}"
        );
        assert!(
            report.contains(
                "ASSERTED:   Aud30: the root set names it, the closure does not reach it, and \
the asserted block does not name it"
            ),
            "the guard names the root the third source omits: {report}"
        );
    }

    #[test]
    fn placement_pg27_marker_the_register_disagrees_with_fails_closed() {
        let text = plant(
            &placement_document(),
            "id=drop-list rows>=8",
            "id=drop-list rows>=9",
        );
        let (code, report) = placement("pg27-marker", &text);
        assert_eq!(
            code, 2,
            "a marker the register disagrees with is fail-closed: {report}"
        );
        assert!(
            report.contains(
                "FAIL: the `drop-list` block of this document: the marker states rows>=9 and \
the register states 8; the guard is fail-closed (DR7)."
            ),
            "the guard names the block and the two counts: {report}"
        );
    }

    #[test]
    fn placement_pg27_row_that_names_no_referent_is_a_finding() {
        let text = plant(
            &placement_document(),
            "pins not-declared",
            "pins declared-name",
        );
        let (code, report) = placement("pg27-member", &text);
        assert_eq!(
            code, 1,
            "a row that names no referent is a finding: {report}"
        );
        assert!(
            report.contains("MEMBER:     pins: tpaa: no Rust block of this document declares it"),
            "the guard names the block, the row, and the reason: {report}"
        );
    }

    #[test]
    fn placement_pg27b_floor_below_the_row_count_is_a_finding() {
        let text = plant(
            &placement_document(),
            "PartialOrd\n```",
            "PartialOrd\nRc\n```",
        );
        let (code, report) = placement("pg27b", &text);
        assert_eq!(
            code, 1,
            "a floor below the row count is a finding: {report}"
        );
        assert!(
            report.contains(
                "FLOOR:      drop-list: the block holds 9 rows and the stated floor is 8"
            ),
            "the guard names the block, its rows, and its floor: {report}"
        );
    }

    #[test]
    fn placement_pg28_shared_limit_an_enforcer_cannot_reach_is_a_finding() {
        let text = plant(
            &placement_document(),
            "CAP_004 duet-ab duet-ab",
            "CAP_004 duet-ab duet-ao",
        );
        let (code, report) = placement("pg28", &text);
        assert_eq!(
            code, 1,
            "an enforcer that cannot reach the owner is a finding: {report}"
        );
        assert!(
            report.contains("LIMIT:      CAP_004: duet-ao: duet-ao does not reach duet-ab"),
            "the guard names the constant, the enforcer, and the owner: {report}"
        );
    }

    #[test]
    fn placement_pg29_rule_no_probe_row_carries_is_a_finding() {
        let mut tools = placement_tools();
        tools[0] = ("placement_check.py", "# PG2 PG3 rule\n".to_owned());
        let (code, report) = placement_run("pg29", &placement_document(), &tools, true);
        assert_eq!(
            code, 1,
            "a rule the probe table omits is a finding: {report}"
        );
        assert!(
            report.contains("PROBE:      PG3: the probe table carries no row"),
            "the guard names the rule the prototypes implement: {report}"
        );
    }

    #[test]
    fn placement_pg29_prototype_that_does_not_open_fails_closed() {
        let tools = vec![("placement_check.py", "# PG2 rule\n".to_owned())];
        let (code, report) = placement_run("pg29-absent", &placement_document(), &tools, true);
        assert_eq!(
            code, 2,
            "a prototype that does not open is fail-closed: {report}"
        );
        assert!(
            report.contains(
                "FAIL: the prototype `conversion_check.py` does not open, so the rule set is \
unknown; the guard is fail-closed (PG29)."
            ),
            "the guard names the prototype it cannot read: {report}"
        );
    }

    #[test]
    fn placement_pg30_used_by_cell_that_cites_nothing_is_a_finding() {
        let text = plant(
            &placement_document(),
            "| B1 | 4 | a synthetic bound |  |",
            "| B1 | 4 | a synthetic bound | 9.1 |",
        );
        let (code, report) = placement("pg30", &text);
        assert_eq!(
            code, 1,
            "a `Used by` cell that cites nothing is a finding: {report}"
        );
        assert!(
            report.contains(
                "USED BY:    B1: the `Used by` cell names section 9.1 and it cites nothing"
            ),
            "the guard names the budget and the section: {report}"
        );
    }

    #[test]
    fn placement_pg31_link_that_runs_backward_is_a_finding() {
        let text = plant(
            &placement_document(),
            "### 13.3 The phases\n",
            "### 13.4 The links\n\n| Link | Why |\n|---|---|\n\
| A2 before A1 | a synthetic link |\n\n### 13.3 The phases\n",
        );
        let (code, report) = placement("pg31", &text);
        assert_eq!(code, 1, "a link that runs backward is a finding: {report}");
        assert!(
            report.contains("LINK:       A2 before A1: A2 is in phase 1 and A1 is in phase 0"),
            "the guard names the link and the two phases: {report}"
        );
    }

    #[test]
    fn placement_pg31b_same_phase_crate_edge_with_no_exemption_is_a_finding() {
        let text = plant(&placement_document(), "C duet\n", "C duet-aa\n");
        let (code, report) = placement("pg31b", &text);
        assert_eq!(
            code, 1,
            "a same-phase crate edge with no exemption is a finding: {report}"
        );
        assert!(
            report.contains(
                "PAIR:       A1 and C1: both in phase 0, and duet depends on duet-aa; no \
exemption row states why the edge does not bind"
            ),
            "the guard names the pair, the phase, and the edge: {report}"
        );
    }

    #[test]
    fn placement_pg33_suppression_register_that_is_not_one_set_is_a_finding() {
        let text = plant(
            &placement_document(),
            "one functions carry a suppression: `probe_fn`.",
            "one functions carry a suppression: `ghost_fn`.",
        );
        let (code, report) = placement("pg33", &text);
        assert_eq!(
            code, 1,
            "a suppression register that is not one set is a finding: {report}"
        );
        assert!(
            report.contains(
                "SUPPRESS:   ghost_fn: the suppression register names it and no Rust block \
declares the function"
            ),
            "the guard names the third source the register misses: {report}"
        );
    }

    #[test]
    fn placement_pg34_cost_bullet_the_phase_table_refutes_is_a_finding() {
        let text = plant(
            &placement_document(),
            "sixteen phases replace four",
            "fifteen phases replace four",
        );
        let (code, report) = placement("pg34", &text);
        assert_eq!(
            code, 1,
            "a cost bullet the phase table refutes is a finding: {report}"
        );
        assert!(
            report.contains(
                "TAIL:       SM6: the cost bullet states fifteen phases and the table holds 16"
            ),
            "the guard names the stated count and the measured one: {report}"
        );
    }

    #[test]
    fn placement_pg35_lock_writer_sequence_the_chunks_refute_is_a_finding() {
        let text = plant(
            &placement_document(),
            "phases 0 to 15: 0, 0,",
            "phases 0 to 15: 1, 0,",
        );
        let (code, report) = placement("pg35", &text);
        assert_eq!(
            code, 1,
            "a stated sequence the chunk tables refute is a finding: {report}"
        );
        assert!(
            report.contains("LOCK SEQ:   13.3: the stated sequence is 1, 0,"),
            "the guard states the sequence it read and the one it derived: {report}"
        );
    }

    #[test]
    fn placement_pg36_pin_owner_the_appendices_disagree_about_is_a_finding() {
        let text = plant(
            &placement_document(),
            "| `tpaa` | a synthetic pin | M1 |",
            "| `tpaa` | a synthetic pin | M2 |",
        );
        let (code, report) = placement("pg36", &text);
        assert_eq!(
            code, 1,
            "a pin owner the appendices disagree about is a finding: {report}"
        );
        assert!(
            report.contains(
                "PIN OWNER:  tpaa: Appendix B.3 names owner M2 and 13.1 gives the pin to M1"
            ),
            "the guard names the pin and the two owners: {report}"
        );
    }

    #[test]
    fn placement_pg37_budget_value_the_citing_line_refutes_is_a_finding() {
        let text = plant(
            &placement_document(),
            "| B1 | 4 | a synthetic bound |  |",
            "| B1 | 5 | a synthetic bound |  |",
        );
        let (code, report) = placement("pg37", &text);
        assert_eq!(
            code, 1,
            "a budget value the citing line refutes is a finding: {report}"
        );
        assert!(
            report.contains(
                "VALUE:      B1: the row states 5 and the declaration line that cites it writes 4"
            ),
            "the guard names the budget, the row value, and the declared capacity: {report}"
        );
    }

    #[test]
    fn placement_pg38_ragged_table_row_is_a_finding() {
        let text = plant(
            &placement_document(),
            "| Audio | the engine | the graph | allocate |",
            "| Audio | the engine | the graph | allocate | extra |",
        );
        let (code, report) = placement("pg38", &text);
        assert_eq!(
            code, 1,
            "a row whose cell count differs from its header is a finding: {report}"
        );
        assert!(
            report.contains("RAGGED:     Audio: the row holds 5 cells and its header holds 4"),
            "the guard names the row and the two cell counts: {report}"
        );
    }

    #[test]
    fn placement_pg39_index_that_omits_a_live_id_is_a_finding() {
        let text = plant(
            &placement_document(),
            "| PP2 | probes the open | 1.9 |\n",
            "",
        );
        let (code, report) = placement("pg39", &text);
        assert_eq!(code, 1, "an id section 1.7 omits is a finding: {report}");
        assert!(
            report.contains("INDEX:      PP2: the rule set holds it and section 1.7 omits it"),
            "the guard names the id the index omits: {report}"
        );
    }

    /// The fixture with chunk row A1 writing one other path, run through the guard.
    fn placement_pg40_row(label: &str, path: &str) -> (i32, String) {
        let text = plant(
            &placement_document(),
            PLACEMENT_A1_ROW,
            &format!("| A1 | 0 | a synthetic chunk | writes {path} | a synthetic gate |"),
        );
        placement(label, &text)
    }

    #[test]
    fn placement_pg40_baseline_parses_every_chunk_path() {
        let (code, report) = placement("pg40-base", &placement_document());
        assert_eq!(code, 0, "every chunk path of the fixture parses: {report}");
        assert!(
            report.contains("LINE CHUNKS:     64     CHUNK PATHS: 63     CHUNK CRATE BAD: 0"),
            "the guard prints the chunk rows and the parsed paths it decided: {report}"
        );
    }

    #[test]
    fn placement_pg40_chunk_that_writes_outside_its_line_is_a_finding() {
        let (code, report) =
            placement_pg40_row("pg40", "`crates/bc_synth/duet-aa/lang_rust/src/one.rs`");
        assert_eq!(
            code, 1,
            "a chunk that writes outside its own line is a finding: {report}"
        );
        assert!(
            report.contains(
                "CHUNK CRATE:A1: the row writes into crate `duet-aa` and line `A` owns `duet` (PG40)"
            ),
            "the guard names the chunk, the crate, and the owning line: {report}"
        );
    }

    #[test]
    fn placement_pg40_path_in_the_old_shape_is_a_finding() {
        let (code, report) = placement_pg40_row("pg40-old", "`crates/duet-aa/src/one.rs`");
        assert_eq!(
            code, 1,
            "a path that skips the context directory is a finding: {report}"
        );
        assert!(
            report.contains(
                "CHUNK CRATE:A1: the row names `crates/duet-aa/src/one.rs`, a path outside the \
`crates/bc_<context>/<crate>/` shape (PG40)"
            ),
            "the guard names the chunk and the path that does not parse: {report}"
        );
    }

    #[test]
    fn placement_pg40_path_under_the_wrong_context_is_a_finding() {
        let (code, report) = placement_pg40_row(
            "pg40-context",
            "`crates/bc_synth/duet/lang_rust/src/one.rs`",
        );
        assert_eq!(
            code, 1,
            "a crate under a context the map does not give it is a finding: {report}"
        );
        assert!(
            report.contains(
                "CHUNK CRATE:A1: the row names `crates/bc_synth/duet/lang_rust/src/one.rs` under \
context `bc_synth` and the `context-map` block puts `duet` in `bc_app` (PG40)"
            ),
            "the guard names the path and both contexts: {report}"
        );
    }

    #[test]
    fn placement_pg40_context_map_that_omits_a_crate_is_a_finding() {
        let text = plant(
            &placement_document(),
            "\nduet-ao synth\n",
            "\nduet-zz synth\n",
        );
        let (code, report) = placement("pg40-map", &text);
        assert_eq!(
            code, 1,
            "a context map that omits a crate is a finding: {report}"
        );
        assert!(
            report.contains(
                "CHUNK CRATE:<context-map>: the `crate-table` block names `duet-ao` and the \
`context-map` block carries no row for it (PG40)"
            ),
            "the guard names the crate the map omits: {report}"
        );
        assert!(
            report.contains(
                "CHUNK CRATE:<context-map>: the `context-map` block names `duet-zz` and the \
`crate-table` block does not (PG40)"
            ),
            "the guard names the crate the map holds and the table does not: {report}"
        );
    }

    #[test]
    fn placement_pg40_no_parsed_path_fails_closed() {
        let text = placement_document().replace(PLACEMENT_SCOPE, "the workspace");
        let (code, report) = placement("pg40-zero", &text);
        assert_eq!(
            code, 1,
            "a rule with no path to decide is a finding: {report}"
        );
        assert!(
            report.contains(
                "CHUNK CRATE:PG40: the rule scanned 64 chunk rows and parsed no `crates/` path; \
a rule with no path to decide is a silent pass (PG40)"
            ),
            "the guard names the zero denominator: {report}"
        );
    }

    #[test]
    fn placement_pg41_carrier_row_with_no_declared_end_is_a_finding() {
        let text = plant(
            &placement_document(),
            "pub struct Car00 { w: triple_buffer::Input<u32>, r: triple_buffer::Output<u32> }",
            "pub struct Car00 { r: triple_buffer::Output<u32> }",
        );
        let (code, report) = placement("pg41", &text);
        assert_eq!(
            code, 1,
            "a carrier row with no declared end is a finding: {report}"
        );
        assert!(
            report.contains(
                "CARRIER:    msg00: `Car00.w` holds no carrier end, so it cannot be the write \
end of this row"
            ),
            "the guard names the message and the end it cannot find: {report}"
        );
    }

    /// The line the skipped form of the roster guard prints.
    ///
    /// No full run prints it, so the two forms of the guard are never
    /// confusable by exit code alone.
    const ROSTER_SKIP_LINE: &str = "ROSTER COMPILE:   skipped (--generate-only)";

    /// How many types the synthetic section 1.5 table places.
    const ROSTER_TYPES: usize = 20;

    /// How many impl blocks the synthetic roster document spells out.
    ///
    /// The `impl-sites` block states 66 rows against a floor of 65, so a probe
    /// that removes one row keeps the block above its floor and reaches the
    /// count rule instead of the fail-closed row count.
    const ROSTER_IMPL_SITES: usize = 66;

    /// The repository manifest the roster fixture gives the guard.
    ///
    /// The guard copies the `[workspace.lints]` table into the scratch
    /// workspace, and it reads the table between the rust heading and the
    /// profile banner, so the fixture carries both marks.
    const ROSTER_REPO_MANIFEST: &str = "[workspace]
members = []

[workspace.lints.rust]
unsafe_code = \"deny\"

[workspace.lints.rustdoc]
all = \"deny\"

# ---------------------------------------------------------------------------
# Profiles
[profile.release]
opt-level = 3
";

    /// A two-line workflow that gives `check-roster` the skip flag.
    const PLANTED_JOB_LINE: &str =
        "jobs:\n  - run: cargo xtask check-roster document scratch repo --generate-only\n";

    /// The one defect one roster fixture plants.
    #[derive(Debug, Clone, Copy)]
    enum RosterDefect {
        /// The document breaks no rule the skipped form measures.
        Clean,
        /// One declaration the section 1.5 table places is absent.
        MissingDeclaration,
        /// One impl block carries a body beside a bodiless signature.
        MixedImplBlock,
        /// The `impl-sites` block lists one site fewer than the document holds.
        MissingImplSite,
    }

    /// One type name the synthetic section 1.5 table places.
    fn roster_type(index: usize) -> String {
        format!("Rst{index:02}")
    }

    /// The crate row the synthetic section 1.5 table gives one type.
    fn roster_row(index: usize) -> &'static str {
        if index.is_multiple_of(2) {
            "probe-one"
        } else {
            "probe-two"
        }
    }

    /// One registered table block, as the synthetic document writes it.
    fn roster_table_block(heading: &str, id: &str, minimum: usize, rows: Vec<String>) -> String {
        let mut out = vec![
            format!("#### {heading}"),
            String::new(),
            format!("<!-- GUARD BLOCK id={id} rows>={minimum} -->"),
        ];
        out.extend(rows);
        out.push(String::new());
        out.join("\n")
    }

    /// One registered fenced block, as the synthetic document writes it.
    fn roster_fenced_block(head: RosterHead<'_>, language: &str, lines: Vec<String>) -> String {
        let mut out = vec![
            format!("#### {}", head.heading),
            String::new(),
            format!("<!-- GUARD BLOCK id={} rows>={} -->", head.id, head.minimum),
            format!("```{language}"),
        ];
        out.extend(lines);
        out.push("```".to_owned());
        out.push(String::new());
        out.join("\n")
    }

    /// What one registered block of the roster fixture calls itself.
    #[derive(Debug, Clone, Copy)]
    struct RosterHead<'a> {
        /// The `####` heading the guard register states.
        heading: &'a str,
        /// The block id the marker line states.
        id: &'a str,
        /// The row floor the marker line states.
        minimum: usize,
    }

    /// Every row of the synthetic type ownership table.
    fn roster_ownership() -> Vec<String> {
        let rows = (0..ROSTER_TYPES)
            .map(|index| {
                md_row(&[
                    &format!("`{}`", roster_row(index)),
                    &format!("`{}`", roster_type(index)),
                    "15.1",
                ])
            })
            .collect();
        md_table(&["Crate", "Types it declares", "Declared in"], rows)
    }

    /// Every line of the synthetic internal edge list.
    fn roster_edges() -> Vec<String> {
        let mut lines = vec!["probe-one -> probe-two".to_owned()];
        lines.extend((1..18).map(|index| format!("probe-edge-{index:02} -> probe-two")));
        lines
    }

    /// Every line of the synthetic external path list.
    fn roster_paths() -> Vec<String> {
        (0..70)
            .map(|index| format!("Ext{index:02}        std::probe::Ext{index:02}"))
            .collect()
    }

    /// Every line of the synthetic external pin list.
    fn roster_pins() -> Vec<String> {
        (0..13)
            .map(|index| format!("probe-pin-{index:02}   1.0.0"))
            .collect()
    }

    /// Every line of the synthetic workspace constant block.
    fn roster_constants() -> Vec<String> {
        (0..122)
            .map(|index| format!("pub const PROBE_{index:03}: usize = 0;"))
            .collect()
    }

    /// Every line of the synthetic substitution block.
    ///
    /// The guard refuses a digit after the head token of one line, so each
    /// line states its rule in words alone.
    fn roster_substitutions() -> Vec<String> {
        (1..=8)
            .map(|index| format!("S{index} probe-rule    the synthetic block states this rule"))
            .collect()
    }

    /// Every line of the synthetic recorded size block.
    fn roster_sizes() -> Vec<String> {
        (0..51)
            .map(|index| format!("Size{index:02}     8   8  generic"))
            .collect()
    }

    /// Every line of the synthetic Drop block.
    fn roster_drops() -> Vec<String> {
        (0..3).map(roster_type).collect()
    }

    /// Every line of the synthetic impl site block.
    fn roster_impl_sites() -> Vec<String> {
        (0..ROSTER_IMPL_SITES)
            .map(|index| format!("{} inherent {index:02}", roster_type(index % ROSTER_TYPES)))
            .collect()
    }

    /// The impl block the synthetic document spells out at one index.
    fn roster_impl_block(index: usize) -> String {
        format!(
            "impl {} {{ fn probe_{index:02}(&self) -> usize; }}",
            roster_type(index % ROSTER_TYPES)
        )
    }

    /// The unregistered Rust block that spells every declaration and impl out.
    fn roster_declarations() -> String {
        let mut out = vec![
            "#### The declarations the roster spells out".to_owned(),
            String::new(),
            "```rust".to_owned(),
        ];
        out.extend(
            (0..ROSTER_TYPES)
                .map(|index| format!("struct {} {{ probe: usize }}", roster_type(index))),
        );
        out.extend((0..ROSTER_IMPL_SITES).map(roster_impl_block));
        out.push("```".to_owned());
        out.push(String::new());
        out.join("\n")
    }

    /// The whole synthetic document, with the one defect the caller states.
    fn roster_document(defect: RosterDefect) -> String {
        let parts = vec![
            "# The synthetic roster document\n".to_owned(),
            roster_table_block(
                "The type ownership table",
                "ownership-table",
                16,
                roster_ownership(),
            ),
            roster_fenced_block(
                RosterHead {
                    heading: "The internal edge list",
                    id: "edge-list",
                    minimum: 18,
                },
                "text",
                roster_edges(),
            ),
            roster_fenced_block(
                RosterHead {
                    heading: "Where every external name comes from",
                    id: "external-paths",
                    minimum: 70,
                },
                "text",
                roster_paths(),
            ),
            roster_fenced_block(
                RosterHead {
                    heading: "The external crate pins the roster compile uses",
                    id: "pins",
                    minimum: 13,
                },
                "text",
                roster_pins(),
            ),
            roster_fenced_block(
                RosterHead {
                    heading: "Every workspace constant",
                    id: "constants",
                    minimum: 122,
                },
                "rust",
                roster_constants(),
            ),
            roster_fenced_block(
                RosterHead {
                    heading: "Every substitution the roster compile applies",
                    id: "substitutions",
                    minimum: 8,
                },
                "text",
                roster_substitutions(),
            ),
            roster_fenced_block(
                RosterHead {
                    heading: "Every size the guard records",
                    id: "recorded-sizes",
                    minimum: 51,
                },
                "text",
                roster_sizes(),
            ),
            roster_fenced_block(
                RosterHead {
                    heading: "Every declaration with a hand-written Drop impl",
                    id: "drop-impls",
                    minimum: 3,
                },
                "text",
                roster_drops(),
            ),
            roster_fenced_block(
                RosterHead {
                    heading: "Every impl block the roster compiles",
                    id: "impl-sites",
                    minimum: 65,
                },
                "text",
                roster_impl_sites(),
            ),
            roster_declarations(),
        ];
        roster_plant(&parts.join("\n"), defect)
    }

    /// The synthetic document with one defect planted in it.
    fn roster_plant(text: &str, defect: RosterDefect) -> String {
        match defect {
            RosterDefect::Clean => text.to_owned(),
            RosterDefect::MissingDeclaration => plant(
                text,
                &format!(
                    "struct {} {{ probe: usize }}\n",
                    roster_type(ROSTER_TYPES - 1)
                ),
                "",
            ),
            RosterDefect::MixedImplBlock => plant(
                text,
                &roster_impl_block(0),
                "impl Rst00 { fn probe_00(&self) -> usize; fn probe_mixed(&self) -> usize { 0 } }",
            ),
            RosterDefect::MissingImplSite => {
                let last = ROSTER_IMPL_SITES - 1;
                plant(
                    text,
                    &format!("{} inherent {last:02}\n", roster_type(last % ROSTER_TYPES)),
                    "",
                )
            },
        }
    }

    /// Run the roster guard over one synthetic document, and clean up.
    ///
    /// The fixture holds its own repository root and its own scratch
    /// directory, and the scratch sits beside the repository and never under
    /// it, because the guard refuses a scratch path inside the repository.
    fn roster_generate_only(label: &str, defect: RosterDefect) -> (i32, String) {
        let root = scratch(label);
        let document = root.join("architecture.md");
        write_bytes(&document, roster_document(defect).as_bytes());
        write_bytes(
            &root.join("repo").join("Cargo.toml"),
            ROSTER_REPO_MANIFEST.as_bytes(),
        );
        let report = xtask(
            &root,
            &[
                "check-roster",
                &document.display().to_string(),
                &root.join("work").display().to_string(),
                &root.join("repo").display().to_string(),
                "--generate-only",
            ],
        );
        clean(&root);
        report
    }

    /// Whether no line of one workflow gives `check-roster` the skip flag.
    ///
    /// The helper reads one LINE at a time, and that is its own limit. A
    /// folded YAML scalar, a shell variable that holds the flag, and an `env:`
    /// entry each defeat it, and review holds those three.
    fn roster_job_line_is_clean(workflow: &str) -> bool {
        !workflow
            .lines()
            .any(|line| line.contains("check-roster") && line.contains("--generate-only"))
    }

    /// The workflow directory of this repository.
    fn workflow_directory() -> PathBuf {
        repository_root().join(".github").join("workflows")
    }

    /// The line the skipped half of `check-roster` prints and the full run never does.
    ///
    /// A planted run asserts it as well as the clean run, because an exit-1 run
    /// is where a skipped form and a full form are most confusable.
    const SKIPPED_COMPILE_LINE: &str = "ROSTER COMPILE:   skipped (--generate-only)";

    #[test]
    fn roster_generate_only_clean_document_exits_zero() {
        let (code, report) = roster_generate_only("roster-clean", RosterDefect::Clean);
        assert_eq!(
            code, 0,
            "a synthetic document that breaks no rule exits 0: {report}"
        );
        assert!(
            report.contains(ROSTER_SKIP_LINE),
            "the skipped form prints the line that names itself: {report}"
        );
    }

    #[test]
    fn roster_generate_only_roster_below_the_denominator_is_a_finding() {
        let (code, report) = roster_generate_only("roster-floor", RosterDefect::MissingDeclaration);
        assert_eq!(
            code, 1,
            "a roster below the section 1.5 floor is a finding: {report}"
        );
        assert!(
            report
                .lines()
                .any(|line| line.starts_with("FINDING: the roster holds")),
            "the guard names the first name with no declaration: {report}"
        );
        assert!(
            report.contains(SKIPPED_COMPILE_LINE),
            "a planted run states which half ran, which is where the two forms are most confusable: {report}"
        );
    }

    #[test]
    fn roster_generate_only_mixed_impl_block_is_a_finding() {
        let (code, report) = roster_generate_only("roster-mixed", RosterDefect::MixedImplBlock);
        assert_eq!(code, 1, "a mixed impl block is a finding: {report}");
        assert!(
            report
                .lines()
                .any(|line| line.starts_with("FINDING: the impl block for")),
            "the guard names the declaration the mixed block is for: {report}"
        );
        assert!(
            report.contains(SKIPPED_COMPILE_LINE),
            "a planted run states which half ran, which is where the two forms are most confusable: {report}"
        );
    }

    #[test]
    fn roster_generate_only_impl_count_that_differs_is_a_finding() {
        let (code, report) = roster_generate_only("roster-impls", RosterDefect::MissingImplSite);
        assert_eq!(
            code, 1,
            "an impl count that differs from the block is a finding: {report}"
        );
        assert!(
            report
                .lines()
                .any(|line| line.starts_with("FINDING: the roster parsed")),
            "the guard names both counts and the difference: {report}"
        );
        assert!(
            report.contains(SKIPPED_COMPILE_LINE),
            "a planted run states which half ran, which is where the two forms are most confusable: {report}"
        );
    }

    #[test]
    fn roster_generate_only_never_reaches_a_job() {
        let directory = workflow_directory();
        let entries = fs::read_dir(&directory).expect("the workflow directory opens");
        let mut files = 0_usize;
        for entry in entries {
            let path = entry.expect("one entry of the workflow directory").path();
            if !path.is_file() {
                continue;
            }
            let text = fs::read_to_string(&path).expect("one workflow file");
            assert!(
                roster_job_line_is_clean(&text),
                "no job gives check-roster the skip flag: {}",
                path.display()
            );
            files += 1;
        }
        assert!(
            files > 0,
            "the workflow directory holds at least one file, so the probe has a denominator"
        );
    }

    #[test]
    fn roster_job_line_probe_refuses_a_planted_flag() {
        assert!(
            !roster_job_line_is_clean(PLANTED_JOB_LINE),
            "a planted skip flag makes the job-line helper return false"
        );
    }
}
