use std::process::Command;

fn main() {
    assert!(Command::new("pandoc")
        .arg("-s")
        .arg("-t").arg("man")
        .arg("git-dit.1.md")
        .arg("-o").arg("git-dit.1")
        .status()
        .unwrap()
        .success())
}

