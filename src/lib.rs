pub use crate::errors::Error;
use crate::extractors::Extractor;
use crate::extractors::RepoInformation;
use crate::github::github_link_for_commit;
use crate::github::github_link_for_issue;
use crate::github::github_link_for_range;
use crate::utils::pairwise;
use chrono::{DateTime, NaiveDateTime, Utc};
use git2::Repository;
use markdown_composer::traits::MarkdownElement;
use markdown_composer::transforms::Bold;
use markdown_composer::PRELIMINARY_REMARK;
use markdown_composer::{Link, List, Markdown};
use semver::SemVerError;
use semver::Version;
use std::cmp::Ordering;
use std::error::Error as StdError;
use std::path::Path;

mod errors;
mod extractors;
mod github;
mod utils;

fn populate_changelog_from_commits(
    commits: Vec<&(&git2::Commit, conventional_commits_parser::Commit)>,
    repo_info: &RepoInformation,
) -> Result<List, Box<dyn StdError>> {
    let mut list = List::unordered();

    // For each commit...
    for (commit, p) in commits {
        // If a scope is present, make it bold and combine it with the commit description.
        let mut item = match p.scope {
            Some(scope) => format!("{}: {}", scope.bold(), p.desc),
            None => p.desc.to_string(),
        };

        // Additional information that can be appended to a changelog line. Currently issue number and the commit hash.
        let mut additions = Vec::with_capacity(2);
        let short_hash = commit.as_object().short_id()?;
        let short_hash = short_hash.as_str().unwrap();
        let hash_link = Link::builder()
            .text(short_hash)
            .url(github_link_for_commit(&commit.id().to_string(), &repo_info))
            .inlined()
            .build();

        additions.push(format!("({})", hash_link));

        // If the footer contains fixes or closes, add them as well.
        for footer in &p.footer {
            match footer.token {
                "Fixes" | "Closes" => {
                    let issue_link = Link::builder()
                        .text(format!("#{}", footer.value))
                        .url(github_link_for_issue(footer.value, &repo_info))
                        .inlined()
                        .build();
                    additions.push(format!(", closes {}", issue_link));
                }
                _ => {}
            }
        }

        if !additions.is_empty() {
            let joined_additions = format!(" {}", additions.join(" "));
            item.push_str(&joined_additions);
        }

        list.add(Box::new(item));
    }

    Ok(list)
}

