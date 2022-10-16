use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use linux_embedded_hal::I2cdev;
use ssd1306::{
    prelude::*, rotation::DisplayRotation, size::DisplaySize128x64, I2CDisplayInterface, Ssd1306,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut i2c = I2cdev::new("/dev/i2c-1")?;

    i2c.set_slave_address(0x3D)?;

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();
    display.set_display_on(true).unwrap();
    display.set_brightness(Brightness::BRIGHTEST).unwrap();

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    let im = Text::new("Hello world", Point { x: 0, y: 0 }, style);

    im.draw(&mut display).unwrap();

    display.flush().unwrap();

    loop {}
}
