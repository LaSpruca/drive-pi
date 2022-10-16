use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{error::Error, fs, io, path::PathBuf};
use sys_mount::{Mount, SupportedFilesystems, UnmountDrop, UnmountFlags};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceQuery {
    pub blockdevices: Vec<Blockdevice>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blockdevice {
    pub name: String,
    pub fstype: Value,
    pub fsver: Value,
    pub label: Value,
    pub uuid: Value,
    pub fsavail: Value,
    #[serde(rename = "fsuse%")]
    pub fsuse: Value,
    pub mountpoints: Vec<Option<String>>,
    #[serde(default)]
    pub children: Vec<Children>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Children {
    pub name: String,
    pub fstype: Option<String>,
    pub fsver: Option<String>,
    pub label: Option<String>,
    pub uuid: Option<String>,
    pub fsavail: Option<String>,
    #[serde(rename = "fsuse%")]
    pub fsuse: Option<String>,
    pub mountpoints: Vec<Option<String>>,
}

#[derive(Clone)]
pub struct Device {
    pub name: String,
    pub fs: String,
    pub mounted: bool,
    pub size: String,
}

impl Device {
    pub fn mount(&self, base_path: PathBuf) -> io::Result<UnmountDrop<Mount>> {
        let dev = self.name.clone();

        let target = base_path.join(dev.clone());
        let source = PathBuf::from("/dev").join(dev);

        if !target.exists() {
            println!("Creating mount point");

            fs::create_dir(&target)?;
        }

        println!("{}", source.display());

        let supported = SupportedFilesystems::new().unwrap();

        println!("{supported:?}");

        println!("{}", supported.is_supported("ntfs"));

        match sys_mount::MountBuilder::default()
            .fstype(self.fs.as_str())
            .mount_autodrop(source, target.clone(), UnmountFlags::empty())
        {
            Ok(x) => Ok(x),
            Err(ex) => {
                fs::remove_dir(target)?;
                Err(ex)
            }
        }
    }
}

pub fn get_devices(mount_point: &str) -> Result<Vec<Device>, Box<dyn Error>> {
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
                if mountpoints
                    .iter()
                    .find(|x| x.starts_with(mount_point))
                    .is_none()
                {
                    continue 'inner;
                }
                mounted = true;
            }

            println!("{} {:?}", part.name, part.fstype);

            if let Some(ref fstype) = part.fstype {
                devices.push(Device {
                    name: part.name.clone(),
                    size: part.size.clone(),
                    fs: fstype.to_string(),
                    mounted,
                });
            }
        }
    }

    Ok(devices)
}
