//! Probes for the four `xtask` guards: `check-conversions`, `check-manifests`,
//! `check-plan-graph`, and `check-closure`.
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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{env, fs};

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

    /// A scratch directory this probe owns, outside the repository.
    fn scratch(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let dir = env::temp_dir().join(format!(
            "xtask-probe-{label}-{}-{nanos}",
            std::process::id()
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
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/duet-time",
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
            report.contains("  CAST: crates/duet-time/src/other.rs: line 2"),
            "the finding names the sibling file: {report}"
        );
    }

    #[test]
    fn conversion_cg7_a_suppression_inside_the_exempt_file_is_exempt() {
        let (code, report) = conversions(
            "cg7-exempt-suppression",
            &root_manifest(&["crates/*"], &[]),
            &[Member::lib(
                "crates/duet-time",
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
                    chunk("A1", "core", &[], &["crates/alpha/src/a.rs"]),
                ),
                (
                    "b2.md",
                    chunk("B2", "core", &["A1"], &["crates/beta/src/b.rs"]),
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
                chunk("A1", "core", &["Z9"], &["crates/alpha/src/a.rs"]),
            )],
        );
        assert_eq!(code, 1, "an unknown dependency is a finding: {report}");
        assert!(
            report.contains("FINDING: A1 depends on Z9, which has no chunk file"),
            "the finding names the chunk and the missing id: {report}"
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
                    chunk("A1", "core", &["B1"], &["crates/alpha/src/a.rs"]),
                ),
                (
                    "b1.md",
                    chunk("B1", "edge", &["A1"], &["crates/beta/src/b.rs"]),
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
                    chunk("A1", "core", &["B2"], &["crates/alpha/src/a.rs"]),
                ),
                ("b2.md", chunk("B2", "edge", &[], &["crates/beta/src/b.rs"])),
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
        let shared = "crates/shared/src/lib.rs";
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
                    chunk("A1", "core", &[], &["crates/alpha/src/a.rs"]),
                ),
                ("b1.md", chunk("B1", "core", &[], &["crates/beta/src/b.rs"])),
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
                chunk("A1", "core", &[], &["crates/alpha/src/a.rs"]),
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
                    chunk("A1", "core", &[], &["crates/alpha/src/a.rs"]),
                ),
                ("b2.md", chunk("B2", "edge", &[], &["crates/beta/src/b.rs"])),
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

    /// The plan name the guard derives from the throwaway document.
    ///
    /// The guard reads the directory that holds the document, which every
    /// fixture names `plan`.
    const CLOSURE_PLAN: &str = "roadmap/plan";

    /// A plan-store key that carries the suffix of another review (`CL1c`).
    const CLOSURE_OTHER_KEY: &str = "36304kihouge4-review-r21-inner";

    /// The repository the closure guard resolves at compile time.
    ///
    /// The guard reads `.claude/plan-coordination/db.sh` of the repository two
    /// levels above its own crate, so a probe of `CL1c` resolves the same
    /// launcher the same way.
    fn guard_repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("the repository root sits two levels above this crate")
            .to_path_buf()
    }

    /// A run token no other closure probe shares.
    ///
    /// The token opens with a letter after the revision number, because the id
    /// generator reads a digit run there as a longer revision.
    fn run_token() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        format!("r99z{}z{nanos}", std::process::id())
    }

    /// Append one value to the throwaway store and return the key it minted.
    fn store_append(root: &Path, suffix: &str, value: &str) -> String {
        let launcher = guard_repo()
            .join(".claude")
            .join("plan-coordination")
            .join("db.sh");
        let output = Command::new("bash")
            .arg(&launcher)
            .args(["append", CLOSURE_PLAN, suffix, value])
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

    /// The whole throwaway closure section, which records the minted key.
    fn closure_block(key: &str) -> String {
        format!(
            "### 1.9 A probe section\n\n\
             #### The revision-16 review, over the frozen document\n\n\
             {CLOSURE_SENTENCE}\n\n\
             {}\n\
             The review file this block records is stored at plan-store key `{key}` ({CLOSURE_BLOCK}).\n\n\
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
    }

    impl Closure {
        /// Write the review, append it to a throwaway store, and mint its key.
        fn new(label: &str) -> Self {
            let root = scratch(label);
            let token = run_token();
            let review = root.join("review").join(format!("critic-spec-{token}.md"));
            write_bytes(&review, CLOSURE_REVIEW.as_bytes());
            let key = store_append(&root, &format!("review-{token}"), CLOSURE_REVIEW);
            Self { root, token, key }
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

        /// The baseline closure section, which records the minted key.
        fn block(&self) -> String {
            closure_block(&self.key)
        }

        /// Append one more copy of the review under this fixture's suffix.
        fn append(&self, value: &str) -> String {
            store_append(&self.root, &self.suffix(), value)
        }

        /// Run the guard over one document text and this fixture's review.
        fn guard(&self, document: &str) -> (i32, String) {
            self.guard_with(document, &self.review(), CLOSURE_BLOCK)
        }

        /// Run the guard over one document text, review path, and block id.
        fn guard_with(&self, document: &str, review: &Path, block: &str) -> (i32, String) {
            let path = self.root.join("plan").join("architecture.md");
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
    fn closure_baseline_is_clean() {
        let (code, report) = planted("cl-base", str::to_owned);
        assert_eq!(code, 0, "a well-formed throwaway pair is clean: {report}");
        assert!(
            report.starts_with("REVIEW: critic-spec-r99z"),
            "the summary names the review this run read: {report}"
        );
        assert!(
            report.contains(
                "BLOCK: closure-r16   GENERATED: 4   ROWS: 4   STORE: matches   STORE COPIES: 1   CLOSURE BAD: 0"
            ),
            "the summary states the generated set, the rows, and the stored copy: {report}"
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
}
