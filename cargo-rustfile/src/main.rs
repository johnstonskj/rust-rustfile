use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;
use rustfile::{error::Error, model::EntryKind, parser::parse_str, prelude::*};
use std::{env, path::PathBuf, process::ExitCode};
use tracing::{error, level_filters::LevelFilter, trace};
use tracing_subscriber::filter::EnvFilter;

// use clap_complete::aot::{generate, Generator, Shell};

// ------------------------------------------------------------------------------------------------
// Command-Line Structure
// ------------------------------------------------------------------------------------------------

const COMMAND_NAME: &str = env!("CARGO_BIN_NAME");

pub trait Command {
    type Output;
    type Error: std::error::Error; // std::convert::Infallible

    fn execute(&self) -> Result<Self::Output, Self::Error>;
}

pub trait ParameterizedCommand {
    type Parameters;
    type Output;
    type Error: std::error::Error;

    fn with_parameters(self, parameters: Self::Parameters) -> Self;

    fn execute(&self) -> Result<Self::Output, Self::Error>;
}

#[derive(Debug, Parser)]
#[command(version, name = COMMAND_NAME, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Find the location (if any) of your default Rustfile
    Find,

    /// Display, filter, sort,1 and organize entries in the Rustfile
    Display(DisplayCommand),

    /// Install all entries in the Rustfile
    Install(InstallCommand),

    /// Upgrade all entries in the Rustfile
    Upgrade(UpgradeCommand),

    /// Uninstall all entries in the Rustfile
    Uninstall(UninstallCommand),
    //    /// Generate shell completions
    //    Completions(Shell),
}

#[derive(Debug, Parser)]
struct DisplayCommand {
    #[structopt(flatten)]
    shared: SharedCommandArgs,

    /// Sort order
    #[arg(value_enum)]
    sort: Option<SortOrder>,

    /// Display format
    #[arg(value_enum)]
    display_format: Option<DisplayFormat>,
}

#[derive(Debug, Parser)]
struct InstallCommand {
    #[structopt(flatten)]
    shared: SharedCommandArgs,
}

#[derive(Debug, Parser)]
struct UpgradeCommand {
    #[structopt(flatten)]
    shared: SharedCommandArgs,
}

#[derive(Debug, Parser)]
struct UninstallCommand {
    #[structopt(flatten)]
    shared: SharedCommandArgs,
}

#[derive(Debug, Parser)]
struct SharedCommandArgs {
    ///
    /// The path to the Rustfile to use, overriding default locations.
    ///
    /// The file `Rustfile` is looked for in the following locations, in order:
    ///
    /// 1. the `--file` argument, if present,
    /// 2. the value of the `CARGO_RUSTFILE` environment variable, if present,
    /// 3. in the directory `${XDG_CONFIG_HOME}/cargo`, if the XDG variable is present,
    /// 4. in the directory `${HOME}/.config/cargo`, which should be equivalent to the previous,
    /// 5. in the current directory.
    ///
    #[arg(long, short = 'f', env = "CARGO_RUSTFILE")]
    file: Option<PathBuf>,

    ///
    /// Only install entries that match this kind
    ///
    #[arg(long, short = 'k')]
    kind: Option<EntryKind>,

    ///
    /// Only install entries whose name match this name pattern (regex)
    ///
    #[arg(long, short = 'n')]
    name_matches: Option<Regex>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ValueEnum)]
enum SortOrder {
    /// Do not sort
    None,
    /// Sort in ascending order
    Ascending,
    /// Sort in descending order
    Descending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ValueEnum)]
enum DisplayFormat {
    Flat,
    FlatNoArguments,
    Tree,
    TreeNoArguments,
    TreeArgumentsAsNodes,
    Table,
    TableNoArguments,
    TableArgumentsAsCells,
}

// ------------------------------------------------------------------------------------------------
// Main Function
// ------------------------------------------------------------------------------------------------

fn main() -> Result<ExitCode, Error> {
    Cli::parse().execute()
}

// ------------------------------------------------------------------------------------------------
// Command Implementations
// ------------------------------------------------------------------------------------------------

impl Command for Cli {
    type Output = ExitCode;
    type Error = Error;

    fn execute(&self) -> Result<Self::Output, Self::Error> {
        println!("VERBOSITY::{:?}", self.verbosity);
        init_tracing(self.verbosity, Some("rustfile"))?;
        self.cmd.execute()
    }
}

impl Command for Commands {
    type Output = ExitCode;
    type Error = Error;

    fn execute(&self) -> Result<Self::Output, Self::Error> {
        match self {
            Self::Find => {
                if let Some(rustfile_path) = Rustfile::find_rustfile() {
                    println!(
                        "Your default Rustfile is `{}`",
                        rustfile_path.display()
                    );
                } else {
                    println!("No Rustfile found in any default location.");
                }
            }
            Self::Display(cmd) => {
                let _rustfile = rustfile_from_args(&cmd.shared)?;
            }
            Self::Install(cmd) => {
                let rustfile = rustfile_from_args(&cmd.shared)?;
                rustfile.install_all()?;
            }
            Self::Upgrade(cmd) => {
                let rustfile = rustfile_from_args(&cmd.shared)?;
                rustfile.upgrade_all()?;
            }
            Self::Uninstall(cmd) => {
                let rustfile = rustfile_from_args(&cmd.shared)?;
                rustfile.uninstall_all()?;
            } // Self::Completions(shell) => {
              //
              // }
        }
        Ok(ExitCode::SUCCESS)
    }
}

// ------------------------------------------------------------------------------------------------
// Private Functions
// ------------------------------------------------------------------------------------------------

fn init_tracing(
    log_level: clap_verbosity_flag::Verbosity,
    library_root: Option<&str>,
) -> Result<(), Error> {
    let filter = EnvFilter::from_default_env().add_directive(
        format!("{}={}", module_path!(), log_level)
            .parse()
            .map_err(|e| {
                error!("Error parsing trace env-filter expression; source: {e}");
                Error::from(e)
            })?,
    );
    let filter = if let Some(library_root) = library_root {
        filter.add_directive(
            format!("{library_root}={}", log_level)
                .parse()
                .map_err(|e| {
                    error!("Error parsing trace env-filter expression; source: {e}");
                    Error::from(e)
                })?,
        )
    } else {
        filter
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_max_level(LevelFilter::TRACE)
        .with_level(true)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true)
        .pretty()
        .init();

    Ok(())
}

fn rustfile_from_args(args: &SharedCommandArgs) -> Result<Rustfile, Error> {
    trace!(
        "looking for Rustfile; argument supplied on command line: {:?}",
        args.file
    );
    let file_path = if let Some(file_path) = args.file.clone() {
        file_path
    } else if let Some(file_path) = Rustfile::find_rustfile() {
        file_path
    } else {
        error!("No Rustfile found, not much more we can do");
        return Err(std::io::ErrorKind::NotFound.into());
    };

    let file_name = file_path.display().to_string();
    let content = std::fs::read_to_string(file_path)?;
    let rustfile = parse_str(&content, file_name.as_str())?;

    Ok(if args.kind.is_some() || args.name_matches.is_some() {
        trace!(
            "filtering the loaded file using kind: {:?}, name: {:?}",
            args.kind, args.name_matches
        );
        rustfile.filter(&args.kind, &args.name_matches)
    } else {
        rustfile
    })
}
