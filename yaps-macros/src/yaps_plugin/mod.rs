// We allow this for defs.rs usage
#![allow(clippy::borrow_interior_mutable_const)]

mod yaps_consumer;
mod yaps_extern;
mod yaps_extern_arg;

mod yaps_provider;
mod yaps_export;

mod defs;
mod yaps_plugin_macro;

pub(crate) use yaps_plugin_macro::process_yaps_plugin;
