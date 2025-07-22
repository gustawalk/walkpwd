pub mod crypto;
use std::fs::{self, File};
use std::{error::Error};
use std::path::Path;
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use passwords::PasswordGenerator;
use arboard::Clipboard;
use std::process::{Command, Stdio};
use std::{env, io::Write, thread, time::Duration};

#[derive(Serialize, Deserialize)]
struct PasswordEntry {
    name: String,
    password: String,
}

pub fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn Error>> {
    let wayland = env::var("WAYLAND_DISPLAY").is_ok();
    let x11 = env::var("DISPLAY").is_ok();
    let os = env::consts::OS;

    if wayland {
        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
        {
            child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
            thread::sleep(Duration::from_millis(100));
            return Ok(());
        }
    } else if x11 {
        if let Ok(mut child) = Command::new("xclip")
            .args(&["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()
        {
            child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
            return Ok(());
        }

        if let Ok(mut child) = Command::new("xsel")
            .args(&["--clipboard", "--input"])
            .stdin(Stdio::piped())
            .spawn()
        {
            child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
            return Ok(());
        }
    } else if os == "macos" {
        if let Ok(mut child) = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
        {
            child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
            return Ok(());
        }
    }

    if let Ok(mut clipboard) = Clipboard::new() {
        clipboard.set_text(text.to_string())?;
        return Ok(());
    }

    Err("No method avaliable.".into())
}

pub fn init_vault() -> Result<(), Box<dyn Error>> {
    let vault_dir = get_vault_dir()?;
    fs::create_dir_all(&vault_dir)?;
    let init_file = format!("{}/vault_initialized.flag", vault_dir);
    File::create(&init_file)?.write_all(b"initialized")?;
    let vault_file = format!("{}/vault.json", vault_dir);
    File::create(&vault_file)?.write_all(b"[]")?;
    println!("Vault initialized successfully!");
    Ok(())
}

pub fn get_vault_dir() -> Result<String, Box<dyn Error>> {
    let proj_dirs = ProjectDirs::from("", "", "walkpwd")
        .ok_or("Could not determine project directory")?;
    let vault_dir = proj_dirs.data_dir().to_string_lossy().to_string();
    Ok(vault_dir)
}

pub fn is_vault_initialized() -> Result<bool, Box<dyn Error>> {
    let vault_dir = get_vault_dir()?;
    let init_file = format!("{}/vault_initialized.flag", vault_dir);
    Ok(Path::new(&init_file).exists())
}

pub fn generate_random_password(length: Option<usize>, use_symbols: bool) -> Result<String, Box<dyn Error>> {
    let length = length.unwrap_or(12);
    let generator = PasswordGenerator{
        length,
        numbers: true,
        uppercase_letters: true,
        lowercase_letters: true,
        symbols: use_symbols,
        exclude_similar_characters: true,
        spaces: false,
        strict: false,
    };
    let password = generator.generate_one();
    Ok(password?)
}

pub fn add_password(name: String, password: String) -> Result<String, Box<dyn Error>> {
    let vault_dir = get_vault_dir()?;
    let vault_file = format!("{}/vault.json", vault_dir);
    let mut entries: Vec<PasswordEntry> = if Path::new(&vault_file).exists() {
        let content = fs::read_to_string(&vault_file)?;
        serde_json::from_str(&content)?
    } else {
        Vec::new()
    };

    if entries.iter().any(|entry| entry.name == name){
        println!("Password entry for '{}' already exists.", name);
        return Err("Password entry already exists".into());
    }

    entries.push(PasswordEntry { name: name.clone(), password: password.clone() });
    let content = serde_json::to_string(&entries)?;
    File::create(&vault_file)?.write_all(content.as_bytes())?;
    println!("Password added for {} + copied to clipboard", name);
    Ok(password)
}

pub fn get_password(name: String, reveal: bool) -> Result<(), Box<dyn Error>> {
    let vault_dir = get_vault_dir()?;
    let vault_file = format!("{}/vault.json", vault_dir);
    let entries: Vec<PasswordEntry> = if Path::new(&vault_file).exists() {
        let content = fs::read_to_string(&vault_file)?;
        serde_json::from_str(&content)?
    }else{
        Vec::new()
    };

    if let Some(entry) = entries.iter().find(|entry| entry.name == name){
        if reveal {
            println!("The password is '{}'", entry.password);
        }
        copy_to_clipboard(&entry.password)?;
        println!("Password copied to clipboard.");
    }else{
        println!("No password found for '{}'", name);
    }
    Ok(())
}

pub fn list_passwords() -> Result<(), Box<dyn Error>> {
    let vault_dir = get_vault_dir()?;
    let vault_file = format!("{}/vault.json", vault_dir);
    let entries: Vec<PasswordEntry> = if Path::new(&vault_file).exists() {
        let content = fs::read_to_string(&vault_file)?;
        serde_json::from_str(&content)?
    } else {
        Vec::new()
    };
    if entries.is_empty() {
        println!("No passwords stored.");
    } else {
        println!("Stored passwords:");
        for entry in entries {
            println!("- {}", entry.name);
        }
    }
    Ok(())
}

pub fn delete_password(name: String) -> Result<(), Box<dyn Error>> {
    let vault_dir = get_vault_dir()?;
    let vault_file = format!("{}/vault.json", vault_dir);
    let mut entries: Vec<PasswordEntry> = if Path::new(&vault_file).exists() {
        let content = fs::read_to_string(&vault_file)?;
        serde_json::from_str(&content)?
    } else {
        Vec::new()
    };

    if let Some(pos) = entries.iter().position(|entry| entry.name == name) {
        entries.remove(pos);
        let content = serde_json::to_string(&entries)?;
        File::create(&vault_file)?.write_all(content.as_bytes())?;
        println!("Password entry for '{}' deleted.", name);
    } else {
        println!("No password found for '{}'", name);
    }
    Ok(())
}
