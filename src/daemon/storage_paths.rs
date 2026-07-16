use crate::config;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StoragePaths {
    pub(crate) mosaico_home: PathBuf,
    pub(crate) config_path: PathBuf,
    pub(crate) socket_path: PathBuf,
    pub(crate) lock_path: PathBuf,
    pub(crate) daemon_log_path: PathBuf,
    pub(crate) state_db_path: PathBuf,
    pub(crate) nmp_store_path: PathBuf,
    pub(crate) mosaico_home_set: bool,
    pub(crate) mosaico_home_is_default: bool,
    pub(crate) isolated_home_acknowledged: bool,
}

impl StoragePaths {
    pub(crate) fn current() -> Self {
        let home = config::mosaico_home_selection();
        Self {
            nmp_store_path: home.mosaico_home.join("nmp.redb"),
            mosaico_home: home.mosaico_home,
            config_path: config::config_path(),
            socket_path: super::socket_path(),
            lock_path: super::lock_path(),
            daemon_log_path: super::log_path(),
            state_db_path: super::store_path(),
            mosaico_home_set: home.mosaico_home_set,
            mosaico_home_is_default: home.mosaico_home_is_default,
            isolated_home_acknowledged: config::isolated_home_acknowledged(),
        }
    }
}
