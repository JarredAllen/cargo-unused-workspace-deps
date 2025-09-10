#![doc = include_str!("../README.md")]

use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[doc = include_str!("../README.md")]
#[derive(Parser)]
#[command(bin_name = "cargo")]
#[command(author, version, about, long_about=None)]
struct Arguments {
    /// When run as a cargo subcommand, it provides `unused-workspace-deps` as the first argument
    #[command(subcommand)]
    command: Command,
}

/// When run as a cargo subcommand, it provides `unused-workspace-deps` as the first argument
#[derive(clap::Subcommand)]
enum Command {
    /// Check for unused dependencies listed in a workspace-level `Cargo.toml` file.
    UnusedWorkspaceDeps(UnusedWorkspaceDepsArguments),
}

#[derive(clap::Args)]
struct UnusedWorkspaceDepsArguments {
    /// The path to the workspace you want to lint.
    ///
    /// Defaults to the current working directory.
    workspace_path: Option<PathBuf>,

    /// The path to the `cargo` executable to run.
    ///
    /// Defaults to the value of the `$CARGO` environment variable, or if that isn't set, falls
    /// back to `cargo` and lets the system look it up on `$PATH`.
    #[arg(long)]
    cargo_path: Option<PathBuf>,
    /// Remove unused dependencies instead of reporting an error.
    #[arg(short, long)]
    fix: bool,
}

fn main() -> ExitCode {
    let args = Arguments::parse();
    let Command::UnusedWorkspaceDeps(args) = args.command;
    let mut metadata_command = cargo_metadata::MetadataCommand::new();
    metadata_command.no_deps();
    if let Some(path) = args.workspace_path.as_ref() {
        metadata_command.manifest_path(path);
    }
    if let Some(path) = args.cargo_path {
        metadata_command.cargo_path(path);
    }

    let mut workspace_deps =
        cargo_unused_workspace_deps::read_workspace_deps(args.workspace_path.as_ref());
    for package in metadata_command.exec().expect("Failed to run").packages {
        let manifest_path = package.manifest_path.into_std_path_buf();
        for dep in cargo_unused_workspace_deps::list_package_workspace_deps(manifest_path) {
            let _ = workspace_deps.remove(&dep);
        }
    }

    if workspace_deps.is_empty() {
        println!("All workspace dependencies are in use!");
        ExitCode::SUCCESS
    } else {
        println!("Remaining workspace dependencies:");
        let mut unused_dependencies: Vec<String> = workspace_deps.into_keys().collect();
        unused_dependencies.sort_unstable();
        for unused_dep in &unused_dependencies {
            println!("\t{unused_dep}");
        }
        if args.fix {
            println!("Removing unused dependencies from manifest because `--fix` was given");
            cargo_unused_workspace_deps::remove_deps_from_workspace(
                args.workspace_path
                    .unwrap_or(std::path::Path::new("./Cargo.toml").to_owned()),
                &unused_dependencies,
            );
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    }
}
