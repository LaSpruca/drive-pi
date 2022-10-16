use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use piston_window::{clear, rectangle, Context, G2d};

use crate::{HEIGHT, SIZE, WIDTH};

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

struct Px {
    x: u32,
    y: u32,
}

pub struct ScreenSimulator {
    px_buffer: Vec<Px>,
}

impl ScreenSimulator {
    pub fn new() -> Self {
        Self { px_buffer: vec![] }
    }

    pub fn draw(&self, c: Context, g: &mut G2d) {
        for Px { x, y } in self.px_buffer.iter() {
            rectangle(
                WHITE,
                [
                    (x * SIZE) as f64,
                    (y * SIZE) as f64,
                    SIZE as f64,
                    SIZE as f64,
                ],
                c.transform,
                g,
            );
        }

        clear([0.0, 0.0, 0.0, 0.0], g);
    }
}

impl OriginDimensions for ScreenSimulator {
    fn size(&self) -> Size {
        Size {
            height: HEIGHT,
            width: WIDTH,
        }
    }
}

impl DrawTarget for ScreenSimulator {
    type Color = BinaryColor;

    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        for Pixel(point, colour) in pixels {
            if point.x >= 0 && point.x < WIDTH as i32 && point.y >= 0 && point.y < HEIGHT as i32 {
                if colour.is_on() {
                    self.px_buffer.push(Px {
                        x: point.x as u32,
                        y: point.y as u32,
                    });
                }
            }
        }

        Ok(())
    }

    fn clear(&mut self, colour: Self::Color) -> Result<(), Self::Error> {
        self.px_buffer = Vec::with_capacity((WIDTH * HEIGHT) as usize);

        if colour.is_on() {
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    self.px_buffer.push(Px { x, y })
                }
            }
        }

        Ok(())
    }
}
