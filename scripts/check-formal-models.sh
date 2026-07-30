#!/usr/bin/env bash
set -euo pipefail

root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
idris="${IDRIS2:-idris2}"
agda="${AGDA:-agda}"

grep -Fxq '%default total' "$root/formal/idris2/SlopeRoute.idr"
grep -Fxq '{-# OPTIONS --safe --without-K #-}' "$root/formal/agda/SlopeCapability.agda"

if grep -En 'believe_me|assert_total|assert_smaller|unsafe|(^|[^[:alnum:]_])partial([^[:alnum:]_]|$)|[?][A-Za-z_]|[?][?][?]' \
    "$root/formal/idris2/SlopeRoute.idr"; then
    exit 1
fi
if grep -En '^[[:space:]]*postulate\b|\{![^!]*!\}|TERMINATING|NON_TERMINATING|NO_TERMINATION_CHECK' \
    "$root/formal/agda/SlopeCapability.agda"; then
    exit 1
fi

scratch="$(mktemp -d "${TMPDIR:-/tmp}/slope-formal.XXXXXXXX")"
trap 'rm -rf -- "$scratch"' EXIT
cp "$root/formal/idris2/SlopeRoute.idr" "$scratch/"
cp "$root/formal/agda/SlopeCapability.agda" "$scratch/"
(
    cd "$scratch"
    "$idris" --check SlopeRoute.idr
    XDG_DATA_HOME="$scratch/data" XDG_CONFIG_HOME="$scratch/config" \
        "$agda" --no-libraries --safe --without-K SlopeCapability.agda
)
