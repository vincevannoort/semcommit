use clap::Parser;
use console::style;
use dialoguer::{console::Term, theme::ColorfulTheme, Confirm, FuzzySelect, Input};
use regex::Regex;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    commit_type: String,
    commit_project: String,
    commit_message: String,
}

#[derive(Debug, Clone, Display, clap::ValueEnum)]
enum Mode {
    Normal,
    Emoji,
}

#[derive(Display, EnumIter)]
enum CommitType {
    Feat,
    Fix,
    Docs,
    Style,
    Refactor,
    Test,
    Chore,
}

impl CommitType {
    fn get_emoji_by_type(self) -> String {
        let emoji = match self {
            CommitType::Feat => "🚀",
            CommitType::Fix => "🔨",
            CommitType::Docs => "📄",
            CommitType::Style => "🎨",
            CommitType::Refactor => "🧰",
            CommitType::Test => "🧪",
            CommitType::Chore => "🧹",
        }
        .to_string();
        format!("{} {}", emoji, self.to_string().to_lowercase())
    }
    fn get_option_list_by_mode(mode: Mode) -> Vec<String> {
        return match mode {
            Mode::Normal => CommitType::iter()
                .map(|ct| ct.to_string().to_lowercase())
                .collect(),
            Mode::Emoji => CommitType::iter()
                .map(|ct| ct.get_emoji_by_type())
                .collect(),
        };
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            commit_type: "feat".to_string(),
            commit_project: "none".to_string(),
            commit_message: "something".to_string(),
        }
    }
}

impl Config {
    fn read_from_file() -> Option<Self> {
        let binding = env::temp_dir();
        let dir = binding.to_str().unwrap();
        let location = format!("{dir}/.semcommit-defaults");
        let mut defaults: Config = Config::default();

        if Path::new(&location).exists() {
            let mut result: String = String::new();
            let mut file = OpenOptions::new().read(true).open(&location).unwrap();
            file.read_to_string(&mut result).unwrap();
            defaults = toml::from_str(&result).unwrap_or_default();
        }
        Some(defaults)
    }
    fn store_in_file(&self) {
        let binding = env::temp_dir();
        let dir = binding.to_str().unwrap();
        let location = format!("{dir}/.semcommit-defaults");
        let string_write = toml::to_string(&self).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(location)
            .unwrap();
        file.write_all(string_write.as_bytes()).unwrap();
    }
}

fn check_for_unstaged_changes() {
    let binding = Command::new("git")
        .arg("status")
        .output()
        .expect("Could not retrieve git status");
    let git_status_output = String::from_utf8_lossy(&binding.stdout);
    let no_changes_committed =
        Regex::new(r"(no changes added to commit|nothing to commit, working tree clean|nothing added to commit but untracked files present)").unwrap();

    // early exit if user has staged files
    if !no_changes_committed.is_match(&git_status_output) {
        return;
    }

    let user_wants_to_commit = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("No changes committed, want to commit them?")
        .interact()
        .unwrap();

    if user_wants_to_commit {
        Command::new("git")
            .arg("add")
            .arg(".")
            .spawn()
            .expect("Could not add files")
            .wait()
            .expect("Could not add files");
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(value_enum, short, long, default_value_t=Mode::Normal)]
    mode: Mode,
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    let config_from_temporary_file = Config::read_from_file();
    let theme = ColorfulTheme::default();

    let config: Config = match config_from_temporary_file {
        Some(default) => default,
        None => Config::default(),
    };

    check_for_unstaged_changes();

    println!("{}", style("Format your commit message:").bold());
    let _type = format!("{}", style("type").magenta().bold());
    let _project = format!("{}", style("project").green().bold());
    let _message = format!("{}", style("message").blue().bold());
    println!("{}({}): {}\n", _type, _project, _message);

    let commit_types: Vec<String> = CommitType::get_option_list_by_mode(args.mode);
    let default_item_pos = commit_types.iter().position(|i| i == &config.commit_type);
    let commit_type_index = FuzzySelect::with_theme(&theme)
        .with_prompt(_type)
        .items(&commit_types)
        .default(default_item_pos.unwrap_or(0))
        .interact_on_opt(&Term::stderr())
        .unwrap()
        .unwrap();
    let commit_type = commit_types[commit_type_index].to_string();

    let commit_project: String = Input::with_theme(&theme)
        .with_prompt(_project)
        .default(config.commit_project)
        .interact_text()
        .unwrap();

    let commit_message: String = Input::with_theme(&theme)
        .with_prompt(_message)
        .default(config.commit_message)
        .interact_text()
        .unwrap();

    let complete_commit_message = format!("{commit_type}({commit_project}): {commit_message}");

    // commit files
    Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&complete_commit_message)
        .spawn()
        .expect("failed to commit files")
        .wait()
        .expect("failed to commit files");

    let defaults = Config {
        commit_type,
        commit_project,
        commit_message,
    };

    // always store defaults, such that the user has less work in the future
    defaults.store_in_file();
}
