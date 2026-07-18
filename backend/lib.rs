// ==============================================================================
// AllBright C2 Backend Library
// ==============================================================================
//
// The production binary (`main.rs`) is fully self-contained: it declares its own
// `mod allbright_c2` from the compiled protobufs and does not depend on this lib
// crate. The previous `pub use main::*;` was invalid Rust (a binary crate root
// cannot be imported as a module) and caused `cargo build`/`cargo check` to fail
// with `error[E0432]: unresolved import 'main'`, which also broke the Docker
// backend build.
//
// This crate is intentionally left empty. No target consumes it. If shared
// library code is needed in the future, factor the relevant modules out of
// `main.rs` into dedicated files and re-export them here.
