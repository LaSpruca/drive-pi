use std::fs;

use embedded_graphics::{
    mono_font::{
        iso_8859_10::FONT_6X10, iso_8859_15::FONT_4X6, iso_8859_16::FONT_5X8,
        iso_8859_7::FONT_7X13_BOLD, MonoTextStyle,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::*,
    text::{Alignment, Text},
};
use walkdir::WalkDir;

use crate::{
    config::Config,
    device::{get_devices, Device},
};

#[derive(Default)]
pub struct App {
    screen: Screen,
    should_quit: bool,
    config: Config,
}

impl App {
    pub fn should_exit(&self) -> bool {
        self.should_quit
    }

    pub fn handle_input(&mut self, input: &str) {
        match &self.screen {
            Screen::Home => match input {
                // WiFi
                "a" | "7" => {
                    self.screen = Screen::Error("WiFi not implemented".to_string());
                }
                // Mnt
                "c" | "9" => {
                    self.devices(None);
                }
                // SMB
                "b" | "1" => {
                    self.screen = Screen::Error("Samba password\nnot implemented".to_string());
                }
                // Reboot
                "d" | "3" => {
                    self.screen = Screen::ConfirmExit;
                }
                _ => {}
            },
            Screen::Devices(devices, idex) => match input {
                "a" | "7" => {
                    if *idex == 0 {
                        self.screen = Screen::Devices(devices.clone(), devices.len() - 1)
                    } else {
                        self.screen = Screen::Devices(devices.clone(), idex - 1)
                    }
                }
                "c" | "9" => {
                    let dev = &devices[*idex];
                    if dev.mounted {
                        match dev.unmount() {
                            Ok(_) => {
                                match fs::remove_dir(&dev.path) {
                                    _ => {}
                                };
                                self.devices(Some(*idex));
                            }
                            Err(ex) => {
                                eprintln!("Could not unmount {ex:?}");
                                self.screen =
                                    Screen::Error(format!("Could not unmount {}", dev.name));
                            }
                        }

                        self.devices(None);
                    } else {
                        match dev.mount() {
                            Ok(_) => self.devices(Some(*idex)),
                            Err(ex) => {
                                eprintln!("{ex:?}");
                                self.screen =
                                    Screen::Error(format!("Could not mount {}", dev.name));
                            }
                        }
                    }
                }
                "b" | "1" => {
                    if *idex == devices.len() - 1 {
                        self.screen = Screen::Devices(devices.clone(), 0)
                    } else {
                        self.screen = Screen::Devices(devices.clone(), idex + 1)
                    }
                }
                "d" | "3" => self.screen = Screen::Home,
                _ => {}
            },
            Screen::Error(_) => match input {
                "a" | "7" => self.screen = Screen::Home,
                _ => {}
            },
            Screen::ConfirmExit => match input {
                "a" | "7" => self.should_quit = true,
                "c" | "9" => self.screen = Screen::Home,
                _ => {}
            },
        }
    }

    pub fn load_config(&mut self) {
        if let Some(config) = Config::load() {
            self.config = config;
        }
    }

    fn devices(&mut self, index: Option<usize>) {
        let devices = match get_devices(&self.config.mount_path) {
            Ok(val) => val,
            Err(ex) => {
                eprintln!("{ex:?}");
                self.screen = Screen::Error("Could not get devices".to_string());
                return;
            }
        };

        self.screen = Screen::Devices(devices, index.unwrap_or(0));
    }
}

impl Drop for App {
    fn drop(&mut self) {
        match get_devices(&self.config.mount_path) {
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

        let subpaths = WalkDir::new(&self.config.mount_path);

        for sub in subpaths.into_iter().filter_map(|z| z.ok()) {
            if sub.path().is_dir() && sub.path() == self.config.mount_path {
                match fs::remove_dir(sub.path()) {
                    Ok(_) => {}
                    Err(ex) => {
                        eprintln!("Could not delete director {} {}", sub.path().display(), ex)
                    }
                };
            }
        }
    }
}

impl<'a> Drawable for App {
    type Color = BinaryColor;

    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.screen.draw(target)
    }
}

enum Screen {
    Home,
    Devices(Vec<Device>, usize),
    Error(String),
    ConfirmExit,
}

impl Default for Screen {
    fn default() -> Self {
        Self::Home
    }
}

impl Screen {
    fn opts(&self) -> [&'static str; 4] {
        match self {
            Screen::Home => ["WIFI", "MNT", "SMB", "EXIT"],
            Screen::Devices(drives, index) => {
                if drives.is_empty() {
                    ["", "", "", "BACK"]
                } else if drives[*index].mounted {
                    ["^", "UMT", "v", "BACK"]
                } else {
                    ["^", "MNT", "v", "BACK"]
                }
            }
            Screen::Error(_) => ["BACK", "", "", ""],
            Screen::ConfirmExit => ["YES", "NO", "", ""],
        }
    }
}

