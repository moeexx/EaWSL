use std::io;

use crate::domain::model::distro::RegisteredDistroMetadata;
use crate::WslError;

use super::mapper::map_registry_record;

const LXSS_ROOT_DISPLAY_PATH: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Lxss";
const LXSS_ROOT_SUBKEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Lxss";

#[derive(Debug)]
pub(super) struct RegistryRawRecord {
    pub distribution_name: String,
    pub version: u8,
    pub base_path: Option<String>,
    pub vhd_file_name: Option<String>,
    pub flavor: Option<String>,
    pub os_version: Option<String>,
    pub default_uid: Option<u32>,
}

pub(super) trait RegistryProvider {
    fn list_subkeys(&self, path: &str) -> Result<Vec<String>, String>;
    fn read_string(&self, path: &str, value_name: &str) -> Result<Option<String>, String>;
    fn read_u32(&self, path: &str, value_name: &str) -> Result<Option<u32>, String>;
}

struct SystemRegistryProvider;

pub(super) fn read_all_distros() -> Result<Vec<RegisteredDistroMetadata>, WslError> {
    read_all_distros_with_provider(&SystemRegistryProvider)
}

fn read_all_distros_with_provider<P>(
    provider: &P,
) -> Result<Vec<RegisteredDistroMetadata>, WslError>
where
    P: RegistryProvider,
{
    let mut guids = provider
        .list_subkeys(LXSS_ROOT_SUBKEY_PATH)
        .map_err(|detail| registry_read_failed(LXSS_ROOT_DISPLAY_PATH, detail))?;
    guids.sort();

    guids
        .into_iter()
        .map(|guid| read_raw_distro_with_provider(provider, &guid).map(map_registry_record))
        .collect()
}

fn read_raw_distro_with_provider<P>(provider: &P, guid: &str) -> Result<RegistryRawRecord, WslError>
where
    P: RegistryProvider,
{
    let subkey_path = distro_subkey_path(guid);
    let display_path = distro_display_path(guid);

    let distribution_name =
        read_required_string(provider, &subkey_path, &display_path, "DistributionName")?;
    let version = read_required_u8(provider, &subkey_path, &display_path, "Version")?;
    read_required_u32(provider, &subkey_path, &display_path, "State")?;

    Ok(RegistryRawRecord {
        distribution_name,
        version,
        base_path: read_optional_string(provider, &subkey_path, &display_path, "BasePath")?,
        vhd_file_name: read_optional_string(provider, &subkey_path, &display_path, "VhdFileName")?,
        flavor: read_optional_string(provider, &subkey_path, &display_path, "Flavor")?,
        os_version: read_optional_string(provider, &subkey_path, &display_path, "OsVersion")?,
        default_uid: read_optional_u32(provider, &subkey_path, &display_path, "DefaultUid")?,
    })
}

fn read_required_string<P>(
    provider: &P,
    path: &str,
    display_path: &str,
    value_name: &str,
) -> Result<String, WslError>
where
    P: RegistryProvider,
{
    provider
        .read_string(path, value_name)
        .map_err(|detail| registry_read_failed(display_path, detail))?
        .ok_or_else(|| missing_required_value(display_path, value_name))
}

fn read_optional_string<P>(
    provider: &P,
    path: &str,
    display_path: &str,
    value_name: &str,
) -> Result<Option<String>, WslError>
where
    P: RegistryProvider,
{
    provider
        .read_string(path, value_name)
        .map_err(|detail| registry_read_failed(display_path, detail))
}

fn read_required_u8<P>(
    provider: &P,
    path: &str,
    display_path: &str,
    value_name: &str,
) -> Result<u8, WslError>
where
    P: RegistryProvider,
{
    let value = read_required_u32(provider, path, display_path, value_name)?;
    u8::try_from(value).map_err(|_| {
        registry_read_failed(
            display_path,
            format!("{value_name} value {value} does not fit in u8"),
        )
    })
}

