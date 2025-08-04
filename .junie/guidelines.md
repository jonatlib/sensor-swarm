# STM32 based sensor swarm peroject

## Rust

We are using modern Rust where mod.rs is not needed and you can directly write same filename and same directory name as sub-module.

## Project standards

We are writing documented and tested code:
  - All methods, structs, modules have a documentation string
  - If possible we are writing in-line tests (using `#[cfg(tests)]`). Beware we are using defmt-tests but the definition macro is only needed once in a project and it is defined in `lib.rs`.
  - All in-line tests must be HW agnostic so we can run them in QEMU (this is how `cargo test` is configured), so only test code which don't need actuall HW.
  - We can also write HIL (HW in loop) test, but need to use feature flags for these.

We are keeping modules small and separated by application/business logic into submodules:
  - One module should do a one thing
  - We can combine multiple modules and struct for complex functionality (don't forget about modern Rust modules).

We are not re-implementing wheel,
so if there is existing crate we rather use that instead of writing our own implementations.

We are using Embassy framework.