impl Drawable for Screen {
    type Color = BinaryColor;

    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        boxes(target, self.opts())?;
        match self {
            Screen::Home => Ok(()),
            Screen::Devices(d, hovered) => devices(target, d, *hovered),
            Screen::Error(msg) => error(target, msg),
            Screen::ConfirmExit => confirm_exit(target),
        }
    }
}

fn confirm_exit<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    Text::with_alignment(
        "Are you sure\nyou want to exit",
        display.bounding_box().center(),
        MonoTextStyle::new(&FONT_6X10, BinaryColor::On),
        Alignment::Center,
    )
    .draw(display)?;

    Ok(())
}

fn error<D>(display: &mut D, msg: &String) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let Point { x, .. } = display.bounding_box().center();
    Text::with_alignment(
        "Error",
        Point { x, y: 20 },
        MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On),
        Alignment::Center,
    )
    .draw(display)?;

    Text::with_alignment(
        msg,
        Point { x, y: 40 },
        MonoTextStyle::new(&FONT_5X8, BinaryColor::On),
        Alignment::Center,
    )
    .draw(display)?;

    Ok(())
}

pub fn devices<D>(display: &mut D, devices: &Vec<Device>, hovered: usize) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let style = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);

    Text::new("NAME", Point { x: 5, y: 18 }, style).draw(display)?;
    Text::with_alignment("SIZE", Point { x: 70, y: 18 }, style, Alignment::Center).draw(display)?;
    Text::new("MOUNTED", Point { x: 92, y: 18 }, style).draw(display)?;

    if devices.is_empty() {
        Text::with_alignment(
            "NO DEVICES",
            display.bounding_box().center(),
            style,
            Alignment::Center,
        )
        .draw(display)?;
    } else {
        let top_4: Vec<Device> = devices
            .iter()
            .skip(hovered)
            .take(3)
            .map(|x| x.to_owned())
            .collect();

        Text::new(
            ">",
            Point { x: 1, y: 30 },
            MonoTextStyle::new(&FONT_4X6, BinaryColor::On),
        )
        .draw(display)?;

        for (
            index,
            Device {
                name,
                size,
                mounted,
                ..
            },
        ) in top_4.iter().enumerate()
        {
            Text::new(
                name,
                Point {
                    x: 5,
                    y: 10 * index as i32 + 30,
                },
                style,
            )
            .draw(display)?;

            Text::with_alignment(
                size,
                Point {
                    x: 70,
                    y: 10 * index as i32 + 30,
                },
                style,
                Alignment::Center,
            )
            .draw(display)?;

            if *mounted {
                Text::with_alignment(
                    "*",
                    Point {
                        x: 112,
                        y: 10 * index as i32 + 30,
                    },
                    MonoTextStyle::new(&FONT_5X8, BinaryColor::On),
                    Alignment::Center,
                )
                .draw(display)?;
            }
        }
    }

    Ok(())
}

fn boxes<D>(display: &mut D, opts: [&'static str; 4]) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let Rectangle {
        size: Size { width, height },
        ..
    } = display.bounding_box();

    let text_style = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);

    // Top left
    Text::with_alignment(opts[0], Point::new(3, 8), text_style, Alignment::Left).draw(display)?;
    // Top right
    Text::with_alignment(
        opts[1],
        Point::new(width as i32 - 3, 8),
        text_style,
        Alignment::Right,
    )
    .draw(display)?;
    // Bottom left
    Text::with_alignment(
        opts[2],
        Point::new(3, height as i32 - 4),
        text_style,
        Alignment::Left,
    )
    .draw(display)?;
    // Bottom right
    Text::with_alignment(
        opts[3],
        Point::new(width as i32 - 2, height as i32 - 4),
        text_style,
        Alignment::Right,
    )
    .draw(display)?;

    let rec_size = Size {
        height: 12,
        width: 25,
    };

    let stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    // Top left outline
    Rectangle::new(Point { x: 0, y: 0 }, rec_size)
        .into_styled(stroke)
        .draw(display)?;
    // Top right outline
    Rectangle::new(
        Point {
            x: 0,
            y: height as i32 - 12,
        },
        rec_size,
    )
    .into_styled(stroke)
    .draw(display)?;
    // Bottom left outline
    Rectangle::new(
        Point {
            x: width as i32 - 25,
            y: 0,
        },
        rec_size,
    )
    .into_styled(stroke)
    .draw(display)?;
    // Bottom right outline
    Rectangle::new(
        Point {
            x: width as i32 - 25,
            y: height as i32 - 12,
        },
        rec_size,
    )
    .into_styled(stroke)
    .draw(display)?;

    let lrg = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::with_alignment(
        "DrivePi",
        Point {
            x: display.bounding_box().center().x,
            y: 8,
        },
        lrg,
        Alignment::Center,
    )
    .draw(display)?;

    Ok(())
}
