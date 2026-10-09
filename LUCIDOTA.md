# LUCIDOTA on LiteBox

We fork LiteBox as the **execution substrate for organelles** — sandboxed, attestable compute units.

## Why we forked

LiteBox is the closest thing to receipt-grade sandboxing with confidential computing
(SEV-SNP) as a first-class target. Our doctrine: **use LiteBox, never be its slave —
the governor stays above it.** We write to the North interface; the governor picks the South.

## What we're building

- **Hostile-guest test corpus** — real ELF fixtures that attack the guest/host boundary.
  Sandboxes are only as good as their adversaries.
- **Organelle host** — running organelles as LiteBox guests, each execution producing a receipt.
- **Hardware-attested execution** — proving *where* code ran, not just *what* it did.
  That's the missing half of the provenance story.

## Branches

- `main` — pure upstream mirror, auto-synced daily. Don't touch.
- `lucidota` — our direction. You're here.

## Road to total conversion

A LiteBox distribution where every guest boot produces a proof-ledger receipt and the
governor — not the host — is the trust root.
