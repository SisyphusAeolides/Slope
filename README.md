# Slope

Slope is the capability-oriented userspace ABI and runtime library for
ArachOS. It defines bounded syscall, process, display, input, Wayland, service,
and COSMIC session contracts without assuming the host kernel's syscall table.

Slope is maintained as an independent repository. Its Arach Kernel ABI crates
are pinned to an immutable kernel revision; consuming repositories should pin
Slope the same way so an ABI update is explicit and reviewable.

## Languages and assurance

- Rust implements the `no_std` runtime and typed ABI.
- Fortran implements an optional bounded route scorer behind
  `fortran-policy`.
- Idris 2 makes route selection total and prevents a denied route from
  producing an admission witness.
- Agda proves that denied routes cannot carry capabilities or enter service
  and COSMIC-session states.

## Validation

```sh
cargo fmt --all -- --check
cargo test --all-features
scripts/check-formal-models.sh
```

## Current ArachOS integration status

This project is maintained as part of the ArachOS production graph. Its role is
the bounded userspace ABI shared by the ArachOS service graph..

CI and release evidence are evaluated on immutable revisions. Hardware support
is reported by bounded route and support level; this README does not claim
universal native support. Gate 3 requires signed hardware identity, target
kernel provenance, package authority, health checks, rollback behavior, and
representative physical-hardware evidence before production qualification.
