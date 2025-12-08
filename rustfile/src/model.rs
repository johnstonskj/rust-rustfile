use crate::error::{Error};
use regex::Regex;
use std::{
    borrow::{Borrow}, env, fmt::{Debug, Display}, path::PathBuf, str::FromStr
};
use tracing::{error, trace};
use url::Url;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

pub trait EntryArguments: Clone + Debug + Display + Into<Arguments> + PartialEq {
    fn has_any_arguments(&self) -> bool;

    fn as_string_vec(&self) -> Vec<String>;
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier(String);

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name(Identifier);

pub type NameList = Vec<Name>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rustfile {
    entries: Vec<Entry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    line: usize,
    name: Name,
    arguments: Arguments,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    Toolchain,
    Component,
    Command,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arguments {
    Toolchain(ToolchainArguments),
    Component(ComponentArguments),
    Command(CommandArguments),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToolchainArguments {
    profile: Option<Name>,
    components: Vec<Name>,
    targets: Vec<Name>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComponentArguments {
    target: Option<Name>,
    toolchain: Option<Name>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandArguments {
    all_features: Option<bool>,
    branch: Option<String>,
    debug: Option<bool>,
    features: Vec<Name>,
    frozen: Option<bool>,
    git: Option<Url>,
    index: Option<String>,
    locked: Option<bool>,
    no_default_features: Option<bool>,
    offline: Option<bool>,
    path: Option<String>,
    registry: Option<String>,
    rev: Option<String>,
    tag: Option<String>,
    target: Option<Name>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Url(Url),
    Name(Name),
    NameList(NameList),
}

// ------------------------------------------------------------------------------------------------
// Implementations ❱ Rustfile
// ------------------------------------------------------------------------------------------------

impl Display for Rustfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "# Rustfile {{\n{}\n# }}\n",
            self.entries
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl AsRef<Vec<Entry>> for Rustfile {
    fn as_ref(&self) -> &Vec<Entry> {
        &self.entries
    }
}

impl AsRef<[Entry]> for Rustfile {
    fn as_ref(&self) -> &[Entry] {
        self.entries.as_slice()
    }
}

impl Borrow<[Entry]> for Rustfile {
    fn borrow(&self) -> &[Entry] {
        self.entries.as_slice()
    }
}

impl From<Vec<Entry>> for Rustfile {
    fn from(entries: Vec<Entry>) -> Self {
        Rustfile { entries }
    }
}

impl From<Rustfile> for Vec<Entry> {
    fn from(value: Rustfile) -> Self {
        value.entries
    }
}

impl IntoIterator for Rustfile {
    type Item = Entry;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl FromIterator<Entry> for Rustfile {
    fn from_iter<T: IntoIterator<Item = Entry>>(iter: T) -> Self {
        Self {
            entries: Vec::from_iter(iter),
        }
    }
}

impl Rustfile {
    pub fn iter(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }

    pub fn filter(&self, kind_match: &Option<EntryKind>, name_regex: &Option<Regex>) -> Rustfile {
        Rustfile {
            entries: self
                .entries
                .iter()
                .filter(|entry| {
                    kind_match.map(|k| entry.kind() == k).unwrap_or(true)
                        && name_regex
                            .as_ref()
                            .map(|r| r.is_match(entry.name.as_str()))
                            .unwrap_or(true)
                })
                .cloned()
                .collect(),
        }
    }

    fn check_env_var_path(var_name: &str, path_add: &str) -> Option<PathBuf> {
        if let Ok(env_value) = env::var(var_name) {
            let path = PathBuf::from(env_value);
            let path = if !path_add.is_empty() {
                path.join(path_add)
            } else {
                path
            };
            trace!(
                "Checking if `{}` is_file => {}",
                path.display(),
                path.is_file()
            );
            if path.is_file() { Some(path) } else { None }
        } else {
            trace!("Environment variable ${{{var_name}}} not set");
            None
        }
    }
    
    pub fn find_rustfile() -> Option<PathBuf> {
        trace!("Checking environment variable ${{CARGO_RUSTFILE}}");
        if let Some(file_path) = Self::check_env_var_path("CARGO_RUSTFILE", "") {
            Some(file_path)
        } else {
            trace!("Checking environment variable ${{XDG_CONFIG_HOME}} with path cargo/Rustfile");
            if let Some(file_path) = Self::check_env_var_path("XDG_CONFIG_HOME", "cargo/Rustfile") {
                Some(file_path)
            } else {
                trace!("Checking environment variable ${{HOME}} with path .config/cargo/Rustfile");
                if let Some(file_path) = Self::check_env_var_path("HOME", ".config/cargo/Rustfile") {
                    Some(file_path)
                } else {
                    trace!("Final fall-back is ${{PWD}}/Rustfile");
                    let file_path = PathBuf::from("./Rustfile");
                    if file_path.is_file() {
                        Some(file_path)
                    } else {
                        None
                    }
                }
            }
        }
    }

    pub fn install_all(&self) -> Result<(), Error> {
        for entry in &self.entries {
            match entry.kind() {
                EntryKind::Toolchain => {
                    // Install toolchain using rustup
                    trace!("Installing toolchain: {}", entry.name());
                }
                EntryKind::Component => {
                    // Install component using rustup
                    trace!("Installing component: {}", entry.name());
                }
                EntryKind::Command => {
                    // Install command using cargo install
                    trace!("Installing command: {}", entry.name());
                }
            }
        }
        Ok(())
    }

    pub fn upgrade_all(&self) -> Result<(), Error> {
        for entry in &self.entries {
            match entry.kind() {
                EntryKind::Toolchain => {
                    // Update toolchain using rustup
                    trace!("Upgrading toolchain: {}", entry.name());
                }
                EntryKind::Component => {
                    // Component upgrades are typically handled with toolchain updates
                    trace!("Component upgrade handled with toolchain: {}", entry.name());
                }
                EntryKind::Command => {
                    // Upgrade command using cargo install --force
                    trace!("Upgrading command: {}", entry.name());
                }
            }
        }
        Ok(())
    }

    pub fn uninstall_all(&self) -> Result<(), Error> {
        for entry in &self.entries {
            match entry.kind() {
                EntryKind::Toolchain => {
                    // Uninstall toolchain using rustup
                    trace!("Uninstalling toolchain: {}", entry.name());
                }
                EntryKind::Component => {
                    // Uninstall component using rustup
                    trace!("Uninstalling component: {}", entry.name());
                }
                EntryKind::Command => {
                    // Uninstall command using cargo uninstall
                    trace!("Uninstalling command: {}", entry.name());
                }
            }
        }
        Ok(())
    }

    pub fn add(&self, entry: Entry) -> Result<(), Error> {
        // Check if entry already exists
        if self.entries.iter().any(|e| e.name() == entry.name() && e.kind() == entry.kind()) {
            error!("Entry already exists: {} ({})", entry.name(), entry.kind());
            return Err(Error::Unknown {
                message: format!("Entry already exists: {} ({})", entry.name(), entry.kind()),
            });
        }

        trace!("Adding entry: {} ({})", entry.name(), entry.kind());
        // In a complete implementation, this would write to the Rustfile
        Ok(())
    }

    pub fn remove(&self, entry: Entry) -> Result<(), Error> {
        // Check if entry exists
        if !self.entries.iter().any(|e| e.name() == entry.name() && e.kind() == entry.kind()) {
            error!("Entry not found: {} ({})", entry.name(), entry.kind());
            return Err(Error::Unknown {
                message: format!("Entry not found: {} ({})", entry.name(), entry.kind()),
            });
        }

        trace!("Removing entry: {} ({})", entry.name(), entry.kind());
        // In a complete implementation, this would write to the Rustfile
        Ok(())
    }
}

// ------------------------------------------------------------------------------------------------
// Implementations ❱ Entry
// ------------------------------------------------------------------------------------------------

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} '{}'{}", self.kind(), self.name(), "")
    }
}

impl Entry {
    pub fn new(line: usize, name: Name, arguments: Arguments) -> Self {
        Self {
            line,
            name,
            arguments,
        }        
    }

    pub const fn kind(&self) -> EntryKind {
        match &self.arguments {
            Arguments::Toolchain(_) => EntryKind::Toolchain,
            Arguments::Component(_) => EntryKind::Component,
            Arguments::Command(_) => EntryKind::Command,
        }
    }

    pub const fn name(&self) -> &Name {
        &self.name
    }

    pub const fn arguments(&self) -> &Arguments {
        &self.arguments
    }
}

// ------------------------------------------------------------------------------------------------
// Implementations ❱ EntryKind
// ------------------------------------------------------------------------------------------------

impl Display for EntryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for EntryKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "toolchain" => Ok(Self::Toolchain),
            "component" => Ok(Self::Component),
            "command" => Ok(Self::Command),
            _ => Err(Error::SourceParserError),
        }
    }
}

