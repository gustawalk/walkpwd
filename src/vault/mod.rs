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

pub fn get_home_dir() -> Result<String, Box<dyn Error>>{
    let home_dir = match env::home_dir(){
        Some(path) => path.to_string_lossy().to_string(),
        None => Err("Could not determine your home directory").unwrap(),
    };
    Ok(home_dir)
}

pub fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn Error>> {
    let wayland = env::var("WAYLAND_DISPLAY").is_ok();
    let x11 = env::var("DISPLAY").is_ok();
    let os = env::consts::OS;

    if wayland {
        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(text.as_bytes())?;
                drop(stdin);
            }
            return Ok(());
        }
    } else if x11 {
        if let Ok(mut child) = Command::new("xclip")
            .args(&["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(text.as_bytes())?;
                drop(stdin);
            }
            return Ok(());
        }

        if let Ok(mut child) = Command::new("xsel")
            .args(&["--clipboard", "--input"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(text.as_bytes())?;
                drop(stdin);
            }
            return Ok(());
        }
    } else if os == "macos" {
        if let Ok(mut child) = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(text.as_bytes())?;
                drop(stdin);
            }
            return Ok(());
        }
    }

    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        clipboard.set_text(text.to_string())?;
        return Ok(());
    }

    Err("No method available.".into())
}

pub fn init_vault() -> Result<(), Box<dyn Error>> {

    let vault_dir = get_vault_dir()?;
    fs::create_dir_all(&vault_dir)?;

    if !is_vault_initialized()? {
        println!("Flag not detected, initializing...");
        let init_file = format!("{}/vault_initialized.flag", vault_dir);
        File::create(&init_file)?.write_all(b"initialized")?;
    }

    if vault_file_exists()? {
        println!("Here i will ask if the user want to overwrite to create a new one");
        return Ok(());
    }else {
        println!("Vault not detected, creating a new one...");
        let vault_file = format!("{}/vault.json", vault_dir);
        File::create(&vault_file)?.write_all(b"[]")?;
    }

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

pub fn vault_file_exists() -> Result<bool, Box<dyn Error>> {
    let vault_dir = get_vault_dir()?;
    let vault_file = format!("{}/vault.json", vault_dir);
    Ok(Path::new(&vault_file).exists())
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

// WALKPWD ADD

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
        println!("Password entry for {} already exists.", name);
        return Err("Password entry already exists".into());
    }

    entries.push(PasswordEntry { name: name.clone(), password: password.clone() });
    let content = serde_json::to_string(&entries)?;
    File::create(&vault_file)?.write_all(content.as_bytes())?;
    let _ = copy_to_clipboard(&password);
    println!("Password added for {} + copied to clipboard", name);
    Ok(password)
}

// WALKPWD GET

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
            println!("The password is: {}", entry.password);
        }
        copy_to_clipboard(&entry.password)?;
        println!("Password copied to clipboard.");
    }else{
        println!("No password found for: {}", name);
    }
    Ok(())
}

// WALKPWD LIST

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

// WALKPWD DELETE

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
        println!("Password entry for {} deleted.", name);
    } else {
        println!("No password found for {}", name);
    }
    Ok(())
}