pub fn generate_changelog(repo_dir: impl AsRef<Path>) -> Result<Markdown, Box<dyn StdError>> {
    let repo_dir_path = repo_dir.as_ref();
    let repo = Repository::open(repo_dir_path);
    if let Err(e) = repo {
        return Err(Box::new(Error::NoGitRepository {
            path: repo_dir_path.display().to_string(),
            source: e,
        }));
    };
    let repo = repo.unwrap();

    // Get a list of all git tags inside the repository that match the `vX.X.X` pattern.
    let tag_names = repo.tag_names(Some("v*"))?;
    // Map those tags to valid semantic versions by stripping the prefix `v`. The tuple contains the original tag alongside the semantic version.
    let mut tag_to_semver_mapped: Vec<(&str, Version)> = tag_names
        .iter()
        .flatten()
        .map(|v: &str| {
            let trimmed = v.trim_start_matches('v');
            (v, trimmed)
        })
        .map::<Result<(&str, Version), SemVerError>, _>(|(tag, semver)| {
            Ok((tag, Version::parse(semver)?))
        })
        .flatten()
        .collect::<Vec<_>>();

    // Sort the above vector by their semantic version, thus re-ordering the git tags by their version instead of their creation date.
    tag_to_semver_mapped.sort_by(|a, b| {
        if a.0 == "HEAD" {
            return Ordering::Greater;
        }
        a.1.cmp(&b.1)
    });
    // Create a new vector that only contains the git tag name references.
    let tag_names = tag_to_semver_mapped
        .iter()
        .map(|(tag, _)| tag)
        .collect::<Vec<_>>();
    // Extract the latest git tag, which will be at the end of the sorted vector.
    let latest_tag = tag_names[tag_names.len() - 1];

    // Create pairs of git tags, from a lower version to a higher version. This allows to create git diffs between tags later on.`
    let tag_range_pairs = pairwise(tag_names)
        .chain(std::iter::once((latest_tag, &"HEAD")))
        .collect::<Vec<_>>();

    // Retrieve the needed metadata for the changelog from an applicable extractor implementation.
    let mut extractors = Vec::new();
    for extractor in inventory::iter::<&dyn Extractor> {
        extractors.push(extractor);
    }
    // Sort them by their priority.
    extractors.sort_by_key(|e| e.priority());
    // TODO: proper error handling.
    let mut repo_info = None;
    for extractor in extractors {
        if extractor.is_applicable(repo_dir_path) {
            repo_info = match extractor.extract_repo_information(repo_dir_path) {
                Ok(info) => Some(info),
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    None
                }
            };
        }
    }
    let repo_info = repo_info.expect("failed to retrieve repo info");

    // Create a new Markdown file.
    let mut changelog = Markdown::new();

    // Add the header and notice at the top of the file.
    changelog.header1("Changelog");
    //let mut remark = (&*PRELIMINARY_REMARK).to_vec();
    for line in &*PRELIMINARY_REMARK {
        changelog.paragraph(line.render());
    }
    //changelog.elements.append(&mut remark);

    // For each git tag pair...
    for (from, to) in tag_range_pairs.into_iter().rev() {
        // Create the link that can be used to get a diff view of the tag range.
        let diff_link = github_link_for_range(from, to, &repo_info);
        let diff_link = Link::builder().text(*to).url(diff_link).inlined().build();
        let mut rendered_diff_link = diff_link.render();

        // Extract the date of the git tag which will then be displayed next to the version header.
        let to_tag_ref = repo.resolve_reference_from_short_name(to)?;
        // The branch is needed as the last `to` will contain `HEAD` and resolve to a branch commit.
        let to_tag_date_string = if to_tag_ref.is_tag() {
            let tag = to_tag_ref.peel_to_commit()?;

            // Create a date string based on UTC.
            let time = tag.time();
            let time = NaiveDateTime::from_timestamp(time.seconds(), 0);
            let time: DateTime<Utc> = DateTime::from_utc(time, Utc);

            time.format("%Y-%m-%d").to_string()
        } else if to_tag_ref.is_branch() && *to == "HEAD" {
            // In the case that the `to` points to the current HEAD, the current date will be returned.
            let time: DateTime<Utc> = Utc::now();
            time.format("%Y-%m-%d").to_string()
        //rendered_diff_link.push_str(&format!(" ({})", in_string));
        } else {
            String::with_capacity(0)
        };
        if !rendered_diff_link.is_empty() {
            rendered_diff_link.push_str(&format!(" ({})", to_tag_date_string));
        }

        // Calculate the level of the version header.
        let stripped = to.trim_start_matches('v');
        let level: usize = if stripped == "HEAD" {
            1
        } else {
            let version = Version::parse(&stripped)?;
            if version.patch == 0 {
                1
            } else {
                2
            }
        };
        changelog.header(rendered_diff_link, level);

        // Get all commits between the current two tags.
        let rev = format!("{}..{}", from, to);
        let commits = conventional_commits_next_semver::git_commits_in_range(&repo, &rev)?;
        let commits = commits
            .into_iter()
            .map(|oid| repo.find_commit(oid))
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        // Map all commits to a tuple of (GitCommit, ParsedCommit).
        let parsed_messages = commits
            .iter()
            .map(|c| {
                if let Some(msg) = c.message() {
                    let parsed_msg = conventional_commits_parser::parse_commit_msg(msg);
                    if let Ok(parsed_msg) = parsed_msg {
                        return Some((c, parsed_msg));
                    }
                }

                None
            })
            .filter_map(|t| t)
            .collect::<Vec<_>>();

        // Partition to get a list of bug fixes and feature related commits.
        let (fixes, other): (Vec<_>, Vec<_>) = parsed_messages
            .iter()
            .partition(|(_, parsed)| parsed.ty == "fix");
        let (features, _other): (
            Vec<&(&git2::Commit, conventional_commits_parser::Commit)>,
            Vec<&(&git2::Commit, conventional_commits_parser::Commit)>,
        ) = other.iter().partition(|(_, parsed)| parsed.ty == "feat");

        if !fixes.is_empty() {
            changelog.header3("Bug Fixes");
            changelog.list(populate_changelog_from_commits(fixes, &repo_info)?);
        }
        if !features.is_empty() {
            changelog.header3("Features");
            changelog.list(populate_changelog_from_commits(features, &repo_info)?);
        }
    }

    Ok(changelog)
}
