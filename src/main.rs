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

#[derive(Debug, Deserialize, Serialize, Default)]
struct Config {
    commit_type: String,
    commit_project: String,
}

#[derive(Display, EnumIter)]
enum CommitType {
    feat,
    fix,
    docs,
    style,
    refactor,
    test,
    chore,
}

impl Config {
    fn read_from_file() -> Option<Self> {
        let binding = env::temp_dir();
        let dir = binding.to_str().unwrap();
        let location = format!("{dir}/.semcommit-defaults");
        let mut defaults: Config = Config {
            commit_type: "feat".to_string(),
            commit_project: "none".to_string(),
        };

        if Path::new(&location).exists() {
            let mut result: String = String::new();
            let mut file = OpenOptions::new().read(true).open(&location).unwrap();
            file.read_to_string(&mut result).unwrap();
            defaults = toml::from_str(&result).unwrap_or_default();
        }
        return Some(defaults);
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
            .open(&location)
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
            .output()
            .expect("Could not add files");
    }
}

fn main() {
    let config_from_temporary_file = Config::read_from_file();
    let theme = ColorfulTheme::default();

    let config: Config = match config_from_temporary_file {
        Some(default) => default,
        None => Config {
            commit_type: CommitType::feat.to_string(),
            commit_project: "example".to_string(),
        },
    };

    check_for_unstaged_changes();

    println!("{}", style("Format your commit message:").bold());
    let _type = format!("{}", style("type").magenta().bold());
    let _project = format!("{}", style("project").green().bold());
    let _message = format!("{}", style("message").blue().bold());
    println!("{}({}): {}\n", _type, _project, _message);

    let items: Vec<String> = CommitType::iter().map(|ct| ct.to_string()).collect();
    let default_item_pos = items.iter().position(|i| i == &config.commit_type);
    let commit_type_index = FuzzySelect::with_theme(&theme)
        .with_prompt(_type)
        .items(&items)
        .default(default_item_pos.unwrap_or(0))
        .interact_on_opt(&Term::stderr())
        .unwrap()
        .unwrap();
    let commit_type = items[commit_type_index].to_string();

    let commit_project: String = Input::with_theme(&theme)
        .with_prompt(_project)
        .default(config.commit_project)
        .interact_text()
        .unwrap();

    let commit_message: String = Input::with_theme(&theme)
        .with_prompt(_message)
        .default("some message".into())
        .interact_text()
        .unwrap();

    let complete_commit_message = format!("{commit_type}({commit_project}): {commit_message}");

    println!("\ngit commit -m \"{complete_commit_message}\"");
    let output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&complete_commit_message)
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    let defaults = Config {
        commit_type: commit_type,
        commit_project: commit_project,
    };

    // always store defaults, such that the user has less work in the future
    defaults.store_in_file();
}
