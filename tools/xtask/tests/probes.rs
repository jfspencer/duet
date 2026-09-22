//! Probes for the three `xtask` guards: `check-conversions`, `check-manifests`,
//! and `check-plan-graph`.
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
}
