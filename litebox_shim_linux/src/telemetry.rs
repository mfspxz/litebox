// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//! Minimal syscall/file telemetry for the organelle-host pilot.
//!
//! ADDITIVE PATCH (not upstream): records every guest syscall dispatch and
//! every file open into in-memory logs that the host can drain after the
//! guest exits. No behavior change when the logs are never drained.
//!
//! no_std + alloc compatible. Uses spin::Mutex (already a dependency).

use alloc::string::String;
use alloc::vec::Vec;

/// One guest syscall dispatch: raw x86-64 syscall number + whether the shim
/// handled it successfully.
#[derive(Debug, Clone, Copy)]
pub struct SyscallEvent {
    pub nr: u32,
    pub ok: bool,
}

/// One guest file open: resolved guest path + whether opened for write +
/// whether the open succeeded.
#[derive(Debug, Clone)]
pub struct FileEvent {
    pub path: String,
    pub write: bool,
    pub ok: bool,
}

static SYSCALL_LOG: spin::Mutex<Vec<SyscallEvent>> = spin::Mutex::new(Vec::new());
static FILE_LOG: spin::Mutex<Vec<FileEvent>> = spin::Mutex::new(Vec::new());

/// Record a syscall dispatch. Called from `handle_syscall_request`.
pub fn record_syscall(nr: u32, ok: bool) {
    SYSCALL_LOG.lock().push(SyscallEvent { nr, ok });
}

/// Record a file open. Called from `do_open`.
pub fn record_file_open(path: &str, write: bool, ok: bool) {
    // Bound the log so a malicious guest can't OOM the host via opens.
    let mut log = FILE_LOG.lock();
    if log.len() < 4096 {
        log.push(FileEvent {
            path: String::from(path),
            write,
            ok,
        });
    }
}

/// Drain both logs. Intended to be called once, after the guest exits.
pub fn take() -> (Vec<SyscallEvent>, Vec<FileEvent>) {
    let syscalls = core::mem::take(&mut *SYSCALL_LOG.lock());
    let files = core::mem::take(&mut *FILE_LOG.lock());
    (syscalls, files)
}
