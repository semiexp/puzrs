extern crate rand;

mod common;

#[cfg(not(target_arch = "wasm32"))]
pub mod endview;
#[cfg(not(target_arch = "wasm32"))]
pub mod grid_loop;
#[cfg(not(target_arch = "wasm32"))]
pub mod kakuro;
#[cfg(not(target_arch = "wasm32"))]
pub mod numberlink;
#[cfg(not(target_arch = "wasm32"))]
pub mod slitherlink;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
#[cfg(not(target_arch = "wasm32"))]
pub mod io;

#[cfg(target_arch = "wasm32")]
mod endview;
#[cfg(target_arch = "wasm32")]
mod grid_loop;
#[cfg(target_arch = "wasm32")]
mod kakuro;
#[cfg(target_arch = "wasm32")]
mod numberlink;
#[cfg(target_arch = "wasm32")]
mod slitherlink;

#[cfg(not(target_arch = "wasm32"))]
pub use common::*;

#[cfg(target_arch = "wasm32")]
use common::*;

#[cfg(target_arch = "wasm32")]
mod js;

#[cfg(target_arch = "wasm32")]
pub use js::*;