impl EntryKind {
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Toolchain => "toolchain",
            Self::Component => "component",
            Self::Command => "command",
        }
    }

    pub fn from_maybe_str(s: Option<&str>) -> Result<EntryKind, Error> {
        if let Some(maybe_kind) = s {
            EntryKind::from_str(maybe_kind)
        } else {
            Err(Error::SourceParserError)
        }
    }
}

// ------------------------------------------------------------------------------------------------
// Implementations ❱ Arguments
// ------------------------------------------------------------------------------------------------

impl Display for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Arguments::Toolchain(toolchain_arguments) => toolchain_arguments.to_string(),
                Arguments::Component(component_arguments) => component_arguments.to_string(),
                Arguments::Command(command_arguments) => command_arguments.to_string(),
            }
        )
    }
}

// ------------------------------------------------------------------------------------------------
// Implementations ❱ ToolchainArguments
// ------------------------------------------------------------------------------------------------

impl Display for ToolchainArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ", {}", self.as_string_vec().join(" "))
    }
}

impl From<ToolchainArguments> for Arguments {
    fn from(value: ToolchainArguments) -> Self {
        Arguments::Toolchain(value)
    }
}

impl EntryArguments for ToolchainArguments {
    fn has_any_arguments(&self) -> bool {
        self.profile.is_some() || !self.components.is_empty() || !self.targets.is_empty()
    }

