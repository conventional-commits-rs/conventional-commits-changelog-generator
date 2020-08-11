use crate::extractors::{
    utils::extract_repo_from_url, Extractor, RepoInformation, LANGUAGE_RELATED_PRIORITY,
};
use cargo_toml::Manifest;
use std::path::Path;
use thiserror::Error;

pub const CARGO_TOML_FILE: &str = "Cargo.toml";

#[derive(Debug, Error)]
pub enum CargoError {
    #[error("no package section inside of Cargo.toml")]
    NoPackageSection,
    #[error("no repository field inside of Cargo.toml")]
    NoRepositoryField,
}

struct CargoExtractor;

impl Extractor for CargoExtractor {
    fn extract_repo_information(
        &self,
        repo_dir: &Path,
    ) -> Result<RepoInformation, Box<dyn std::error::Error>> {
        println!("cargo");
        let cargo_toml = repo_dir.join(CARGO_TOML_FILE);
        // Safety: Only runs if the file actually exists.
        let cargo_toml = Manifest::from_path(cargo_toml)?;
        match cargo_toml.package {
            Some(package_section) => match package_section.repository {
                Some(url) => {
                    return extract_repo_from_url(&url);
                }
                None => Err(Box::new(CargoError::NoRepositoryField)),
            },
            None => Err(Box::new(CargoError::NoPackageSection)),
        }
    }

    fn is_applicable(&self, repo_dir: &Path) -> bool {
        println!("cargo check");
        repo_dir.join(CARGO_TOML_FILE).exists()
    }

    fn priority(&self) -> u8 {
        LANGUAGE_RELATED_PRIORITY
    }
}

inventory::submit! {
    &CargoExtractor as &dyn Extractor
}
