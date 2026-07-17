// NOTE: This tool was generated entirely with AI. Full disclosure.

use clap::{Parser, Subcommand};
use console::style;
use serde::Deserialize;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "release-manager")]
#[command(about = "Release helper for deadpool crates")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show release status for crates under ./crates
    Status {
        /// Only show crates whose package name contains this value
        #[arg(long)]
        filter: Option<String>,

        /// Show all crates, including those with no unreleased changes
        #[arg(long)]
        all: bool,

        /// Print concise single-line table output
        #[arg(long)]
        table: bool,

        /// Repository root (defaults to current working directory)
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },
}

#[derive(Debug)]
struct CrateStatus {
    package_name: String,
    relative_path: PathBuf,
    last_release: Option<String>,
    unreleased_items: Vec<String>,
    commits_since_release: Vec<String>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
}

fn main() {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Status {
        filter: None,
        all: false,
        table: false,
        repo_root: PathBuf::from("."),
    });

    let result = match command {
        Commands::Status {
            filter,
            all,
            table,
            repo_root,
        } => run_status(&repo_root, filter.as_deref(), !all, table),
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}

fn run_status(
    repo_root: &Path,
    filter: Option<&str>,
    only_changed: bool,
    table: bool,
) -> Result<(), String> {
    let crates =
        discover_crates(repo_root).map_err(|e| format!("failed to discover crates: {e}"))?;

    let mut statuses = Vec::new();
    for (package_name, relative_path) in crates {
        if let Some(f) = filter {
            if !package_name.contains(f) {
                continue;
            }
        }

        let status = build_crate_status(repo_root, &package_name, &relative_path)
            .map_err(|e| format!("{} ({}): {e}", package_name, relative_path.display()))?;
        statuses.push(status);
    }

    statuses.sort_by(|a, b| a.package_name.cmp(&b.package_name));
    if only_changed {
        statuses.retain(|status| {
            !status.unreleased_items.is_empty() || !status.commits_since_release.is_empty()
        });
    }

    if table {
        print_table(&statuses);
    } else {
        for (idx, status) in statuses.iter().enumerate() {
            if idx > 0 {
                println!();
            }
            print_status(status);
        }
    }

    Ok(())
}

fn discover_crates(repo_root: &Path) -> io::Result<Vec<(String, PathBuf)>> {
    let crates_dir = repo_root.join("crates");
    let mut crates = Vec::new();

    for entry in fs::read_dir(crates_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let cargo_toml_path = path.join("Cargo.toml");
        if !cargo_toml_path.is_file() {
            continue;
        }

        let content = fs::read_to_string(&cargo_toml_path)?;
        let parsed: CargoToml = toml::from_str(&content).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {err}", cargo_toml_path.display()),
            )
        })?;

        let relative = path
            .strip_prefix(repo_root)
            .unwrap_or(path.as_path())
            .to_path_buf();

        crates.push((parsed.package.name, relative));
    }

    Ok(crates)
}

fn build_crate_status(
    repo_root: &Path,
    package_name: &str,
    relative_path: &Path,
) -> Result<CrateStatus, String> {
    let changelog = repo_root.join(relative_path).join("CHANGELOG.md");
    let changelog_content = fs::read_to_string(&changelog)
        .map_err(|e| format!("failed to read {}: {e}", changelog.display()))?;

    let (last_release, unreleased_items) = parse_changelog(&changelog_content);

    let (commits_since_release, note) = if let Some(version) = &last_release {
        let tag = format!("{package_name}-v{version}");
        if git_tag_exists(repo_root, &tag)? {
            let commits = git_commits_since_tag(repo_root, &tag, relative_path)?;
            (commits, None)
        } else {
            (
                Vec::new(),
                Some(format!("tag '{tag}' was not found in this repository")),
            )
        }
    } else {
        (
            Vec::new(),
            Some("could not determine last release from CHANGELOG.md".to_string()),
        )
    };

    Ok(CrateStatus {
        package_name: package_name.to_string(),
        relative_path: relative_path.to_path_buf(),
        last_release,
        unreleased_items,
        commits_since_release,
        note,
    })
}

