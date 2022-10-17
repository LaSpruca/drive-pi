use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::{self, remove_dir},
    io::{self, ErrorKind},
    path::PathBuf,
    process::Command,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceQuery {
    pub blockdevices: Vec<Blockdevice>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blockdevice {
    #[serde(default)]
    pub children: Vec<Children>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Children {
    pub name: String,
    pub size: String,
    pub mountpoints: Vec<Option<String>>,
}

#[derive(Clone)]
pub struct Device {
    pub name: String,
    pub mounted: bool,
    pub size: String,
    pub path: PathBuf,
}

impl Device {
    pub fn mount(&self) -> io::Result<()> {
        let dev = self.name.clone();

        let target = self.path.clone();
        let source = PathBuf::from("/dev").join(dev);

        if !target.exists() {
            println!("Creating mount point");

            fs::create_dir(&target)?;
        }

        let output = Command::new("mount").arg(source).arg(&target).output()?;
        let err_str = String::from_utf8(output.stderr).unwrap();

        println!("{err_str}");
        println!("{}", output.status);

        if !output.status.success() {
            return Err(io::Error::new(ErrorKind::Other, err_str));
        }

        Ok(())
    }

    pub fn unmount(&self) -> io::Result<()> {
        let path = self.path.clone();

        Command::new("umount").arg(&path).output()?;
        remove_dir(&path)?;

        Ok(())
    }
}

pub fn get_devices(mount_point: &PathBuf) -> Result<Vec<Device>, Box<dyn Error>> {
    let command = String::from_utf8(
        std::process::Command::new("lsblk")
            .arg("--json")
            .output()?
            .stdout,
    )?;

    let output: DeviceQuery = serde_json::from_str(&command)?;
    let mut devices = vec![];
    let path_str = format!("{}", mount_point.canonicalize().unwrap().display());

    for device in output.blockdevices.iter() {
        'inner: for part in device.children.iter() {
            let mountpoints: Vec<String> = part
                .mountpoints
                .iter()
                .filter_map(|f| f.as_ref().map(|x| x.to_owned()))
                .collect();

            let mut mounted = false;

            if mountpoints.len() > 0 {
                if mountpoints
                    .iter()
                    .find(|x| x.starts_with(path_str.as_str()))
                    .is_none()
                {
                    continue 'inner;
                }
                mounted = true;
            }

            devices.push(Device {
                name: part.name.clone(),
                size: part.size.clone(),
                mounted,
                path: mount_point.join(part.name.clone()),
            });
        }
    }

    Ok(devices)
}
