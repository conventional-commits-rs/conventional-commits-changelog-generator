use crate::extractors::RepoInformation;
use std::str::FromStr;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum UrlExtractionError {
    #[error("failed to parse url")]
    ParseError,
    /// Thrown if the host is unknown and the information can't be extracted.
    #[error("unsupported host: {host:?}")]
    UnsupportedHost { host: String },
}

pub fn extract_repo_from_url(url: &str) -> Result<RepoInformation, Box<dyn std::error::Error>> {
    let url = Url::from_str(url)?;
    let host_str = url.host_str();
    if let Some(host) = host_str {
        match host {
            "github.com" => {
                if let Some(mut path_segments) = url.path_segments() {
                    let owner = path_segments.next();
                    let repo = path_segments.next();
                    return match (owner, repo) {
                        (Some(owner), Some(repo)) => {
                            let repo = if let Some(repo) = repo.strip_suffix(".git") {
                                repo
                            } else {
                                repo
                            }
                            .to_string();
                            Ok(RepoInformation {
                                owner: owner.to_string(),
                                repo,
                            })
                        }
                        _ => Err(Box::new(UrlExtractionError::ParseError)),
                    };
                }
            }
            _ => {
                return Err(Box::new(UrlExtractionError::UnsupportedHost {
                    host: host.to_string(),
                }));
            }
        }
    }

    Err(Box::new(UrlExtractionError::ParseError))
}
