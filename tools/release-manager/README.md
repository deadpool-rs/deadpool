# release-manager

A small release helper for the deadpool crates. It inspects each crate under
`crates/` and reports what has changed since the last release, so you can see at
a glance which crates need a new version cut.

> **Note:** This tool was generated entirely with AI.

## What it does

For every crate under `crates/` (each identified by its `Cargo.toml`), the tool:

1. Reads the crate's `CHANGELOG.md` to find the most recent released version and
   any items currently under the `## [Unreleased]` heading.
2. Looks for the matching git tag (`<package-name>-v<version>`).
3. If the tag exists, lists the git commits that touched the crate's directory
   since that tag (`<tag>..HEAD`).

This makes it easy to spot crates that have unreleased changelog entries and/or
commits landed since their last tagged release.

## Building

```sh
cargo build --release
```

The binary is `release-manager` (not published to crates.io — `publish = false`).

## Usage

Run from the repository root (the tool discovers crates under `./crates`):

```sh
release-manager status
```

`status` is the default command, so plain `release-manager` behaves the same.

### Options

| Flag | Description |
| --- | --- |
| `--filter <VALUE>` | Only show crates whose package name contains `<VALUE>`. |
| `--all` | Show all crates, including those with no unreleased changes. By default only crates with unreleased changelog entries or commits since the last release are shown. |
| `--table` | Print a concise single-line-per-crate table instead of the detailed view. |
| `--repo-root <PATH>` | Repository root to operate on (defaults to the current directory). |

### Examples

```sh
# Only crates that have something to release (the default)
release-manager status

# Detailed status for every crate, including unchanged ones
release-manager status --all

# Compact table, limited to postgres-related crates
release-manager status --filter postgres --table

# Operate on a checkout in another directory
release-manager status --repo-root ../deadpool
```

## Assumptions

- Crates live under `crates/<name>/` and each has a `Cargo.toml` and a
  `CHANGELOG.md`.
- Changelogs follow the [Keep a Changelog](https://keepachangelog.com/) layout:
  an `## [Unreleased]` section followed by `## [<version>]` release headings, with
  changes listed as `- ` bullet points.
- Release tags are named `<package-name>-v<version>` (e.g. `deadpool-v0.10.0`).
- `git` is available on `PATH`.

If a crate's release tag cannot be found, or the last release cannot be
determined from the changelog, the crate is reported with a warning note rather
than failing the whole run.
