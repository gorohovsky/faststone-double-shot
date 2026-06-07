//! The deletion mode and its confirmation policy.

/// How matched files are removed, and when the user must confirm first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Send files to the Recycle Bin (recoverable). Silent for small sets.
    RecycleBin,
    /// Delete files permanently (unrecoverable). Always confirms.
    Permanent
}

impl Mode {
    /// Recycle Bin deletes run silently up to and including this many files.
    const RECYCLE_SILENT_LIMIT: usize = 2;

    /// Whether the user must approve before deleting `count` files.
    pub fn requires_confirmation(self, count: usize) -> bool {
        match self {
            Mode::RecycleBin => count > Self::RECYCLE_SILENT_LIMIT,
            Mode::Permanent => true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Mode;

    #[test]
    fn recycle_bin_is_silent_for_a_pair() {
        assert!(!Mode::RecycleBin.requires_confirmation(2));
    }

    #[test]
    fn recycle_bin_confirms_beyond_two_files() {
        assert!(Mode::RecycleBin.requires_confirmation(3));
    }

    #[test]
    fn permanent_confirms_even_for_a_single_file() {
        assert!(Mode::Permanent.requires_confirmation(1));
    }
}
