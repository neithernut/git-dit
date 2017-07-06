use std::process::Command;

fn main() {
    if std::env::var("BUILD_GIT_DIT_MAN").is_ok() {
        assert!(Command::new("pandoc")
            .arg("-s")
            .arg("-S")
            .arg("-f").arg("markdown")
            .arg("-t").arg("man")
            .arg("git-dit.1.md")
            .arg("-o").arg("git-dit.1")
            .status()
            .unwrap()
            .success())
    }
}

