use std::env;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

pub(crate) fn directory() -> io::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    let base = env::var_os("APPDATA").map(PathBuf::from);

    #[cfg(target_os = "macos")]
    let base = env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library/Application Support"));

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".config"))
        });

    base.map(|base| base.join("nia"))
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "configuration directory unavailable"))
}
