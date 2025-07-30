// Helps to setup the account for the user. With this user can either create new account or import private key of an existing account.

use bs58;
use clap::Parser;
use colored::*;
use solana_sdk::signature::{Keypair, Signer, write_keypair_file};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct SetupKeypairArgs {
    /// Generate a new keypair
    #[arg(long, conflicts_with = "import")]
    pub generate: bool,

    /// Import a base58-encoded keypair string
    #[arg(long, value_name = "BASE58", conflicts_with = "generate")]
    pub import: Option<String>,

    /// Overwrite existing keypair without prompt
    #[arg(long)]
    pub force: bool,
}

pub fn handle_setup_keypair(args: &SetupKeypairArgs) {
    let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
    let config_dir = format!("{}/.config/mor-supply", home_dir);
    let keypair_path = format!("{}/id.json", config_dir);
    let keypair_path_buf = PathBuf::from(&keypair_path);

    // Ensure config directory exists
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    // If keypair exists, handle force flag
    if keypair_path_buf.exists() {
        if args.force {
            fs::remove_file(&keypair_path_buf).expect("Failed to delete existing keypair");
        } else {
            println!(
                "{} {}.",
                "A keypair already exists at".yellow(),
                keypair_path.cyan()
            );
            println!("{}", "Use --force to overwrite it.".yellow());
            return;
        }
    }

    let keypair = if args.generate {
        let kp = Keypair::new();
        println!(
            "{} {}",
            "Generated new keypair. Public key:".green(),
            kp.pubkey().to_string().cyan()
        );
        kp
    } else if let Some(b58) = &args.import {
        let bytes = match bs58::decode(b58.trim()).into_vec() {
            Ok(b) => b,
            Err(_) => {
                println!("{}", "Invalid base58 string. Exiting.".red());
                return;
            }
        };
        match Keypair::from_bytes(&bytes) {
            Ok(kp) => {
                println!(
                    "{} {}",
                    "Imported keypair. Public key:".green(),
                    kp.pubkey().to_string().cyan()
                );
                kp
            }
            Err(_) => {
                println!("{}", "Failed to parse keypair bytes. Exiting.".red());
                return;
            }
        }
    } else {
        println!(
            "{}",
            "You must specify either --generate or --import <BASE58>.".red()
        );
        println!("{}", "Use --help for more information.".yellow());
        return;
    };

    // Store keypair as JSON array (Solana CLI compatible)
    write_keypair_file(&keypair, &keypair_path).expect("Failed to write keypair file");
    println!("{} {}", "Keypair saved to".green(), keypair_path.cyan());
}
