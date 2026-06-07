//! Human-readable text for the confirmation and error dialogs.

use std::path::Path;

/// Confirmation prompt listing the full path of every file to be deleted.
pub fn confirmation(paths: &[impl AsRef<Path>]) -> String {
    let list = paths
        .iter()
        .map(|path| format!(" - {}", path.as_ref().display()))
        .collect::<Vec<_>>()
        .join("\n");

    format!("Delete the following {} file(s)?\n\n{}", paths.len(), list)
}

/// Error report listing the names of files that could not be deleted.
pub fn error(file_names: &[String]) -> String {
    format!("Could not delete:\n{}", file_names.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::{confirmation, error};

    #[test]
    fn confirmation_lists_each_path_under_a_count_header() {
        let paths = ["C:\\photos\\IMG_1234.jpg", "C:\\photos\\IMG_1234.cr2"];

        assert_eq!(
            confirmation(&paths),
            "Delete the following 2 file(s)?\n\n - C:\\photos\\IMG_1234.jpg\n - C:\\photos\\IMG_1234.cr2"
        );
    }

    #[test]
    fn error_lists_each_failed_name_under_a_header() {
        let failed = vec!["IMG_1234.cr2".to_string()];

        assert_eq!(error(&failed), "Could not delete:\nIMG_1234.cr2");
    }
}
