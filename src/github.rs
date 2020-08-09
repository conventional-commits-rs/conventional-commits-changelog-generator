//! Contains utility functions related to GitHub.

pub(crate) fn github_link_for_range(from: &str, to: &str) -> String {
    format!(
        "https://github.com/SirWindfield/cargo-create/compare/{}...{}",
        from, to
    )
}

pub(crate) fn github_link_for_commit(hash: &str) -> String {
    format!(
        "https://github.com/SirWindfield/cargo-create/commit/{}",
        hash
    )
}

pub(crate) fn github_link_for_issue(issue_nr: &str) -> String {
    format!(
        "https://github.com/SirWindfield/cargo-create/issues/{}",
        issue_nr
    )
}
