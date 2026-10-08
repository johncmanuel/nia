use std::io::{self, ErrorKind};

#[derive(Clone, Debug)]
pub(crate) struct FileFailure {
    pub(crate) kind: ErrorKind,
    detail: String,
}

impl From<io::Error> for FileFailure {
    fn from(error: io::Error) -> Self {
        Self {
            kind: error.kind(),
            detail: error.to_string(),
        }
    }
}

impl FileFailure {
    pub(crate) fn open_notice(&self) -> String {
        match self.kind {
            ErrorKind::NotFound => "File not found. Check its location and try again.".into(),
            ErrorKind::PermissionDenied => {
                "Cannot open the file: permission denied. Check its access settings.".into()
            }
            ErrorKind::InvalidData => "Cannot open the file: it is not valid UTF-8 text.".into(),
            _ => format!("Could not open file: {}", self.detail),
        }
    }

    pub(crate) fn save_notice(&self) -> String {
        match self.kind {
            ErrorKind::AlreadyExists => {
                "The file changed on disk or already exists. Your edits are still here. Use Save As to choose another file.".into()
            }
            ErrorKind::PermissionDenied => {
                "Cannot save: permission denied. Your edits are still here. Check access or use Save As.".into()
            }
            ErrorKind::StorageFull => {
                "Storage is full. Your edits are still here. Free space or use Save As.".into()
            }
            ErrorKind::NotFound => {
                "Cannot save: the file or folder is missing. Your edits are still here. Use Save As.".into()
            }
            _ => format!("Could not save file: {}", self.detail),
        }
    }

    pub(crate) fn recovery_notice(&self) -> &'static str {
        match self.kind {
            ErrorKind::StorageFull => {
                "Storage is full. This draft is not backed up yet. Free space or save the document elsewhere."
            }
            ErrorKind::PermissionDenied => {
                "Cannot write a recovery draft: permission denied. Save the document elsewhere."
            }
            _ => "Could not write a local recovery draft. The editor will retry.",
        }
    }

    pub(crate) fn external_notice(&self) -> &'static str {
        match self.kind {
            ErrorKind::NotFound => "File was moved or deleted. Use Save As to keep your edits.",
            ErrorKind::PermissionDenied => {
                "Cannot read the file: permission denied. Use Save As to keep your edits."
            }
            ErrorKind::InvalidData => {
                "File is no longer valid UTF-8 text. Use Save As to keep your edits."
            }
            _ => "File is unavailable. Use Save As to keep your edits.",
        }
    }
}

#[cfg(test)]
impl From<&str> for FileFailure {
    fn from(detail: &str) -> Self {
        Self {
            kind: ErrorKind::Other,
            detail: detail.into(),
        }
    }
}

#[cfg(test)]
mod tests;
