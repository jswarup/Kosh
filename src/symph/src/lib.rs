#![cfg_attr( target_arch = "spirv", no_std)]
#![deny( warnings)]
#![allow( unexpected_cfgs, non_snake_case, unused_imports)]

pub mod vertshade;
pub mod compshade;

pub use vertshade::*;
pub use compshade::*;
