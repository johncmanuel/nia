use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn save_file(path: &Path, contents: &str, expected: Option<&str>) -> io::Result<()> {
    match expected {
        Some(original) => {
            if fs::read_to_string(path)? != original {
                return Err(io::Error::new(
                    ErrorKind::AlreadyExists,
                    "the file changed on disk; your edits are still in the editor",
                ));
            }
        }
        None if path.exists() => {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                "a file already exists at that path",
            ));
        }
        None => {}
    }

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = path.file_name().ok_or_else(|| {
        io::Error::new(
            ErrorKind::InvalidInput,
            "choose a filename for the document",
        )
    })?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    let temporary = parent.join(format!(
        ".{}.nia-{}-{nonce}.tmp",
        name.to_string_lossy(),
        std::process::id()
    ));

    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        if expected.is_some() {
            file.set_permissions(fs::metadata(path)?.permissions())?;
        }
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        drop(file);
        if expected.is_some() {
            fs::rename(&temporary, path)
        } else {
            fs::hard_link(&temporary, path)
        }
    })();

    if let Err(error) = fs::remove_file(&temporary)
        && error.kind() != ErrorKind::NotFound
    {
        log::warn!("temporary save file cleanup failed: {:?}", error.kind());
    }
    result
}

#[cfg(test)]
mod tests;
