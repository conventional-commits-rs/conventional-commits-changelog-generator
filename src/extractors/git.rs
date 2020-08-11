use crate::extractors::utils::extract_repo_from_url;
use crate::extractors::Extractor;
use crate::extractors::RepoInformation;
use git2::{Remote, Repository};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitExtractorError {
    /// Thrown when no matching remote could be found.
    #[error("no matching remote found")]
    NoMatchingRemote,
    #[error("no url specified for remote {remote:?}")]
    NoUrlInRemote { remote: String },
}

struct GitExtractor;

impl GitExtractor {
    fn extract_from_remote(
        &self,
        remote: Remote,
    ) -> Result<RepoInformation, Box<dyn std::error::Error>> {
        if let Some(url) = remote.url() {
            extract_repo_from_url(url)
        } else {
            Err(Box::new(GitExtractorError::NoUrlInRemote {
                remote: remote.name().unwrap_or_else(|| "<not found>").to_string(),
            }))
        }
    }
}

impl Extractor for GitExtractor {
    fn extract_repo_information(
        &self,
        repo_dir: &Path,
    ) -> Result<RepoInformation, Box<dyn std::error::Error>> {
        println!("git");
        let repo = Repository::open(repo_dir)?;
        let upstream_remote = repo.find_remote("upstream");
        match upstream_remote {
            Ok(remote) => {
                return self.extract_from_remote(remote);
            }
            Err(_) => {
                if let Ok(remote) = repo.find_remote("origin") {
                    return self.extract_from_remote(remote);
                }
            }
        }

        Err(Box::new(GitExtractorError::NoMatchingRemote))
    }

    fn priority(&self) -> u8 {
        0
    }

    fn is_applicable(&self, _repo_dir: &Path) -> bool {
        // TODO: check for git repository.
        println!("git check");
        true
    }
}

inventory::submit! {
    &GitExtractor as &dyn Extractor
}
