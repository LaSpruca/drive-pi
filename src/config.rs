use std::{fs::File, io::Read, path::PathBuf};

macro_rules! s_default {
    {$name:ident $type:ty = $value:expr} => {
        fn $name() -> $type {
            $value
        }
    };
}

s_default! { mount_path PathBuf = PathBuf::from("./") }

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    #[serde(default = "mount_path")]
    pub mount_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mount_path: mount_path(),
        }
    }
}

impl Config {
    pub fn load() -> Option<Self> {
        let mut file = match File::open("/etc/drive-pi/config.toml") {
            Ok(file) => file,
            Err(_) => match File::open("./config.toml") {
                Ok(file) => file,
                Err(ex) => {
                    eprintln!("{ex:?}");
                    return None;
                }
            },
        };

        let mut source = String::new();
        if let Err(ex) = file.read_to_string(&mut source) {
            eprintln!("{ex:?}");
            return None;
        }

        toml::from_str(&source).ok()
    }
}
