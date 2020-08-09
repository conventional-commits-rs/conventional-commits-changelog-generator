use conventional_commits_changelog_generator::generate_changelog;
use git2::Repository;
use indoc::printdoc;
use pico_args::Arguments;
use std::env::current_dir;
use std::fs;
use std::process::exit;

const BINARY_NAME: &str = "changelog";

/// The program's arguments.
struct Args {
    /// True if the help screen should be displayed.
    help: bool,
    /// True if the tool's version should be displayed.
    version: bool,
}

fn print_help() {
    printdoc! {"
        {crate_name} {crate_version}
        {crate_authors}
        {crate_description}

        USAGE:
            {crate_name}

        FLAGS:
            -h,--help       Prints help information
            -V,--version    Prints version information",
        crate_name = BINARY_NAME,
        crate_version = env!("CARGO_PKG_VERSION"),
        crate_authors = env!("CARGO_PKG_AUTHORS"),
        crate_description = env!("CARGO_PKG_DESCRIPTION"),
    };
}

fn print_version() {
    println!("{} v{}", BINARY_NAME, env!("CARGO_PKG_VERSION"));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Arguments::from_env();
    let args = Args {
        help: args.contains(["-h", "--help"]),
        version: args.contains(["-V", "--version"]),
    };

    if args.version {
        print_version();
    } else if args.help {
        print_help();
    } else {
        let current_folder = current_dir()?;
        match Repository::open(&current_folder) {
            Ok(repo) => {
                let changelog = generate_changelog(&repo)?;
                let changelog_file_path = current_folder.join("CHANGELOG.md");
                fs::write(changelog_file_path, changelog.render())?;
            }
            Err(_) => {
                eprintln!("No git repository in current working directory found!");
                exit(1);
            }
        }
    }

    Ok(())
}
