# LiteBox

> A security-focused library OS

> [!NOTE]  
> This project is currently actively evolving and improving. While we are
> working toward a stable release, some APIs and interfaces may change as the
> design continues to mature. You are welcome to explore and experiment, but if
> you need long-term stability, it may be best to wait for a stable release, or
> be prepared to adapt to updates along the way.

LiteBox is a sandboxing library OS that drastically cuts down the interface to the host, thereby reducing attack surface.  It focuses on easy interop of various "North" shims and "South" platforms.  LiteBox is designed for usage in both kernel and non-kernel scenarios.

LiteBox exposes a Rust-y [`nix`](https://docs.rs/nix)/[`rustix`](https://docs.rs/rustix)-inspired "North" interface when it is provided a `Platform` interface at its "South".  These interfaces allow for a wide variety of use-cases, easily allowing for connection between any of the North--South pairs.

Example use cases include:
- Running unmodified Linux programs on Windows
- Sandboxing Linux applications on Linux
- Run programs on top of SEV SNP
- Running OP-TEE programs on Linux
- Running on LVBS

![LiteBox and related projects](./.figures/litebox.svg)

## Hostile-guest containment suite

`hostile/` holds five hostile ELF fixtures (with C sources) that attack the
guest/host boundary: procfs escape, raw-syscall network bypass, shim socket
abuse, filesystem escape, and memory exhaustion. `hostile-tests/` is a Rust
integration suite that launches each fixture as a LiteBox guest via the Linux
userland runner, asserts **containment** (not just "it ran"), and writes a JSON
receipt per fixture to `hostile-tests/receipts/<fixture>.json`:

```json
{"fixture":"a_proc_mem","sha256":"…","policy":"no-net,ro-root","verdict":"contained","ts":"…"}
```

Run it:

```sh
./hostile/build.sh          # compile the five fixtures (gcc -static)
cargo test -p hostile-tests # 5/5 containment assertions + receipts
```

Each test sha256-pins its fixture binary before running. A fixture the current
LiteBox cannot contain gets verdict `"gap"` — a failing containment is a
finding, not a test bug. This suite is test-only; it does not modify LiteBox core.

## Contributing

See the following files for details:

- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md)
- [SECURITY.md](./SECURITY.md)
- [SUPPORT.md](./SUPPORT.md)

## License

MIT License.  See [./LICENSE](./LICENSE) for details.

## Trademarks

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft 
trademarks or logos is subject to and must follow 
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
