#!/bin/bash
# Build the five hostile fixtures as static binaries.
# Outputs to hostile/bin/. Requires gcc.
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
OUT="$DIR/bin"
mkdir -p "$OUT"
for f in a_proc_mem b_raw_socket c_shim_socket d_write_outside e_mem_exhaust; do
    echo "building $f..."
    gcc -static -O2 -o "$OUT/$f" "$DIR/$f.c"
    chmod 755 "$OUT/$f"
done
echo "done. binaries in $OUT:"
ls -la "$OUT"
echo "--- sha256 ---"
sha256sum "$OUT"/*
