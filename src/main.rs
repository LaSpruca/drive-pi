mod app;
mod config;
mod device;
#[cfg(feature = "simulator")]
mod simulator;

use app::App;

#[cfg(feature = "simulator")]
const WIDTH: u32 = 128;
#[cfg(feature = "simulator")]
const HEIGHT: u32 = 64;
#[cfg(feature = "simulator")]
const SIZE: u32 = 1;

#[cfg(feature = "pi")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use embedded_graphics::prelude::*;
    use futures::stream::StreamExt;
    use gpio_cdev::{Chip, EventRequestFlags, LineRequestFlags};
    use linux_embedded_hal::I2cdev;
    use ssd1306::{
        prelude::*, rotation::DisplayRotation, size::DisplaySize128x64, I2CDisplayInterface,
        Ssd1306,
    };

    let mut i2c = I2cdev::new("/dev/i2c-1")?;

    i2c.set_slave_address(0x3C)?;

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();
    display.set_display_on(true).unwrap();
    display.set_brightness(Brightness::BRIGHTEST).unwrap();

    let mut chip = Chip::new("/dev/gpiochip0").unwrap();

    let mut a_events = chip.get_line(4)?.async_events(
        LineRequestFlags::INPUT,
        EventRequestFlags::RISING_EDGE,
        "drive-pi",
    )?;

    let mut b_events = chip.get_line(14)?.async_events(
        LineRequestFlags::INPUT,
        EventRequestFlags::RISING_EDGE,
        "drive-pi",
    )?;

    let mut c_events = chip.get_line(15)?.async_events(
        LineRequestFlags::INPUT,
        EventRequestFlags::RISING_EDGE,
        "drive-pi",
    )?;

    let mut d_events = chip.get_line(18)?.async_events(
        LineRequestFlags::INPUT,
        EventRequestFlags::RISING_EDGE,
        "drive-pi",
    )?;

    let mut app = App::default();

    app.load_config();

    app.draw(&mut display).unwrap();

    display.flush().unwrap();

    loop {
        tokio::select! {
            Some(Ok(_)) = a_events.next() => {
                app.handle_input("a");
                println!("a");
            }
            Some(Ok(_)) = b_events.next() => {
                app.handle_input("b");
                println!("b");
            }
            Some(Ok(_)) = c_events.next() => {
                app.handle_input("c");
                println!("c");
            }
            Some(Ok(_)) = d_events.next() => {
                app.handle_input("d");
                println!("d");
            }
        };

        if app.should_exit() {
            break;
        }

        display.clear();
        app.draw(&mut display).unwrap();
        display.flush().unwrap();
    }

    Ok(())
}

#[cfg(feature = "simulator")]
fn main() {
    use piston_window::{EventLoop, PistonWindow, Window, WindowSettings};
    use simulator::ScreenSimulator;

    let mut window: PistonWindow =
        WindowSettings::new("EG Simulator", [WIDTH * SIZE, HEIGHT * SIZE])
            .exit_on_esc(true)
            .resizable(false)
            //.opengl(OpenGL::V2_1) // Set a different OpenGl version
            .build()
            .unwrap();

    let mut display = ScreenSimulator::new();

    window.set_lazy(true);

    let mut app = App::default();
    app.load_config();

    while let Some(e) = window.next() {
        match &e {
            piston_window::Event::Input(i, _) => match i {
                piston_window::Input::Text(x) => {if x == "q" {window.set_should_close(true);} else {app.handle_input(x);}},
                _ => {}
            },
            _ => {}
            // piston_window::Event::Loop(_) => todo!(),
            // piston_window::Event::Custom(_, _, _) => todo!(),
        }

        display.clear(BinaryColor::Off).unwrap();

        app.draw(&mut display).unwrap();

        window.draw_2d(&e, |c, g, _| {
            display.draw(c, g);
        });

        if app.should_exit() {
            window.set_should_close(true)
        }
    }
}
