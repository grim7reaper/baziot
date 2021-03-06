//! Baziot — a Roaring bitmap implementation, with a more compact serialization

// Lints {{{

#![deny(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    rustdoc::all,
    rustdoc::missing_crate_level_docs,
    missing_docs,
    unreachable_pub,
    unsafe_code,
    unused,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    variant_size_differences,
    warnings,
    clippy::all,
    clippy::pedantic,
    clippy::clone_on_ref_ptr,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::lossy_float_literal,
    clippy::mem_forget,
    clippy::panic,
    clippy::pattern_type_mismatch,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::unneeded_field_pattern,
    clippy::verbose_file_reads,
    clippy::dbg_macro,
    clippy::let_underscore_must_use,
    clippy::print_stdout,
    clippy::todo,
    clippy::unwrap_used,
    clippy::use_debug
)]
#![allow(
    // The 90’s called and wanted their charset back :p
    clippy::non_ascii_literal,
    // Too noisy for little value.
    clippy::must_use_candidate,
    // False positive with criterion…
    unused_crate_dependencies,
)]

// }}}

mod chunk;
mod containers;
mod roaring;
mod roaring_lazy;
mod roaring_tree_map;
mod roaring_two_levels;
mod stats;

pub use roaring::Roaring;
pub use roaring_lazy::RoaringLazy;
pub use roaring_tree_map::RoaringTreeMap;
pub use roaring_two_levels::RoaringTwoLevels;
pub use stats::Stats;

use chunk::Chunk;
use containers::Container;
