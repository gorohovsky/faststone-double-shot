//! Finding every file that shares a base name with the selected photo.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Extensions treated as the same shot (lowercase, no leading dot).
///
/// Editing this list changes which files are paired; rebuild to apply.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", // JPEG
    "cr2", "cr3",  // Canon
    "nef",         // Nikon
    "arw",         // Sony
    "raf",         // Fujifilm
    "orf",         // Olympus
    "rw2",         // Panasonic
    "dng"          // Adobe DNG
];

/// Returns every supported file in `target`'s directory that shares its base
/// name, including `target` itself. Returns empty when `target` is not itself a
/// supported format, so acting on an unrecognised file never deletes its
/// neighbours while leaving the selection behind.
///
/// Comparisons are ASCII-case-insensitive: camera file names are ASCII, and on
/// a rare non-ASCII name this errs toward leaving a file rather than deleting
/// the wrong one. Results are sorted by file name for stable display.
pub fn find_matching_files(target: &Path) -> Vec<PathBuf> {
    if !is_supported(target) {
        return Vec::new();
    }

    let (Some(directory), Some(stem)) = (target.parent(), target.file_stem()) else {
        return Vec::new();
    };

    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut matches: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| shares_base_name(path, stem) && is_supported(path) && path.is_file())
        .collect();

    matches.sort_by_key(|path| sort_key(path));
    matches
}

fn shares_base_name(path: &Path, stem: &OsStr) -> bool {
    path.file_stem().is_some_and(|candidate| candidate.eq_ignore_ascii_case(stem))
}

fn is_supported(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| SUPPORTED_EXTENSIONS.iter().any(|supported| extension.eq_ignore_ascii_case(supported)))
}

fn sort_key(path: &Path) -> String {
    path.file_name().unwrap_or_default().to_string_lossy().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::find_matching_files;
    use crate::test_support::{touch, TempDir};
    use std::path::Path;

    fn matched_names(target: &Path) -> Vec<String> {
        find_matching_files(target)
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn pairs_a_jpg_with_its_raw() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "shot.jpg");
        touch(&directory, "shot.cr2");

        assert_eq!(matched_names(&jpg), ["shot.cr2", "shot.jpg"]);
    }

    #[test]
    fn matches_multiple_raw_formats_for_one_shot() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "shot.jpg");
        touch(&directory, "shot.nef");
        touch(&directory, "shot.dng");

        assert_eq!(matched_names(&jpg), ["shot.dng", "shot.jpg", "shot.nef"]);
    }

    #[test]
    fn matches_a_lone_jpg() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "lonely.jpg");

        assert_eq!(matched_names(&jpg), ["lonely.jpg"]);
    }

    #[test]
    fn matches_a_lone_raw() {
        let directory = TempDir::new();
        let arw = touch(&directory, "lonely.arw");

        assert_eq!(matched_names(&arw), ["lonely.arw"]);
    }

    #[test]
    fn does_not_cross_match_similar_names() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "name.jpg");
        touch(&directory, "name.cr2");
        touch(&directory, "name2.jpg");
        touch(&directory, "name2.cr2");

        assert_eq!(matched_names(&jpg), ["name.cr2", "name.jpg"]);
    }

    #[test]
    fn matches_uppercase_extensions() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "upper.JPG");
        touch(&directory, "upper.CR2");

        assert_eq!(matched_names(&jpg), ["upper.CR2", "upper.JPG"]);
    }

    #[test]
    fn matches_base_names_case_insensitively() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "IMG_0001.jpg");
        touch(&directory, "img_0001.cr2");

        assert_eq!(matched_names(&jpg), ["img_0001.cr2", "IMG_0001.jpg"]);
    }

    #[test]
    fn matches_three_or_more_files() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "burst.jpg");
        touch(&directory, "burst.jpeg");
        touch(&directory, "burst.cr2");
        touch(&directory, "burst.nef");

        assert_eq!(matched_names(&jpg), ["burst.cr2", "burst.jpeg", "burst.jpg", "burst.nef"]);
    }

    #[test]
    fn ignores_unsupported_extensions() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "mixed.jpg");
        touch(&directory, "mixed.cr2");
        touch(&directory, "mixed.tiff");

        assert_eq!(matched_names(&jpg), ["mixed.cr2", "mixed.jpg"]);
    }

    #[test]
    fn ignores_everything_when_the_target_is_unsupported() {
        let directory = TempDir::new();
        let tiff = touch(&directory, "shot.tiff");
        touch(&directory, "shot.jpg");
        touch(&directory, "shot.cr2");

        assert!(matched_names(&tiff).is_empty());
    }

    #[test]
    fn matches_names_with_spaces() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "a shot with spaces.jpg");
        touch(&directory, "a shot with spaces.cr2");

        assert_eq!(matched_names(&jpg), ["a shot with spaces.cr2", "a shot with spaces.jpg"]);
    }

    #[test]
    fn matches_names_with_special_characters() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "(edit)-final.jpg");
        touch(&directory, "(edit)-final.arw");

        assert_eq!(matched_names(&jpg), ["(edit)-final.arw", "(edit)-final.jpg"]);
    }
}
