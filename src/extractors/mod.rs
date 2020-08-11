use std::path::Path;

#[cfg(feature = "extractor-cargo")]
mod cargo;
#[cfg(feature = "extractor-git")]
mod git;
mod utils;

/// The priority that should be used for extractors that rely on language
/// specified files.
pub const LANGUAGE_RELATED_PRIORITY: u8 = 50;

/// Information about a remote repository.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RepoInformation {
    pub owner: String,
    pub repo: String,
}

/// An extractor is used to extract certain information from a project.
pub trait Extractor {
    /// Extracts information about the upstream repository for a project.
    ///
    /// # Arguments
    ///
    /// `repo_dir`: The directory the repository is in.
    ///
    /// # Returns
    ///
    /// The information (when resolved) or an error if it was not possible.
    fn extract_repo_information(
        &self,
        repo_dir: &Path,
    ) -> Result<RepoInformation, Box<dyn std::error::Error>>;

    /// Returns the priority of the extractor.
    ///
    /// The lower the priority the earlier the extractor gets executed.
    ///
    /// # Note
    ///
    /// Extractors that implement logic for certain development environments
    /// (`cargo` for Rust, `npm` for Node) should implement a priority level of
    /// 25.
    fn priority(&self) -> u8;

    /// Returns whether the executor is applicable to extract information from a
    /// repository.
    ///
    /// This method is mostly used to check for certain files that need to exist
    /// for information extraction.
    fn is_applicable(&self, repo_dir: &Path) -> bool;
}

inventory::collect!(&'static dyn Extractor);