fn parse_changelog(changelog: &str) -> (Option<String>, Vec<String>) {
    let mut in_unreleased = false;
    let mut unreleased_items = Vec::new();
    let mut last_release = None;

    for line in changelog.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("## [") {
            if trimmed == "## [Unreleased]" {
                in_unreleased = true;
                continue;
            }

            if in_unreleased && last_release.is_none() {
                last_release = extract_version_heading(trimmed);
                in_unreleased = false;
            } else {
                in_unreleased = false;
            }
        }

        if in_unreleased {
            if let Some(item) = extract_bullet(trimmed) {
                unreleased_items.push(item.to_string());
            }
        }
    }

    (last_release, unreleased_items)
}

fn extract_version_heading(line: &str) -> Option<String> {
    let start = line.find("## [")? + 4;
    let end = line[start..].find(']')? + start;
    let value = &line[start..end];
    if value == "Unreleased" {
        None
    } else {
        Some(value.to_string())
    }
}

fn extract_bullet(line: &str) -> Option<&str> {
    line.strip_prefix("- ")
}

fn git_tag_exists(repo_root: &Path, tag: &str) -> Result<bool, String> {
    let output = Command::new("git")
        .arg("tag")
        .arg("--list")
        .arg(tag)
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("failed to run git tag: {e}"))?;

    if !output.status.success() {
        return Err("git tag --list failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().any(|line| line == tag))
}

fn git_commits_since_tag(
    repo_root: &Path,
    tag: &str,
    relative_path: &Path,
) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("log")
        .arg("--pretty=format:%h %s")
        .arg(format!("{tag}..HEAD"))
        .arg("--")
        .arg(relative_path)
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("failed to run git log: {e}"))?;

    if !output.status.success() {
        return Err(format!("git log failed for {}", relative_path.display()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    Ok(commits)
}

fn print_status(status: &CrateStatus) {
    println!("{}", style(&status.package_name).yellow());
    println!();

    let last_release = status
        .last_release
        .as_deref()
        .unwrap_or("<unknown>")
        .to_string();

    println!("{} {}", style("Last release:").bold(), last_release);
    println!(
        "{} {}",
        style("Git commits since last release:").bold(),
        status.commits_since_release.len()
    );
    println!("{}", style("Unreleased changelog:").bold());

    if status.unreleased_items.is_empty() {
        println!("- <none>");
    } else {
        for item in &status.unreleased_items {
            println!("- {}", item);
        }
    }

    if !status.commits_since_release.is_empty() {
        println!();
        println!("{}", style("Commits:").bold());
        for commit in &status.commits_since_release {
            println!("- {}", commit);
        }
    }

    if let Some(note) = &status.note {
        println!();
        println!("{} {}", style("Note:").bold(), note);
    }

    if !status.relative_path.as_os_str().eq(OsStr::new("")) {
        println!();
        println!(
            "{} {}",
            style("Path:").bold(),
            status.relative_path.display()
        );
    }
}

fn print_table(statuses: &[CrateStatus]) {
    let name_width = statuses
        .iter()
        .map(|s| s.package_name.len())
        .max()
        .unwrap_or(5)
        .max(5);

    println!(
        "{:<name_width$}  {:<12}  {:>7}  {:>10}  {}",
        style("crate").bold(),
        style("last").bold(),
        style("commits").bold(),
        style("unreleased").bold(),
        style("status").bold(),
        name_width = name_width
    );

    for status in statuses {
        let last_release = status.last_release.as_deref().unwrap_or("<unknown>");
        let commits = status.commits_since_release.len();
        let unreleased = status.unreleased_items.len();
        let status_text = if status.note.is_some() {
            style("warning").red().to_string()
        } else if commits > 0 || unreleased > 0 {
            style("changed").yellow().to_string()
        } else {
            style("clean").green().to_string()
        };

        println!(
            "{:<name_width$}  {:<12}  {:>7}  {:>10}  {}",
            style(&status.package_name).yellow(),
            last_release,
            commits,
            unreleased,
            status_text,
            name_width = name_width
        );
    }
}
