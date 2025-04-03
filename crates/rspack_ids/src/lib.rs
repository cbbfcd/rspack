#![feature(iter_intersperse)]
#![feature(iter_order_by)]
#![feature(let_chains)]

mod deterministic_module_ids_plugin;
pub use deterministic_module_ids_plugin::*;
mod named_module_ids_plugin;
pub use named_module_ids_plugin::*;
pub mod id_helpers;
mod named_chunk_ids_plugin;
pub use named_chunk_ids_plugin::*;
mod deterministic_chunk_ids_plugin;
pub use deterministic_chunk_ids_plugin::DeterministicChunkIdsPlugin;
mod natural_module_ids_plugin;
pub use natural_module_ids_plugin::NaturalModuleIdsPlugin;
mod natural_chunk_ids_plugin;
pub use natural_chunk_ids_plugin::NaturalChunkIdsPlugin;
mod occurrence_chunk_ids_plugin;
pub use occurrence_chunk_ids_plugin::*;
