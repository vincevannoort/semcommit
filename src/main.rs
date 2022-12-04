use console::style;
use dialoguer::{console::Term, theme::ColorfulTheme, FuzzySelect, Input};
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize, Serialize)]
struct Defaults {
    commit_type: String,
    commit_project: String,
}

impl Defaults {
    fn read_from_file() -> Self {
        let binding = env::temp_dir();
        let dir = binding.to_str().unwrap();
        let location = format!("{dir}/.semcommit-defaults");
        let mut defaults: Defaults = Defaults {
            commit_type: "feat".to_string(),
            commit_project: "none".to_string(),
        };

        if Path::new(&location).exists() {
            let mut result: String = String::new();
            let mut file = OpenOptions::new().read(true).open(&location).unwrap();
            file.read_to_string(&mut result).unwrap();
            defaults = toml::from_str(&result).unwrap();
            println!("{:?}", defaults);
        }
        return defaults;
    }
    fn store_in_file(&self) {
        let binding = env::temp_dir();
        let dir = binding.to_str().unwrap();
        let location = format!("{dir}/.semcommit-defaults");
        let string_write = toml::to_string(&self).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&location)
            .unwrap();
        file.write_all(string_write.as_bytes()).unwrap();
    }
}

fn main() {
    let defaults = Defaults::read_from_file();
    let theme = ColorfulTheme::default();

    println!("{}", style("Format your commit message:").bold());
    let _type = format!("{}", style("type").magenta().bold());
    let _project = format!("{}", style("project").green().bold());
    let _message = format!("{}", style("message").blue().bold());
    println!("{}({}): {}\n", _type, _project, _message);

    let items = vec!["feat", "fix", "docs", "style", "refactor", "test", "chore"];
    let commit_type_index = FuzzySelect::with_theme(&theme)
        .with_prompt(_type)
        .items(&items)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap()
        .unwrap();
    let commit_type = items[commit_type_index].to_string();

    let commit_project: String = Input::with_theme(&theme)
        .with_prompt(_project)
        .default(defaults.commit_project)
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

    // always store defaults, such that the user has less work in the future
    Defaults {
        commit_type: commit_type,
        commit_project: commit_project,
    }
    .store_in_file();
}
