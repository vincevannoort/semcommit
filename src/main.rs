use console::style;
use dialoguer::{console::Term, theme::ColorfulTheme, FuzzySelect, Input};
use std::process::Command;

fn main() {
    let theme = ColorfulTheme::default();
    println!("{}", style("Format your commit message:").bold());
    let _type = format!("{}", style("type").magenta().bold());
    let _project = format!("{}", style("project").green().bold());
    let _message = format!("{}", style("message").blue().bold());
    println!("{}({}): {}", _type, _project, _message);

    println!();

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
        .default("agencies".into())
        .interact_text()
        .unwrap();

    let commit_message: String = Input::with_theme(&theme)
        .with_prompt(_message)
        .default("some message".into())
        .interact_text()
        .unwrap();

    let complete_commit_message = format!("{commit_type}({commit_project}): {commit_message}");

    let output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&complete_commit_message)
        .output()
        .expect("failed to execute process");

    println!();

    println!("git commit -m \"{complete_commit_message}\"");

    println!();
    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
}
