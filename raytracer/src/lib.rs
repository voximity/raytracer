#![allow(dead_code)]
#![allow(clippy::many_single_char_names)]
#![feature(new_uninit)]

pub mod acceleration;
pub mod camera;
pub mod lighting;
pub mod material;
pub mod math;
pub mod object;
pub mod scene;
pub mod skybox;

#[cfg(feature = "lua")]
pub mod lua;
