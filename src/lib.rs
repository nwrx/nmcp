#![deny(warnings)]
#![allow(unstable_features)]
#![feature(backtrace_frames)]
#![warn(
    clippy::all,
    clippy::doc_markdown,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::use_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    unexpected_cfgs,
    clippy::await_holding_lock,
    clippy::indexing_slicing,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    nonstandard_style,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

pub mod cli;
pub mod controller;
pub mod gateway;
pub mod manager;
pub mod resources;
pub mod utils;

pub use cli::*;
pub use controller::*;
pub use gateway::*;
pub use manager::*;
pub use resources::*;
pub use utils::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub use tests::*;
