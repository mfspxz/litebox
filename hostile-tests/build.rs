// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! Locate the litebox_runner_linux_userland binary for integration tests.
//! Emits HOSTILE_RUNNER_BIN. The binary may not exist yet at build time —
//! the test harness builds it on demand (see `ensure_runner`).

fn main() {
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest.parent().unwrap().to_path_buf();
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map_or_else(|_| repo_root.join("target"), std::path::PathBuf::from);
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let bin = target_dir
        .join(profile)
        .join("litebox_runner_linux_userland");
    println!("cargo:rustc-env=HOSTILE_RUNNER_BIN={}", bin.display());
    // Re-run if the runner binary changes.
    println!("cargo:rerun-if-changed={}", bin.display());
    println!("cargo:rerun-if-env-changed=CARGO_TARGET_DIR");
}
