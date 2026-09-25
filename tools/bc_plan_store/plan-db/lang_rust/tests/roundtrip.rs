//! End-to-end check of the CLI against a real LMDB env in a scratch directory.

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{env, fs};

    /// A scratch directory that is NOT inside a git repository, so the plan key
    /// falls back to the path-based form and never touches `~/.claude/plan-dbs`.
    fn scratch() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let dir = env::temp_dir().join(format!("plan-db-test-{}-{nanos}", std::process::id()));
        fs::create_dir_all(dir.join("work").join("roadmap").join("alpha")).expect("scratch layout");
        dir
    }

    /// Run the binary with the scratch layout and return `(exit code, stdout)`.
    fn db(root: &Path, args: &[&str]) -> (i32, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_plan-db"))
            .args(args)
            .current_dir(root.join("work"))
            .env("PLAN_DB_ROOT", root.join("dbs"))
            .env_remove("GIT_DIR")
            .env("GIT_CEILING_DIRECTORIES", root)
            .output()
            .expect("spawn plan-db");
        let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
        (output.status.code().unwrap_or(-1), stdout)
    }

    #[test]
    fn full_lifecycle_round_trips() {
        let root = scratch();
        let plan = "roadmap/alpha";

        let (resolve_code, resolved) = db(&root, &["resolve", plan]);
        assert_eq!(resolve_code, 0, "resolve exits 0");
        assert!(
            resolved.trim().ends_with("roadmap__alpha"),
            "fallback key is <parent basename>__<leaf>: {resolved}"
        );

        let (init_code, _) = db(&root, &["init", plan]);
        assert_eq!(init_code, 0, "init exits 0");
        let (_, seeded_ctx) = db(&root, &["get", plan, "current_context"]);
        assert_eq!(
            seeded_ctx.trim(),
            r#"{"resume_mode":"fresh","seq":0}"#,
            "seeded context"
        );
        let (_, seeded_signal) = db(&root, &["get", plan, "control:signal"]);
        assert_eq!(seeded_signal.trim(), "run", "seeded signal");

        let (put_code, _) = db(&root, &["put", plan, "control:signal", "pause"]);
        assert_eq!(put_code, 0, "put exits 0");
        let (_, paused) = db(&root, &["get", plan, "control:signal"]);
        assert_eq!(paused.trim(), "pause", "put overwrote");

        db(&root, &["init", plan]);
        let (_, still_paused) = db(&root, &["get", plan, "control:signal"]);
        assert_eq!(still_paused.trim(), "pause", "init is idempotent");

        let (append_code, minted) = db(
            &root,
            &["append", plan, "Orch1 Report", "line one\nline two"],
        );
        assert_eq!(append_code, 0, "append exits 0");
        let minted = minted.trim().to_owned();
        assert!(
            minted.ends_with("-orch1-report"),
            "minted key carries the suffix: {minted}"
        );

        let (_, scanned) = db(&root, &["scan", plan, "control:"]);
        assert_eq!(scanned.trim(), "control:signal", "prefix scan");
        let (_, scanned_values) = db(&root, &["scan", plan, &minted, "-v"]);
        let record: serde_json::Value =
            serde_json::from_str(scanned_values.trim()).expect("one json record");
        assert_eq!(
            record["value"], "line one\nline two",
            "value survives newlines in one line"
        );

        let (_, sized) = db(&root, &["len", plan, &minted]);
        let row: serde_json::Value = serde_json::from_str(sized.trim()).expect("json row");
        assert_eq!(row["bytes"], 17, "byte length");
        assert_eq!(row["approx_tokens"], 5, "ceil(17/4)");

        let (_, index) = db(&root, &["keys", plan]);
        assert_eq!(index.lines().count(), 3, "three keys indexed");

        let (del_code, _) = db(&root, &["del", plan, "control:signal"]);
        assert_eq!(del_code, 0, "del exits 0");
        let (_, gone) = db(&root, &["get", plan, "control:signal"]);
        assert_eq!(gone, "", "deleted key prints nothing");

        let (bad_code, _) = db(&root, &["get", plan, "bad key"]);
        assert_eq!(bad_code, 1, "key alphabet violation exits 1");

        fs::remove_dir_all(&root).expect("cleanup");
    }
}
