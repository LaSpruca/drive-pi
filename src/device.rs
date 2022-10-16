use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceQuery {
    pub blockdevices: Vec<Blockdevice>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blockdevice {
    pub name: String,
    #[serde(rename = "maj:min")]
    pub maj_min: String,
    pub rm: bool,
    pub size: String,
    pub ro: bool,
    #[serde(rename = "type")]
    pub type_field: String,
    pub mountpoints: Vec<Option<String>>,
    #[serde(default)]
    pub children: Vec<Children>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Children {
    pub name: String,
    #[serde(rename = "maj:min")]
    pub maj_min: String,
    pub rm: bool,
    pub size: String,
    pub ro: bool,
    #[serde(rename = "type")]
    pub type_field: String,
    pub mountpoints: Vec<Option<String>>,
}

pub fn get_devices() -> Result<Vec<(String, String, bool)>, Box<dyn Error>> {
    let command = String::from_utf8(
        std::process::Command::new("lsblk")
            .arg("--json")
            .output()?
            .stdout,
    )?;

    let output: DeviceQuery = serde_json::from_str(&command)?;
    let mut devices = vec![];

    for device in output.blockdevices.iter() {
        'inner: for part in device.children.iter() {
            let mountpoints: Vec<String> = part
                .mountpoints
                .iter()
                .filter_map(|f| f.as_ref().map(|x| x.to_owned()))
                .collect();

            let mut mounted = false;

            if mountpoints.len() > 0 {
                if mountpoints.contains(&"[SWAP]".to_string())
                    || mountpoints.contains(&"/".to_string())
                    || mountpoints.contains(&"/boot".to_string())
                    || mountpoints.contains(&"/boot/efi".to_string())
                {
                    continue 'inner;
                }
                mounted = true;
            }

            devices.push((part.name.clone(), part.size.clone(), mounted));
        }
    }

    Ok(devices)
}
