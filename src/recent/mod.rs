use crate::keybindings::config_dir;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

const LIMIT: usize = 20;

pub(crate) fn load() -> io::Result<Vec<PathBuf>> {
    match fs::read(config_dir()?.join("recent.conf")) {
        Ok(bytes) => Ok(decode(&bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

pub(crate) fn save(paths: &[PathBuf]) -> io::Result<()> {
    let directory = config_dir()?;
    fs::create_dir_all(&directory)?;
    let path = directory.join("recent.conf");
    let temporary = directory.join("recent.tmp");
    fs::write(&temporary, encode(paths))?;
    fs::rename(temporary, path)
}

pub(crate) fn remember(paths: &mut Vec<PathBuf>, path: &Path) {
    paths.retain(|other| other != path);
    paths.insert(0, path.to_path_buf());
    paths.truncate(LIMIT);
}

fn encode(paths: &[PathBuf]) -> Vec<u8> {
    paths
        .iter()
        .flat_map(|path| {
            let mut bytes = path.to_string_lossy().as_bytes().to_vec();
            bytes.push(0);
            bytes
        })
        .collect()
}

fn decode(bytes: &[u8]) -> Vec<PathBuf> {
    bytes
        .split(|byte| *byte == 0)
        .filter_map(|part| std::str::from_utf8(part).ok())
        .filter(|part| !part.is_empty())
        .take(LIMIT)
        .map(PathBuf::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_files_round_trip_and_deduplicate() {
        let mut paths = vec![];
        remember(&mut paths, Path::new("/tmp/one.md"));
        remember(&mut paths, Path::new("/tmp/two\nlines.md"));
        remember(&mut paths, Path::new("/tmp/one.md"));
        assert_eq!(paths.len(), 2);
        assert_eq!(decode(&encode(&paths)), paths);
    }
}
