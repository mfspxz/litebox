# Hostile fixtures

Five hostile ELF fixtures (with C sources) that attack the LiteBox guest/host
boundary. Each documents a known hostile behavior; the containment suite in
`../hostile-tests/` proves LiteBox contains them and emits a JSON receipt per run.

## a_proc_mem — procfs escape

Tries to read host `/proc/self/mem`. First via normal `open()` (goes through the
LiteBox shim, which has no `/proc` — expect `ENOENT`). Then via a raw syscall
generated at runtime (`mmap`'d `mov`/`syscall` bytes), so the static syscall
rewriter cannot see it — the outer mount namespace (which has no `/proc`) must
deny it. Containment looks like: both attempts fail, guest exits 0 having
achieved nothing.

## b_raw_socket — syscall-bypass network escape

Tries to open a raw socket via `int $0x80` (the 32-bit compat syscall path)
instead of `syscall` (`0F 05`). The static rewriter targets `syscall`; if it
misses `int $0x80`, the host kernel sees the call and the outer seccomp-BPF
deny-by-default filter must kill the process with `SIGSYS`. Containment looks
like: the guest dies by signal (it never reaches its "UNEXPECTED" print).

## c_shim_socket — shim socket layer attack

Calls `socket()` normally via libc, which goes through the LiteBox shim. The
shim has no TUN device backing, so `connect()` to the outside world must fail
or hang (watchdog timeout) — no host socket may ever be created. Containment
looks like: `socket()` denied or `connect()` fails; the guest never reaches
the network.

## d_write_outside — filesystem escape

Tries to write outside its allowed root: `/etc/evil` must fail (not in the
virtual FS), `/bin/evil` must fail (read-only), while `/tmp/evil` may succeed
(writable scratch). Containment looks like: no file appears outside the scratch
dir on the host after the run.

## e_mem_exhaust — resource exhaustion

Tries to allocate 10GB in 100MB chunks, touching every page. `RLIMIT_AS` (or the
platform memory limit) must make `malloc`/`mmap` fail; the guest must be
stopped before it affects the host. Containment looks like: allocation fails at
the limit, guest exits cleanly, host memory is unaffected.
