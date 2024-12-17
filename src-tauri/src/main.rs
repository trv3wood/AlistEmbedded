// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::process::exit;

fn main() {
    if let Err(e) = alistembedded_lib::init() {
        eprintln!("{:?}", e);
        exit(-1);
    }
    alistembedded_lib::run()
}

#[cfg(test)]
mod tests {
    use std::{env, process};

    #[test]
    fn test_alist_path() {
        let path = env::var("PATH").expect("PATH not found");
        println!("{}", path);
        let alist_path = path
            .split(';')
            .find(|item| item.contains("alist"))
            .expect("Alist not found in PATH");
        dbg!(alist_path);
    }
    #[test]
    fn test_launch_alist() -> std::io::Result<()> {
        let path = env::var("PATH").expect("PATH not found");
        let alist_path = path
            .split(';')
            .find(|item| item.contains("alist"))
            .expect("Alist not found in PATH");
        let mut child = process::Command::new("alist")
            .current_dir(alist_path)
            .arg("server")
            .spawn()
            .expect("Failed to execute command");
        dbg!(&child);
        child.kill()?;
        Ok(())
    }
}
