use std::{fmt::Display, io::Write, str::FromStr};

use flat_error::FlatError;

use crate::{error::Error, model::*};

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SortOrder {
    /// Sort in ascending order
    Ascending,
    /// Sort in descending order
    Descending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArgumentDisplay {
    Hide,
    Collapse,
    Expand,
}

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

pub fn print_as_tree(
    rustfile: &Rustfile,
    sort_order: Option<SortOrder>,
    arguments: ArgumentDisplay,
) -> Result<(), Error> {
    let mut stdout = std::io::stdout().lock();
    write_as_tree(rustfile, sort_order, arguments, &mut stdout)?;
    writeln!(&mut stdout)?;
    Ok(())
}

pub fn write_as_tree<W: Write>(
    rustfile: &Rustfile,
    sort_order: Option<SortOrder>,
    arguments: ArgumentDisplay,
    w: &mut W,
) -> Result<(), Error> {
    let mut entries: Vec<_> = rustfile.iter().collect();

    if let Some(order) = sort_order {
        match order {
            SortOrder::Ascending => entries.sort_by_key(|e| e.name()),
            SortOrder::Descending => entries.sort_by_key(|e| std::cmp::Reverse(e.name())),
        }
    }

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let prefix = if is_last { "└── " } else { "├── " };

        match arguments {
            ArgumentDisplay::Hide => {
                writeln!(w, "{}{} {}", prefix, entry.kind(), entry.name())?;
            }
            ArgumentDisplay::Collapse => {
                let args = entry.arguments();
                if args.to_string().trim().is_empty() {
                    writeln!(w, "{}{} {}", prefix, entry.kind(), entry.name())?;
                } else {
                    writeln!(w, "{}{} {} [...]", prefix, entry.kind(), entry.name())?;
                }
            }
            ArgumentDisplay::Expand => {
                writeln!(
                    w,
                    "{}{} {}{}",
                    prefix,
                    entry.kind(),
                    entry.name(),
                    entry.arguments()
                )?;
            }
        }
    }

    Ok(())
}

pub fn print_as_table(
    rustfile: &Rustfile,
    sort_order: Option<SortOrder>,
    arguments: ArgumentDisplay,
) -> Result<(), Error> {
    let mut stdout = std::io::stdout().lock();
    write_as_table(rustfile, sort_order, arguments, &mut stdout)?;
    writeln!(&mut stdout)?;
    Ok(())
}

pub fn write_as_table<W: Write>(
    rustfile: &Rustfile,
    sort_order: Option<SortOrder>,
    arguments: ArgumentDisplay,
    w: &mut W,
) -> Result<(), Error> {
    let mut entries: Vec<_> = rustfile.iter().collect();

    if let Some(order) = sort_order {
        match order {
            SortOrder::Ascending => entries.sort_by_key(|e| e.name()),
            SortOrder::Descending => entries.sort_by_key(|e| std::cmp::Reverse(e.name())),
        }
    }

    // Calculate column widths
    let kind_width = entries
        .iter()
        .map(|e| e.kind().to_string().len())
        .max()
        .unwrap_or(4);
    let name_width = entries
        .iter()
        .map(|e| e.name().to_string().len())
        .max()
        .unwrap_or(4);

    // Print header
    writeln!(
        w,
        "{:<kind_width$}  {:<name_width$}  Arguments",
        "Kind",
        "Name",
        kind_width = kind_width,
        name_width = name_width
    )?;
    writeln!(
        w,
        "{:-<kind_width$}  {:-<name_width$}  {:-<9}",
        "",
        "",
        "",
        kind_width = kind_width,
        name_width = name_width
    )?;

    // Print entries
    for entry in entries {
        let args_str = match arguments {
            ArgumentDisplay::Hide => String::new(),
            ArgumentDisplay::Collapse => {
                let args = entry.arguments();
                if args.to_string().trim().is_empty() {
                    String::new()
                } else {
                    "[...]".to_string()
                }
            }
            ArgumentDisplay::Expand => entry.arguments().to_string(),
        };

        writeln!(
            w,
            "{:<kind_width$}  {:<name_width$}  {}",
            entry.kind(),
            entry.name(),
            args_str,
            kind_width = kind_width,
            name_width = name_width
        )?;
    }

    Ok(())
}

// ------------------------------------------------------------------------------------------------
// Other Implementations
// ------------------------------------------------------------------------------------------------

impl Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ascending => "asc[ending]",
                Self::Descending => "desc[ending]",
            }
        )
    }
}

impl FromStr for SortOrder {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" | "ascending" => Ok(Self::Ascending),
            "desc" | "descending" => Ok(Self::Descending),
            _ => Err(Error::ValueParserError {
                type_name: "SortOrder",
                source: FlatError::new(
                    "SortOrder",
                    format!(
                        "The string value {s:?} is not a valid representation for type `SortOrder`."
                    ),
                ),
            }),
        }
    }
}
