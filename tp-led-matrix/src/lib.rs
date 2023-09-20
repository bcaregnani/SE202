#![no_std]

pub use image::*;
pub mod image;

pub use gamma::gamma_correct;
pub mod gamma;

pub use matrix::Matrix;
pub mod matrix;

pub use embedded::*;
pub mod embedded;
