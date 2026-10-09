// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! Hostile-guest containment suite.
//!
//! Five hostile ELF fixtures attack the LiteBox guest/host boundary. Each
//! integration test launches its fixture as a LiteBox guest via the Linux
//! userland runner, asserts containment (not just "it ran"), and writes a
//! JSON receipt to `receipts/<fixture>.json`.
//!
//! A fixture the current LiteBox cannot contain gets verdict `"gap"` — a
//! failing containment is a finding, not a test bug.

use std::path::{Path, PathBuf};

/// Pinned sha256 of each fixture binary (from `hostile/build.sh`).
/// If you rebuild the fixtures, update these pins.
pub const FIXTURES: &[(&str, &str)] = &[
    (
        "a_proc_mem",
        "17958b4e748037a71819f1e309baac8afcea987dfc605c89854a4e5de8839c1c",
    ),
    (
        "b_raw_socket",
        "206d414a7386406ca6dffe644cf94c619afc9dc8042e61908b257afd2be1fcc0",
    ),
    (
        "c_shim_socket",
        "fae87fbe8390d32b81d37f2494018203cee0fc4dcd4399a4c95a17aacccc30ff",
    ),
    (
        "d_write_outside",
        "1727a42112e509d623ba145dce72ccf9e35dfb17ee04aa92561dbecbefa3e99b",
    ),
    (
        "e_mem_exhaust",
        "6f529110c289eb1f4f8c37489bf05fea682c671cf0a16d9110e1405a6b1eb0d7",
    ),
];

/// Repo root (parent of this crate dir).
///
/// # Panics
///
/// Panics if the crate isn't inside the litebox repo root.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("hostile-tests must live in the litebox repo root")
        .to_path_buf()
}

/// Path to a fixture binary, verifying its sha256 pin first.
///
/// # Panics
///
/// Panics if the fixture is unknown, unreadable, or its hash doesn't match the pin.
pub fn fixture_path(name: &str) -> PathBuf {
    let pin = FIXTURES
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("unknown fixture {name}"))
        .1;
    let path = repo_root().join("hostile/bin").join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("cannot read fixture {}: {e}", path.display()));
    let actual = hex_digest(&bytes);
    assert!(
        actual == pin,
        "fixture {name} sha256 mismatch: got {actual}, pinned {pin} — rebuild via hostile/build.sh and update pins"
    );
    path
}

/// Path to the built litebox_runner_linux_userland binary, building it on
/// demand if missing.
///
/// # Panics
///
/// Panics if the runner can't be built or found.
pub fn runner_bin() -> PathBuf {
    let p = PathBuf::from(env!("HOSTILE_RUNNER_BIN"));
    if p.is_file() {
        return p;
    }
    // Build it now — tests run after the main cargo build, so no lock contention.
    let cargo = option_env!("CARGO").unwrap_or("cargo");
    let status = std::process::Command::new(cargo)
        .args(["build", "-p", "litebox_runner_linux_userland"])
        .current_dir(repo_root())
        .status()
        .expect("failed to spawn cargo build for the runner");
    assert!(
        status.success() && p.is_file(),
        "runner binary not found at {} — build it with: cargo build -p litebox_runner_linux_userland",
        p.display()
    );
    p
}

use std::fmt::Write as _;
fn hex_digest(bytes: &[u8]) -> String {
    use sha2::{Digest as _, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    let mut s = String::with_capacity(64);
    for b in out {
        write!(s, "{b:02x}").unwrap();
    }
    s
}
