//! Contains utility functions related to GitHub.

use crate::extractors::RepoInformation;

pub(crate) fn github_link_for_range(from: &str, to: &str, info: &RepoInformation) -> String {
    format!(
        "https://github.com/{}/{}/compare/{}...{}",
        info.owner, info.repo, from, to
    )
}

pub(crate) fn github_link_for_commit(hash: &str, info: &RepoInformation) -> String {
    format!(
        "https://github.com/{}/{}/commit/{}",
        info.owner, info.repo, hash
    )
}

pub(crate) fn github_link_for_issue(issue_nr: &str, info: &RepoInformation) -> String {
    format!(
        "https://github.com/{}/{}/issues/{}",
        info.owner, info.repo, issue_nr
    )
}
