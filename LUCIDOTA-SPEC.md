# SPEC: LiteBox Hostile-Guest Containment Suite

**Fork:** mfspxz/litebox · **Branch:** lucidota · **Budget:** 50 iterations

## Goal

Prove LiteBox containment against five known hostile behaviors. Every test run emits
a JSON receipt. This is the empirical foundation for "receipt-grade sandboxing."

## Non-goals

- Modifying LiteBox core. We test it, we don't fork its internals (yet).
- Windows/LVBS/OP-TEE targets. Linux userland runner only for v1.

## Background

We have five hostile ELF fixtures (with C sources) from our organelle-host work:
- `a_proc_mem` — tries to read host `/proc/mem` (guest escape via procfs)
- `b_raw_socket` — tries to open a raw socket (network escape)
- `c_shim_socket` — attacks the shim socket layer
- `d_write_outside` — tries to write outside its root filesystem
- `e_mem_exhaust` — tries to exhaust host memory (resource exhaustion)

Each has a `.c` source documenting the attack.

## Deliverables

### 1. `hostile/` — fixture sources and build

- `a_proc_mem.c`, `b_raw_socket.c`, `c_shim_socket.c`, `d_write_outside.c`, `e_mem_exhaust.c`
- `build.sh` — compiles all five to static binaries (`gcc -static`), outputs to `hostile/bin/`
- `HOSTILE.md` — one paragraph per fixture: what it attacks, what containment looks like

### 2. `hostile-tests/` — Rust containment suite

A Rust crate (member of the workspace or standalone) with one integration test per fixture.
Each test:

1. Loads the fixture binary (sha256-pinned — the test asserts the hash before running)
2. Launches it as a LiteBox guest via the Linux userland runner with a restrictive policy:
   - No network access
   - Read-only rootfs except a scratch dir
   - No `/proc` beyond what the policy allows
   - Memory limit enforced
3. Asserts containment:
   - `a_proc_mem`: guest exits non-zero / gets denied; host `/proc/mem` unread
   - `b_raw_socket`: socket creation fails inside the guest
   - `c_shim_socket`: shim layer holds; no host socket created
   - `d_write_outside`: no file appears outside the scratch dir
   - `e_mem_exhaust`: guest is killed by the memory limit; host unaffected
4. Writes `receipts/<fixture>.json`:
   ```json
   {"fixture":"a_proc_mem","sha256":"…","policy":"no-net,ro-root","verdict":"contained","ts":"…"}
   ```

### 3. Wire-up

- `cargo test -p hostile-tests` runs the suite.
- README section explaining how to run it and what the receipts mean.

## Acceptance criteria

- [ ] `hostile/build.sh` produces 5 binaries
- [ ] `cargo test -p hostile-tests` — 5/5 pass (each asserting containment, not just "it ran")
- [ ] 5 receipt JSON files, schema-valid
- [ ] No modification to LiteBox core crates (test-only addition)

## Constraints

- Linux only. If a fixture can't be contained by the current LiteBox, the test documents
  the gap (verdict: `gap`) — a failing containment is a finding, not a test bug.
- Never `reset`/`force-push`. Commit on the `lucidota` branch only.