fn read_required_u32<P>(
    provider: &P,
    path: &str,
    display_path: &str,
    value_name: &str,
) -> Result<u32, WslError>
where
    P: RegistryProvider,
{
    provider
        .read_u32(path, value_name)
        .map_err(|detail| registry_read_failed(display_path, detail))?
        .ok_or_else(|| missing_required_value(display_path, value_name))
}

fn read_optional_u32<P>(
    provider: &P,
    path: &str,
    display_path: &str,
    value_name: &str,
) -> Result<Option<u32>, WslError>
where
    P: RegistryProvider,
{
    provider
        .read_u32(path, value_name)
        .map_err(|detail| registry_read_failed(display_path, detail))
}

fn missing_required_value(display_path: &str, value_name: &str) -> WslError {
    registry_read_failed(display_path, format!("missing required value {value_name}"))
}

fn registry_read_failed(display_path: &str, detail: impl Into<String>) -> WslError {
    WslError::RegistryReadFailed {
        key: Some(display_path.to_string()),
        detail: detail.into(),
    }
}

fn distro_subkey_path(guid: &str) -> String {
    format!(r"{LXSS_ROOT_SUBKEY_PATH}\{guid}")
}

fn distro_display_path(guid: &str) -> String {
    format!(r"{LXSS_ROOT_DISPLAY_PATH}\{guid}")
}

impl RegistryProvider for SystemRegistryProvider {
    fn list_subkeys(&self, path: &str) -> Result<Vec<String>, String> {
        let key = open_subkey(path)?;
        let mut subkeys = Vec::new();

        for subkey in key.enum_keys() {
            subkeys.push(subkey.map_err(|err| err.to_string())?);
        }

        Ok(subkeys)
    }

