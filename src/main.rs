mod app;
mod config;
mod device;
#[cfg(feature = "simulator")]
mod simulator;

use std::fs;

use app::App;
use config::Config;
use device::get_devices;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*, Drawable};
use walkdir::WalkDir;

const WIDTH: u32 = 128;
const HEIGHT: u32 = 64;
const SIZE: u32 = 1;

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

    clean_up();
}

fn clean_up() {
    let config = Config::load().unwrap_or(Config::default());
    match get_devices(&config.mount_path) {
        Ok(devices) => {
            for device in devices {
                if device.mounted {
                    match device.unmount() {
                        Ok(_) => {}
                        Err(ex) => {
                            eprintln!("{ex}")
                        }
                    };
                }
            }
        }
        Err(_) => {}
    };

    let subpaths = WalkDir::new(&config.mount_path);

    for sub in subpaths.into_iter().filter_map(|z| z.ok()) {
        if sub.path().is_dir() && sub.path() == config.mount_path {
            match fs::remove_dir(sub.path()) {
                Ok(_) => {}
                Err(ex) => {
                    eprintln!("Could not delete director {} {}", sub.path().display(), ex)
                }
            };
        }
    }
}