    fn as_string_vec(&self) -> Vec<String> {
        let mut results = Vec::default();

        if let Some(profile) = &self.profile {
            results.push(format!("--profile"));
            results.push(profile.to_string());
        }
        if !self.components.is_empty() {
            results.push(format!("--component"));
            results.push(
                self.components
                    .iter()
                    .map(Name::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        if !self.targets.is_empty() {
            results.push(format!("--profile"));
            results.push(
                self.targets
                    .iter()
                    .map(Name::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }

        results
    }
}

impl ToolchainArguments {

    pub const fn profile(&self) -> Option<&Name> {
        self.profile.as_ref()
    }

    pub fn set_profile(&mut self, profile: Name) {
        self.profile = Some(profile);
    }

    pub fn unset_profile(&mut self) {
        self.profile = None;
    }

    pub const fn components(&self) -> &NameList {
        &self.components
    }

    pub fn set_components(&mut self, components: NameList) {
        self.components = components;
    }

    pub fn clear_components(&mut self) {
        self.components.clear();
    }

    pub const fn targets(&self) -> &NameList {
        &self.targets
    }

    pub fn set_targets(&mut self, targets: NameList) {
        self.targets = targets;
    }

    pub fn clear_targets(&mut self) {
        self.targets.clear();
    }
}

// ------------------------------------------------------------------------------------------------
// Implementations ❱ ComponentArguments
// ------------------------------------------------------------------------------------------------

impl Display for ComponentArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ", {}", self.as_string_vec().join(" "))
    }
}

impl From<ComponentArguments> for Arguments {
    fn from(value: ComponentArguments) -> Self {
        Arguments::Component(value)
    }
}

impl EntryArguments for ComponentArguments {
    fn has_any_arguments(&self) -> bool {
        self.target.is_some() || self.toolchain.is_some()
    }

    fn as_string_vec(&self) -> Vec<String> {
        let mut results = Vec::default();

        if let Some(target) = &self.target {
            results.push(format!("--target"));
            results.push(target.to_string());
        }
        if let Some(toolchain) = &self.toolchain {
            results.push(format!("--toolchain"));
            results.push(toolchain.to_string());
        }

        results
    }
}


impl ComponentArguments {

    pub const fn target(&self) -> Option<&Name> {
        self.target.as_ref()
    }

    pub fn set_target(&mut self, target: Name) {
        self.target = Some(target);
    }

    pub fn unset_target(&mut self) {
        self.target = None;
    }

    pub const fn toolchain(&self) -> Option<&Name> {
        self.toolchain.as_ref()
    }

    pub fn set_toolchain(&mut self, toolchain: Name) {
        self.toolchain = Some(toolchain);
    }

    pub fn unset_toolchain(&mut self) {
        self.toolchain = None;
    }
}

// ------------------------------------------------------------------------------------------------
// Implementations ❱ CommandArguments
// ------------------------------------------------------------------------------------------------

impl Display for CommandArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ", {}", self.as_string_vec().join(" "))
    }
}

