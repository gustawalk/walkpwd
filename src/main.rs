mod vault;
use clap::{Parser, Subcommand};
use std::error::Error;

#[derive(Parser)]
#[clap(name = "walkpwd", version = "0.1.0", author = "Gustavo Walk", about = "A secure CLI password manager")]
struct WalkPwd {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(name = "init", about = "Initialize the password vault")]
    Init,
    #[clap(name = "add", about = "Add a new password to the vault")]
    Add {
        #[clap(short='n', long, help = "Name of the password entry")]
        name: String,
        #[clap(short='p', long, help = "Password to be stored (if not provided, a random 12 characters password will be generated)")]
        password: Option<String>,
        #[clap(short='l', long, help = "Set the length of the generated password (default is 12 characters)")]
        length: Option<usize>,
        #[clap(short, long, help = "Use symbols in the generated password (default is false)")]
        use_symbols: bool,
    },
    #[clap(name = "get", about = "Retrieve a password from the vault")]
    Get {
        #[clap(short='n', long, help = "Name of the password entry to retrieve")]
        name: String,
        #[clap(short='r', long, help = "Reveal the password in plaintext")]
        reveal: bool,
    },
    #[clap(name = "list", about = "List all passwords in the vault")]
    List,
    #[clap(name = "delete", about = "Delete a password from the vault")]
    Delete {
        #[clap(short='n', long, help = "Name of the password entry to delete")]
        name: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = WalkPwd::parse();

    match &cli.command {
        Commands::Add { .. } | Commands::Get { .. } | Commands::List | Commands::Delete {..} => {
            if !vault::is_vault_initialized()? {
                return Err("Vault is not initialized. Please run 'walkpwd init' first.".into());
            }
        }
        Commands::Init => {}
    }

    match cli.command {
        Commands::Init => {
            let _ = vault::init_vault();
        }
        Commands::Add { name, password, length, use_symbols } => {
            let final_password = match password {
                Some(p) => {
                    match length{
                        Some(_) => {
                            return Err("Cannot specify both password and length".into());
                        }
                        None => p.clone(),
                    };
                    match use_symbols {
                        true => {
                            return Err("Cannot specify both password and use_symbols".into());
                        }
                        false => p.clone(),
                    }
                }
                None => {
                    vault::generate_random_password(length, use_symbols)?
                },
            };
            let _ = vault::add_password(name, final_password);
        }
        Commands::Get { name, reveal } => {
            let _ = vault::get_password(name, reveal);
        }
        Commands::List => {
            let _ = vault::list_passwords();
        }
        Commands::Delete { name } => {
            let _ = vault::delete_password(name);
        }
    }

    Ok(())
}
