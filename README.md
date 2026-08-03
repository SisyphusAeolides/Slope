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
