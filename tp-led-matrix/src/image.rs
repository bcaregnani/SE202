use crate::gamma::gamma_correct;
use core::mem::transmute;
use core::ops::{Div, Index, IndexMut, Mul};
use micromath::F32Ext;

/// Structure Color containing three unsigned bytes named after the
/// primary colors used in the led matrix: r, g and b.
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub const RED: Color = Color { r: 255, g: 0, b: 0 };
pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        let new_r = f32_to_u8((self.r as f32) * rhs);
        let new_g = f32_to_u8((self.g as f32) * rhs);
        let new_b = f32_to_u8((self.b as f32) * rhs);
        Self {
            r: new_r,
            g: new_g,
            b: new_b,
        }
    }
}

impl Div<f32> for Color {
    type Output = Color;

    // Required method
    fn div(self, rhs: f32) -> Self::Output {
        let new_rhs = (1_f32) / rhs;
        self.mul(new_rhs)
    }
}

impl Color {
    pub fn gamma_correct(&self) -> Self {
        Self {
            // Added 2* to make intensity stronger
            r: 2 * gamma_correct(self.r),
            g: 2 * gamma_correct(self.g),
            b: 2 * gamma_correct(self.b),
        }
    }
}

fn f32_to_u8(number: f32) -> u8 {
    let number_trunc = F32Ext::trunc(number);
    match number_trunc {
        x if x > 255_f32 => 255,
        x if x < 0_f32 => 0,
        _ => number_trunc as u8,
    }
}

/// Structure Image containing a unique unnamed field consisting of an array of 64 Color.
#[repr(transparent)]
pub struct Image([Color; 64]);

impl Default for Image {
    fn default() -> Self {
        Image::new_solid(Default::default())
    }
}

impl Index<(usize, usize)> for Image {
    type Output = Color;

    // Required method
    /// Indexing for Image is bounded between 0 to 7 for both index.
    /// The row is indicated in index 0.
    /// The column is indicated in index 1.
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let row = index.0;
        let column = index.1;
        if column > 7 {
            panic!()
        };
        if row > 7 {
            panic!()
        } else {
            &self.0[8 * row + column]
        }
    }
}

impl IndexMut<(usize, usize)> for Image {
    /// Indexing for Image is bounded between 0 to 7 for both index
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let row = index.0;
        let column = index.1;
        if column > 7 {
            panic!()
        };
        if row > 7 {
            panic!()
        } else {
            &mut self.0[8 * row + column]
        }
    }
}

impl AsRef<[u8; 192]> for Image {
    fn as_ref(&self) -> &[u8; 192] {
        unsafe { transmute::<&Image, &[u8; 192]>(self) }
    }
}

impl AsMut<[u8; 192]> for Image {
    fn as_mut(&mut self) -> &mut [u8; 192] {
        unsafe { transmute::<&mut Image, &mut [u8; 192]>(self) }
    }
}

impl Image {
    /// Public function which returns an image filled with the color given as an argument.
    pub fn new_solid(color: Color) -> Self {
        Image([color; 64])
    }

    /// Public function referencing the content of one particular row.
    pub fn row(&self, row: usize) -> &[Color] {
        match row {
            0 => &self.0[0..8],
            1 => &self.0[8..16],
            2 => &self.0[16..24],
            3 => &self.0[24..32],
            4 => &self.0[32..40],
            5 => &self.0[40..48],
            6 => &self.0[48..56],
            7 => &self.0[56..64],
            _ => panic!(),
        }
    }

    /// Function returning an image containing a gradient from a given color to black.
    pub fn gradient(color: Color) -> Self {
        let mut n_image = Image::default();
        for row in 0..8 {
            for column in 0..8 {
                n_image[(row, column)] = color.div((1 + row * row + column) as f32);
            }
        }
        n_image
    }
}
