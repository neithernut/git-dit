use std::process::Command;

fn main() {
    if cfg!(feature = "manpage") {
        assert!(Command::new("pandoc")
            .arg("-s")
            .arg("-f").arg("markdown")
            .arg("-t").arg("man")
            .arg("git-dit.1.md")
            .arg("-o").arg("git-dit.1")
            .status()
            .unwrap()
            .success())
    }
}

