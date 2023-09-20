use crate::Color;
use crate::Image;
use core::convert::Infallible;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::Rgb888, prelude::*};

impl From<Rgb888> for Color {
    fn from(color: Rgb888) -> Self {
        Self {
            r: color.r(),
            g: color.g(),
            b: color.b(),
        }
    }
}

impl DrawTarget for Image {
    type Error = Infallible;
    type Color = Rgb888;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            // Check if the pixel coordinates are out of bounds (negative or greater than
            // (7,7)). `DrawTarget` implementation are required to discard any out of bounds
            // pixels without returning an error or causing a panic.
            if let Ok((x @ 0..=7, y @ 0..=7)) = coord.try_into() {
                self[(y as usize, x as usize)] = color.into();
            }
        }
        Ok(())
    }
}

impl OriginDimensions for Image {
    fn size(&self) -> Size {
        Size::new(8, 8)
    }
}
