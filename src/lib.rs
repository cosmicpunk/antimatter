pub mod contract;
pub mod error;
pub mod msg;
pub mod package;
pub mod state;

extern crate cosmwasm_std;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points!(contract);
