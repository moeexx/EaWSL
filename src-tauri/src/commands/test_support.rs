use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) fn unique_temp_path(scope: &str, name: &str) -> PathBuf {
    let unique_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("eawsl-{scope}-{name}-{unique_id}"))
}
