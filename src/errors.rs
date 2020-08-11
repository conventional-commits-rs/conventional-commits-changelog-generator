use git2::Error as GitError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("no repository at {path:?}")]
    NoGitRepository {
        path: String,
        #[source]
        source: GitError,
    },
}
