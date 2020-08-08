use chrono::{DateTime, NaiveDateTime, Utc};
use git2::Repository;
use markdown_composer::{Link, MarkdownBuilder};
use semver::SemVerError;
use semver::Version;
use std::cmp::Ordering;
use std::error::Error;

fn pairwise<I, J>(right: I) -> impl Iterator<Item = (I::Item, I::Item)>
where
    I: IntoIterator<Item = J> + Clone,
    J: std::fmt::Display,
{
    let left = right.clone().into_iter();
    let right = right.into_iter().skip(1);
    left.zip(right)
}

fn github_link_between_tags(from: &str, to: &str) -> String {
    format!(
        "https://github.com/SirWindfield/cargo-create/compare/{}...{}",
        from, to
    )
}

fn github_link_for_commit(hash: &str) -> String {
    format!(
        "https://github.com/SirWindfield/cargo-create/commit/{}",
        hash
    )
}

pub fn generate_changelog(repo: &Repository) -> Result<String, Box<dyn Error>> {
    // Walk between each version and collect the commits to analyse.
    let tag_names = repo.tag_names(Some("v*"))?;
    // Map tags to semver versions.
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

    // Sort by semver specification.
    tag_to_semver_mapped.sort_by(|a, b| {
        if a.0 == "HEAD" {
            return Ordering::Greater;
        }
        a.1.cmp(&b.1)
    });
    let tag_names = tag_to_semver_mapped
        .iter()
        .map(|(tag, _)| tag)
        .collect::<Vec<_>>();
    let latest_tag = tag_names[tag_names.len() - 1];

    let tag_range_pairs = pairwise(tag_names)
        .chain(std::iter::once((latest_tag, &"HEAD")))
        .collect::<Vec<_>>();
    let mut changelog = MarkdownBuilder::new();

    for (from, to) in tag_range_pairs.into_iter().rev() {
        let diff_link = github_link_between_tags(from, to);
        let diff_link = Link::new(*to, diff_link);
        let mut rendered_diff_link = format!("{}", diff_link);
        let to_date = repo.resolve_reference_from_short_name(to)?;
        let date = if to_date.is_tag() {
            let tag = to_date.peel_to_commit()?;
            let time = tag.time();
            let time = NaiveDateTime::from_timestamp(time.seconds(), 0);
            let time: DateTime<Utc> = DateTime::from_utc(time, Utc);
            let in_string = time.format("%Y-%m-%d");
            rendered_diff_link.push_str(&format!(" ({})", in_string));
        } else if to_date.is_branch() && *to == "HEAD" {
            // Just use the current date and time.
            let time: DateTime<Utc> = Utc::now();
            let in_string = time.format("%Y-%m-%d");
            rendered_diff_link.push_str(&format!(" ({})", in_string));
        };
        let stripped = to.trim_start_matches('v');
        println!("{}", stripped);
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
        changelog.header(&rendered_diff_link, level);

        let rev = format!("{}..{}", from, to);
        let commits = conventional_commits_next_semver::git_commits_in_range(repo, &rev)?;
        let commits = commits
            .into_iter()
            .map(|oid| repo.find_commit(oid))
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        let parsed_messages = commits
            .iter()
            .map(|c| c.message())
            .filter_map(|msg| msg)
            .map(|msg| conventional_commits_parser::parse_commit_msg(msg))
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        let (fixes, other): (
            Vec<&conventional_commits_parser::Commit>,
            Vec<&conventional_commits_parser::Commit>,
        ) = parsed_messages
            .iter()
            .partition(|parsed| parsed.ty == "fix");
        let (features, other): (
            Vec<&conventional_commits_parser::Commit>,
            Vec<&conventional_commits_parser::Commit>,
        ) = other.iter().partition(|parsed| parsed.ty == "feat");

        // Add all bug fixes.
        if !fixes.is_empty() {
            let mut list = markdown_composer::List::new();
            for p in fixes {
                let item = match p.scope {
                    Some(scope) => format!("{}: {}", scope, p.desc),
                    None => p.desc.to_string(),
                };
                list.add(item);
            }
            list.unordered();

            changelog.header3("Bug Fixes");
            changelog.list(list);
        }

        // Followed by all new features.
        if !features.is_empty() {
            let mut list = markdown_composer::List::new();
            for p in features {
                let item = match p.scope {
                    Some(scope) => format!("{}: {}", scope, p.desc),
                    None => p.desc.to_string(),
                };
                list.add(item);
            }
            list.unordered();

            changelog.header3("Features");
            changelog.list(list);
        }
    }

    println!("{}", changelog);

    // // Walk over every commit of the current HEAD.
    // let mut revwalk = repo.revwalk()?;
    // revwalk.push_head()?;

    // // TODO: can we pre-allocate the vec already?
    // let mut commits_to_consider = Vec::new();
    // for res in revwalk {
    //     let oid = res?;
    //     let commit = repo.find_commit(oid)?;
    //     commits_to_consider.push(commit);
    // }

    Ok("".into())
}