impl From<CommandArguments> for Arguments {
    fn from(value: CommandArguments) -> Self {
        Arguments::Command(value)
    }
}

impl EntryArguments for CommandArguments {
    fn has_any_arguments(&self) -> bool {
        self.all_features.is_some()
            || self.branch.is_some()
            || self.debug.is_some()
            || !self.features.is_empty()
            || self.frozen.is_some()
            || self.git.is_some()
            || self.index.is_some()
            || self.locked.is_some()
            || self.no_default_features.is_some()
            || self.offline.is_some()
            || self.path.is_some()
            || self.registry.is_some()
            || self.rev.is_some()
            || self.tag.is_some()
            || self.target.is_some()
    }

    fn as_string_vec(&self) -> Vec<String> {
        let mut results = Vec::default();

        if let Some(all_features) = &self.all_features
            && *all_features
        {
            results.push(format!("--all-features"));
        }
        if let Some(branch) = &self.branch {
            results.push(format!("--branch"));
            results.push(branch.to_string());
        }
        if let Some(debug) = &self.debug
            && *debug
        {
            results.push(format!("--debug"));
        }
        if !self.features.is_empty() {
            results.push(format!("--features"));
            results.push(
                self.features
                    .iter()
                    .map(Name::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        if let Some(frozen) = &self.frozen
            && *frozen
        {
            results.push(format!("--frozen"));
        }
        if let Some(git) = &self.git {
            results.push(format!("--git"));
            results.push(git.to_string());
        }
        if let Some(index) = &self.index {
            results.push(format!("--index"));
            results.push(index.to_string());
        }
        if let Some(locked) = &self.locked
            && *locked
        {
            results.push(format!("--locked"));
        }
        if let Some(no_default_features) = &self.no_default_features
            && *no_default_features
        {
            results.push(format!("--no-default-features"));
        }
        if let Some(offline) = &self.offline
            && *offline
        {
            results.push(format!("--offline"));
        }
        if let Some(path) = &self.path {
            results.push(format!("--path"));
            results.push(path.to_string());
        }
        if let Some(registry) = &self.registry {
            results.push(format!("--registry"));
            results.push(registry.to_string());
        }
        if let Some(rev) = &self.path {
            results.push(format!("--rev"));
            results.push(rev.to_string());
        }
        if let Some(tag) = &self.tag {
            results.push(format!("--tag"));
            results.push(tag.to_string());
        }
        if let Some(target) = &self.target {
            results.push(format!("--target"));
            results.push(target.to_string());
        }

        results
    }
}

impl CommandArguments {
    pub const fn all_features(&self) -> Option<bool> {
        self.all_features
    }

    pub fn set_all_features(&mut self, all_features: bool) {
        self.all_features = Some(all_features);
    }

    pub fn unset_all_features(&mut self) {
        self.all_features = None;
    }

    pub const fn branch(&self) -> Option<&String> {
        self.branch.as_ref()
    }

    pub fn set_branch(&mut self, branch: String) {
        self.branch = Some(branch);
    }

    pub fn unset_branch(&mut self) {
        self.branch = None;
    }

    pub const fn debug(&self) -> Option<bool> {
        self.debug
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = Some(debug);
    }

    pub fn unset_debug(&mut self) {
        self.debug = None;
    }

    pub const fn features(&self) -> &NameList {
        &self.features
    }

    pub fn set_features(&mut self, features: NameList) {
        self.features = features;
    }

    pub fn clear_features(&mut self) {
        self.features.clear();
    }

    pub const fn frozen(&self) -> Option<bool> {
        self.frozen
    }

    pub fn set_frozen(&mut self, frozen: bool) {
        self.frozen = Some(frozen);
    }

    pub fn unset_frozen(&mut self) {
        self.frozen = None;
    }

    pub const fn git(&self) -> Option<&Url> {
        self.git.as_ref()
    }

    pub fn set_git(&mut self, git: Url) {
        self.git = Some(git);
    }

    pub fn unset_git(&mut self) {
        self.git = None;
    }

    pub const fn index(&self) -> Option<&String> {
        self.index.as_ref()
    }

    pub fn set_index(&mut self, index: String) {
        self.index = Some(index);
    }

    pub fn unset_bindex(&mut self) {
        self.index = None;
    }

    pub const fn locked(&self) -> Option<bool> {
        self.locked
    }

    pub fn set_locked(&mut self, locked: bool) {
        self.locked = Some(locked);
    }

    pub fn unset_locked(&mut self) {
        self.locked = None;
    }

    pub const fn no_default_features(&self) -> Option<bool> {
        self.no_default_features
    }

    pub fn set_no_default_features(&mut self, no_default_features: bool) {
        self.no_default_features = Some(no_default_features);
    }

    pub fn unset_no_default_features(&mut self) {
        self.no_default_features = None;
    }

    pub const fn offline(&self) -> Option<bool> {
        self.offline
    }

    pub fn set_offline(&mut self, offline: bool) {
        self.offline = Some(offline);
    }

    pub fn unset_offline(&mut self) {
        self.offline = None;
    }

    pub const fn path(&self) -> Option<&String> {
        self.path.as_ref()
    }

    pub fn set_path(&mut self, path: String) {
        self.path = Some(path);
    }

    pub fn unset_path(&mut self) {
        self.path = None;
    }

    pub const fn registry(&self) -> Option<&String> {
        self.registry.as_ref()
    }

    pub fn set_registry(&mut self, registry: String) {
        self.registry = Some(registry);
    }

    pub fn unset_registry(&mut self) {
        self.registry = None;
    }

    pub const fn rev(&self) -> Option<&String> {
        self.rev.as_ref()
    }

    pub fn set_rev(&mut self, rev: String) {
        self.rev = Some(rev);
    }

    pub fn unset_rev(&mut self) {
        self.rev = None;
    }

    pub const fn tag(&self) -> Option<&String> {
        self.tag.as_ref()
    }

    pub fn set_tag(&mut self, tag: String) {
        self.tag = Some(tag);
    }

    pub fn unset_tag(&mut self) {
        self.tag = None;
    }

    pub const fn target(&self) -> Option<&Name> {
        self.target.as_ref()
    }

    pub fn set_target(&mut self, target: Name) {
        self.target = Some(target);
    }

    pub fn unset_target(&mut self) {
        self.target = None;
    }
}

// ------------------------------------------------------------------------------------------------
// Implementations ❱ Name
// ------------------------------------------------------------------------------------------------

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "'{}'", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl FromStr for Identifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            error!("Identifier::from_str; value is empty");
            Err(Error::Unknown {
                message: "Identifier::from_str; value is empty".to_string(),
            })
        } else if !s.chars().next().unwrap().is_ascii_alphabetic() {
            error!("Identifier::from_str; first character must be alphabetic");
            Err(Error::Unknown {
                message: "Identifier::from_str; first character must be alphabetic".to_string(),
            })
        } else if !(&s[1..])
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '+' || c == '@')
        {
            error!("Identifier::from_str; value is empty");
            Err(Error::Unknown {
                message: "Identifier::from_str; value is empty".to_string(),
            })
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl Identifier {
    pub fn new_unchecked(s: &str) -> Self {
        Self(s.to_string())
    }
}

// ------------------------------------------------------------------------------------------------
// Implementations ❱ Name
// ------------------------------------------------------------------------------------------------

impl AsRef<Identifier> for Name {
    fn as_ref(&self) -> &Identifier {
        &self.0
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.0)
    }
}

impl FromStr for Name {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Identifier::from_str(s)?))
    }
}

