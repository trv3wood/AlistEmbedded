// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
use std::process::{exit, Child};
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let child = init().unwrap_or_else(|e| {
                eprintln!("{:?}", e);
                exit(-1);
            });
            app.manage(Mutex::new(child));
            Ok(())
        })
        .on_window_event(|app, event| match event {
            tauri::WindowEvent::Destroyed => {
                let child = app.state::<Mutex<Child>>();
                let mut child = child.try_lock().unwrap_or_else(|e| {
                    eprintln!("{:?}", e);
                    exit(-1);
                });
                child.kill().expect("Failed to kill child process");
            }
            _ => {}
        })
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
use std::{
    env,
    io::{BufRead, BufReader},
    os::windows::process::CommandExt,
    process::{Command, Stdio},
    sync::Mutex,
    thread::spawn,
};

fn find_alist_path<'a>(path_env: &'a str) -> &'a str {
    path_env
        .split(';')
        .find(|item| item.contains("alist"))
        .expect("Alist not found in PATH")
}

#[derive(Debug)]
struct AppConfig<'a> {
    pub path: &'a str,
    pub storage_count: usize,
}

impl<'a> AppConfig<'a> {
    pub fn from_env(env_path: &'a str) -> std::io::Result<Self> {
        let alist_path = find_alist_path(env_path);
        let mut child = Command::new("alist")
            .current_dir(alist_path)
            .args(["storage", "list"])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?;

        let log_stream = child.stderr.take(); // 打印日志用了错误流
        let reader = BufReader::new(log_stream.unwrap());
        let found_storage = reader
            .lines()
            .find(|line| {
                if let Ok(line) = line {
                    return line.contains("Found");
                } else {
                    false
                }
            })
            .expect("failed to find storage list")?;
        let storage_count = found_storage
            .split_whitespace()
            .rfind(|ch| ch.parse::<usize>().is_ok())
            .unwrap()
            .parse::<usize>()
            .unwrap();
        child.kill()?;
        let config = Self {
            path: alist_path,
            storage_count,
        };
        return Ok(config);
    }
}

fn init() -> std::io::Result<Child> {
    let env_path = env::var("PATH").unwrap();
    let config = AppConfig::from_env(&env_path)?;
    let mut alist_service = Command::new("alist")
        .creation_flags(0x08000000)
        .current_dir(config.path)
        .arg("server")
        .stderr(Stdio::piped())
        .spawn()?;
    println!("Waiting for Alist to start...");
    let reader = BufReader::new(alist_service.stderr.take().unwrap());
    let mut found_count = 0;
    let listening_child = spawn(move || {
        for line in reader.lines() {
            if let Ok(line) = line {
                dbg!(&line);
                if line.contains("load storage") {
                    found_count += 1;
                }
            }
            if found_count == config.storage_count {
                break;
            }
        }
    });
    listening_child.join().unwrap();
    println!("Alist server is running!");
    Ok(alist_service)
}

#[cfg(test)]
mod tests {
    use std::{env::var, process, thread::spawn};

    use super::*;
    #[test]
    fn test_listening() -> std::io::Result<()> {
        let env_path = env::var("PATH").unwrap();
        let config = AppConfig::from_env(&env_path)?;
        dbg!(&config);
        let mut alist_servive = process::Command::new("alist")
            .current_dir(config.path)
            .arg("server")
            .stderr(Stdio::piped())
            .spawn()?;
        println!("Waiting for Alist to start...");
        let reader = BufReader::new(alist_servive.stderr.take().unwrap());
        let mut found_count = 0;
        let listening_child = spawn(move || {
            for line in reader.lines() {
                if let Ok(line) = line {
                    dbg!(&line);
                    if line.contains("load storage") {
                        found_count += 1;
                    }
                }
                if found_count == config.storage_count {
                    break;
                }
            }
        });
        listening_child.join().unwrap();
        println!("Alist server is running!");
        alist_servive.kill()?;
        Ok(())
    }
    #[test]
    fn list_storage() -> std::io::Result<()> {
        let path = var("PATH").expect("PATH not found");
        let alist_path = path
            .split(';')
            .find(|item| item.contains("alist"))
            .expect("Alist not found in PATH");
        let mut alist_service = Command::new("alist")
            .current_dir(alist_path)
            .args(["storage", "list"])
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .expect("Failed to execute command");
        let reader = BufReader::new(alist_service.stderr.take().unwrap());
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("{}", line);
            } else {
                break;
            }
        }
        alist_service.kill()?;
        Ok(())
    }
    #[test]
    fn get_storage() -> std::io::Result<()> {
        let env_path = var("PATH").expect("PATH not found");
        let alist_path = find_alist_path(&env_path);
        let mut child = Command::new("alist")
            .current_dir(alist_path)
            .args(["storage", "list"])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to execute alist command");
        let stdout = child.stderr.take().expect("Failed to capture stdout");
        let reader = BufReader::new(stdout);
        let found_storage = reader
            .lines()
            .find(|line| {
                if let Ok(line) = line {
                    return line.contains("Found");
                } else {
                    false
                }
            })
            .expect("failed to find storage list")
            .expect("stdout missing");
        child.kill()?;
        let storage = found_storage
            .split_whitespace()
            .rfind(|ch| ch.parse::<usize>().is_ok())
            .unwrap()
            .parse::<usize>()
            .unwrap();
        println!("storage {}", storage);
        Ok(())
    }
}
