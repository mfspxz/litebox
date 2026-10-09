// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! Hostile-guest containment suite.
//!
//! Each test launches a hostile fixture as a LiteBox guest via the Linux
//! userland runner, asserts *containment* (not just "it ran"), and writes a
//! JSON receipt to `hostile-tests/receipts/<fixture>.json`.
//!
//! A fixture the current LiteBox cannot contain gets verdict `"gap"` in its
//! receipt and fails the test with the evidence attached — a failing
//! containment is a finding, not a test bug.

use hostile_tests::{fixture_path, repo_root, runner_bin, FIXTURES};
use serde::Serialize;
use std::io::Read as _;
use std::os::unix::process::ExitStatusExt as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct Receipt {
    fixture: String,
    sha256: String,
    policy: String,
    verdict: String,
    ts: String,
    detail: String,
}

struct GuestResult {
    code: Option<i32>,
    signal: Option<i32>,
    timed_out: bool,
    stdout: String,
}

/// Run a fixture as a LiteBox guest. `mem_limit_mb` sets RLIMIT_AS on the
/// runner+guest process tree (per-process, inherited across fork).
fn run_guest(fixture: &str, timeout: Duration, mem_limit_mb: Option<u64>) -> GuestResult {
    let fixture_bin = fixture_path(fixture);
    // The runner drops privileges in its sandbox; ensure it can read the fixture.
    let tmp = std::env::temp_dir().join(format!("hostile_{fixture}"));
    std::fs::copy(&fixture_bin, &tmp).expect("copy fixture to temp");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let mut cmd = Command::new(runner_bin());
    cmd.args(["--unstable", "--rewrite-syscalls"])
        .arg(&tmp)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    if let Some(mb) = mem_limit_mb {
        // SAFETY: pre_exec runs after fork, before exec, in the child only.
        // setrlimit is async-signal-safe.
        unsafe {
            use std::os::unix::process::CommandExt as _;
            cmd.pre_exec(move || {
                let lim = libc::rlimit {
                    rlim_cur: mb * 1024 * 1024,
                    rlim_max: mb * 1024 * 1024,
                };
                if libc::setrlimit(libc::RLIMIT_AS, std::ptr::addr_of!(lim)) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    let mut child = cmd.spawn().expect("spawn runner");
    let deadline = Instant::now() + timeout;
    let timed_out = loop {
        if child.try_wait().expect("try_wait").is_some() {
            break false;
        }
        if Instant::now() >= deadline {
            child.kill().expect("kill timed-out guest");
            break true;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let status = child.wait().expect("wait");
    let mut stdout = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut stdout)
        .unwrap();
    // stderr is piped to avoid blocking the child on a full buffer; assertions
    // use stdout + exit status.
    drop(child.stderr.take());

    GuestResult {
        code: status.code(),
        signal: status.signal(),
        timed_out,
        stdout,
    }
}

fn sha_pin(fixture: &str) -> String {
    FIXTURES
        .iter()
        .find(|(n, _)| *n == fixture)
        .unwrap()
        .1
        .to_string()
}

fn ts_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Minimal ISO-8601 UTC rendering without extra deps.
    format!("{secs}")
}

/// Evaluate evidence, write the receipt, and assert containment.
/// Returns () on "contained"; panics with the gap evidence on "gap".
fn conclude(fixture: &str, policy: &str, contained: bool, detail: String) {
    let verdict = if contained { "contained" } else { "gap" };
    let receipt = Receipt {
        fixture: fixture.to_string(),
        sha256: sha_pin(fixture),
        policy: policy.to_string(),
        verdict: verdict.to_string(),
        ts: ts_now(),
        detail: detail.clone(),
    };
    let dir: PathBuf = repo_root().join("hostile-tests/receipts");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{fixture}.json"));
    std::fs::write(&path, serde_json::to_string_pretty(&receipt).unwrap()).unwrap();
    println!("receipt: {}", path.display());
    assert!(
        contained,
        "CONTAINMENT GAP in {fixture}: {detail} (documented in {})",
        path.display()
    );
}

#[test]
fn a_proc_mem_is_contained() {
    let r = run_guest("a_proc_mem", Duration::from_secs(30), None);
    let no_escape = !r.stdout.contains("UNEXPECTED");
    let shim_denied = r.stdout.contains("OK: normal open failed");
    let detail = format!(
        "exit={:?} signal={:?} stdout={:?}",
        r.code, r.signal, r.stdout
    );
    conclude(
        "a_proc_mem",
        "no-net,ro-root,no-proc",
        no_escape && shim_denied,
        detail,
    );
}

#[test]
fn b_raw_socket_is_contained() {
    let r = run_guest("b_raw_socket", Duration::from_secs(30), None);
    // SIGSYS = 31: seccomp-BPF deny-by-default killed the int $0x80 bypass.
    let killed_by_seccomp = r.signal == Some(31);
    let no_escape = !r.stdout.contains("UNEXPECTED");
    let detail = format!(
        "exit={:?} signal={:?} stdout={:?}",
        r.code, r.signal, r.stdout
    );
    conclude(
        "b_raw_socket",
        "no-net,seccomp-deny-default",
        killed_by_seccomp && no_escape,
        detail,
    );
}

#[test]
fn c_shim_socket_is_contained() {
    // The fixture hangs on connect() — we kill it after 10s. A hang (timeout)
    // IS the containment signal: no host socket, no network.
    let r = run_guest("c_shim_socket", Duration::from_secs(10), None);
    let hung = r.timed_out;
    let attempted = r.stdout.contains("attempting connect");
    let no_return = !r.stdout.contains("connect returned");
    let detail = format!(
        "timed_out={} exit={:?} signal={:?} stdout={:?}",
        r.timed_out, r.code, r.signal, r.stdout
    );
    conclude(
        "c_shim_socket",
        "no-net,no-tun",
        hung && attempted && no_return,
        detail,
    );
}

#[test]
fn d_write_outside_is_contained() {
    let r = run_guest("d_write_outside", Duration::from_secs(30), None);
    let exited_ok = r.code == Some(0);
    let etc_denied = r.stdout.contains("OK: write to /etc/evil denied");
    let bin_denied = r.stdout.contains("OK: write to /bin/evil denied");
    // The guest's /tmp is virtual — nothing may appear on the host.
    let host_clean = !PathBuf::from("/tmp/evil").exists() && !PathBuf::from("/etc/evil").exists();
    let detail = format!(
        "exit={:?} stdout={:?} host_clean={host_clean}",
        r.code, r.stdout
    );
    conclude(
        "d_write_outside",
        "ro-root,scratch-tmp-only",
        exited_ok && etc_denied && bin_denied && host_clean,
        detail,
    );
}

#[test]
fn e_mem_exhaust_is_contained() {
    // RLIMIT_AS=512MB on the runner+guest tree: the guest's 100MB chunks
    // must fail well before affecting the host.
    let r = run_guest("e_mem_exhaust", Duration::from_secs(60), Some(512));
    let exited_ok = r.code == Some(0);
    let limit_hit = r.stdout.contains("OK: malloc failed");
    let no_runaway = !r.stdout.contains("UNEXPECTED");
    let detail = format!(
        "exit={:?} signal={:?} stdout_tail={:?}",
        r.code,
        r.signal,
        r.stdout.lines().last().unwrap_or("")
    );
    conclude(
        "e_mem_exhaust",
        "rlimit-as=512MB",
        exited_ok && limit_hit && no_runaway,
        detail,
    );
}
