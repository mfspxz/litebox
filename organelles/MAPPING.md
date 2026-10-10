# litebox — organelle mapping decisions

**Upstream fork. Tier C DECLARED ONLY. We do not re-architect upstream code.**

This is the LiteBox security-focused library OS / process sandbox
(workspace crates: `litebox`, `litebox_common*`, `litebox_platform_*`,
`litebox_runner_*`, `litebox_shim_*`, `litebox_packager`, `litebox_service_heki`).
It is upstream code we fork and build on — not our computation to
decompose into organelles.

## Decision

- **Tier C declared**: `organelles/litebox.manifest.json` (codon `lx`,
  valence **Candidate**, no registration, no code changes). Committed as
  `fa8d38a`.
- Two declared ops: `describe` (sandbox capabilities) and `sandbox_exec`
  (run a guest under the sandbox; ExternalEffect, capability
  `sandbox-exec`). These describe the sandbox's *interface*, not an
  organelle we operate — no host registration is attempted.
- **No map-or-design**: the 12 reference organelles are our substrate,
  not a lens for upstream code. LiteBox is the sandbox the organelles
  *run inside* (LiteBox spec §19/§23 is literally the migration
  standard's source) — mapping it onto its own substrate would be
  circular.

## Reference-organelle check

Deliberately not applied. LiteBox is infrastructure, not a capability
unit of ours. (If a future wave builds a `litebox-runner` organelle that
*uses* the sandbox to execute guests, that organelle gets its own
map-or-design then — against the `sandbox_exec` interface declared
here.)

## Notes

- The `describe` op declares no per-op `mutation_class`, so it inherits
  the authority-level `external_effect` + `sandbox-exec` gate. That is
  over-gating for a read-only description, but the manifest validates
  (effective capability inherits) and Tier C never registers — recorded
  here, not "fixed", because we do not re-architect upstream declarations
  either. A future `litebox-runner` organelle should declare `describe`
  as ReadOnly.
- Valence stays Candidate permanently for this declaration: we assert
  nothing about upstream's behavior.
- Codon `lx`: checked unique migration-wide.