    fn read_string(&self, path: &str, value_name: &str) -> Result<Option<String>, String> {
        let key = open_subkey(path)?;
        match key.get_value::<String, _>(value_name) {
            Ok(value) => Ok(Some(value)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }

    fn read_u32(&self, path: &str, value_name: &str) -> Result<Option<u32>, String> {
        let key = open_subkey(path)?;
        match key.get_value::<u32, _>(value_name) {
            Ok(value) => Ok(Some(value)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }
}

fn open_subkey(path: &str) -> Result<winreg::RegKey, String> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(path)
        .map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        distro_subkey_path, read_all_distros_with_provider, read_raw_distro_with_provider,
        LXSS_ROOT_SUBKEY_PATH,
    };
    use crate::WslError;

    use std::collections::{BTreeMap, HashMap};

    #[derive(Default)]
    struct FakeRegistryProvider {
        keys: BTreeMap<String, FakeRegistryKey>,
    }

    #[derive(Default)]
    struct FakeRegistryKey {
        subkeys: Vec<String>,
        values: HashMap<String, FakeRegistryValue>,
    }

    enum FakeRegistryValue {
        String(String),
        U32(u32),
    }

    impl FakeRegistryProvider {
        fn with_root_subkeys(mut self, subkeys: &[&str]) -> Self {
            self.keys.insert(
                LXSS_ROOT_SUBKEY_PATH.to_string(),
                FakeRegistryKey {
                    subkeys: subkeys.iter().map(|value| value.to_string()).collect(),
                    values: HashMap::new(),
                },
            );
            self
        }

        fn with_distro(
            mut self,
            guid: &str,
            values: impl IntoIterator<Item = (&'static str, FakeRegistryValue)>,
        ) -> Self {
            self.keys.insert(
                distro_subkey_path(guid),
                FakeRegistryKey {
                    subkeys: Vec::new(),
                    values: values
                        .into_iter()
                        .map(|(name, value)| (name.to_string(), value))
                        .collect(),
                },
            );
            self
        }
    }

    impl super::RegistryProvider for FakeRegistryProvider {
        fn list_subkeys(&self, path: &str) -> Result<Vec<String>, String> {
            self.keys
                .get(path)
                .map(|key| key.subkeys.clone())
                .ok_or_else(|| "key not found".to_string())
        }

        fn read_string(&self, path: &str, value_name: &str) -> Result<Option<String>, String> {
            match self.keys.get(path) {
                Some(key) => match key.values.get(value_name) {
                    Some(FakeRegistryValue::String(value)) => Ok(Some(value.clone())),
                    Some(FakeRegistryValue::U32(_)) => {
                        Err(format!("value {value_name} has unexpected type"))
                    }
                    None => Ok(None),
                },
                None => Err("key not found".to_string()),
            }
        }

        fn read_u32(&self, path: &str, value_name: &str) -> Result<Option<u32>, String> {
            match self.keys.get(path) {
                Some(key) => match key.values.get(value_name) {
                    Some(FakeRegistryValue::U32(value)) => Ok(Some(*value)),
                    Some(FakeRegistryValue::String(_)) => {
                        Err(format!("value {value_name} has unexpected type"))
                    }
                    None => Ok(None),
                },
                None => Err("key not found".to_string()),
            }
        }
    }

    fn string(value: &str) -> FakeRegistryValue {
        FakeRegistryValue::String(value.to_string())
    }

    fn u32_value(value: u32) -> FakeRegistryValue {
        FakeRegistryValue::U32(value)
    }

    #[test]
    fn read_all_distros_parses_multiple_subkeys() {
        let provider = FakeRegistryProvider::default()
            .with_root_subkeys(&["{b-guid}", "{a-guid}"])
            .with_distro(
                "{a-guid}",
                [
                    ("DistributionName", string("Ubuntu")),
                    ("Version", u32_value(2)),
                    ("State", u32_value(1)),
                    ("BasePath", string(r"\\?\D:\WSL\Ubuntu")),
                    ("VhdFileName", string("ext4.vhdx")),
                    ("Flavor", string("ubuntu")),
                    ("OsVersion", string("24.04")),
                    ("DefaultUid", u32_value(1000)),
                ],
            )
            .with_distro(
                "{b-guid}",
                [
                    ("DistributionName", string("Debian")),
                    ("Version", u32_value(1)),
                    ("State", u32_value(0)),
                ],
            );

        let entries = read_all_distros_with_provider(&provider).expect("read distros");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "Ubuntu");
        assert_eq!(
            entries[0].base_path.as_deref(),
            Some(std::path::Path::new(r"D:\WSL\Ubuntu"))
        );
        assert_eq!(entries[0].vhd_file_name.as_deref(), Some("ext4.vhdx"));
        assert_eq!(entries[0].flavor.as_deref(), Some("ubuntu"));
        assert_eq!(entries[0].os_version.as_deref(), Some("24.04"));
        assert_eq!(entries[0].default_uid, Some(1000));
        assert_eq!(entries[1].name, "Debian");
    }

    #[test]
    fn read_raw_distro_fails_when_required_field_is_missing() {
        let provider = FakeRegistryProvider::default().with_distro(
            "{guid}",
            [("Version", u32_value(2)), ("State", u32_value(1))],
        );

        let err = read_raw_distro_with_provider(&provider, "{guid}")
            .expect_err("missing required value should fail");
        match err {
            WslError::RegistryReadFailed { key, detail } => {
                assert_eq!(
                    key.as_deref(),
                    Some(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Lxss\{guid}")
                );
                assert!(detail.contains("DistributionName"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn read_all_distros_fails_when_root_key_is_missing() {
        let provider = FakeRegistryProvider::default();

        let err = read_all_distros_with_provider(&provider).expect_err("missing root should fail");
        match err {
            WslError::RegistryReadFailed { key, detail } => {
                assert_eq!(
                    key.as_deref(),
                    Some(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Lxss")
                );
                assert!(detail.contains("key not found"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn read_raw_distro_fails_when_required_numeric_field_is_out_of_range() {
        let provider = FakeRegistryProvider::default().with_distro(
            "{guid}",
            [
                ("DistributionName", string("Ubuntu")),
                ("Version", u32_value(300)),
                ("State", u32_value(1)),
            ],
        );

        let err = read_raw_distro_with_provider(&provider, "{guid}")
            .expect_err("out-of-range version should fail");
        match err {
            WslError::RegistryReadFailed { detail, .. } => {
                assert!(detail.contains("does not fit in u8"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}