impl From<Identifier> for Name {
    fn from(id: Identifier) -> Self {
        Self(id)
    }
}

impl Name {
    pub fn new_unchecked(s: &str) -> Self {
        Self(Identifier::new_unchecked(s))
    }

    pub fn as_str(&self) -> &str {
        self.as_ref().as_ref()
    }
}

// ------------------------------------------------------------------------------------------------
// Implementations ❱ Value
// ------------------------------------------------------------------------------------------------

impl Value {
    /// Boolean(bool),
    pub const fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }
    pub const fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(v) => Some(*v),
            _ => None,
        }
    }
    pub  fn into_boolean(self) -> Option<bool> {
        match self {
            Self::Boolean(v) => Some(v),
            _ => None,
        }
    }
    /// Integer(i64),
    pub const fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }
    pub const fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(v) => Some(*v),
            _ => None,
        }
    }
    pub  fn into_integer(self) -> Option<i64> {
        match self {
            Self::Integer(v) => Some(v),
            _ => None,
        }
    }
    /// Float(f64),
    pub const fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }
    pub const fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }
    pub  fn into_float(self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(v),
            _ => None,
        }
    }
    /// String(String),
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }
    pub const fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }
    pub  fn into_string(self) -> Option<String> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }
    /// Url(Url),
    pub const fn is_url(&self) -> bool {
        matches!(self, Self::Url(_))
    }
    pub const fn as_url(&self) -> Option<&Url> {
        match self {
            Self::Url(v) => Some(v),
            _ => None,
        }
    }
    pub  fn into_url(self) -> Option<Url> {
        match self {
            Self::Url(v) => Some(v),
            _ => None,
        }
    }
    /// Name(Name),
    pub const fn is_name(&self) -> bool {
        matches!(self, Self::Name(_))
    }
    pub const fn as_name(&self) -> Option<&Name> {
        match self {
            Self::Name(v) => Some(v),
            _ => None,
        }
    }
    pub  fn into_name(self) -> Option<Name> {
        match self {
            Self::Name(v) => Some(v),
            _ => None,
        }
    }
    /// NameList(NameList),
    pub const fn is_name_list(&self) -> bool {
        matches!(self, Self::NameList(_))
    }
    pub const fn as_name_list(&self) -> Option<&NameList> {
        match self {
            Self::NameList(v) => Some(v),
            _ => None,
        }
    }
    pub  fn into_name_list(self) -> Option<NameList> {
        match self {
            Self::NameList(v) => Some(v),
            _ => None,
        }
    }
}
