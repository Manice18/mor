use clap::{Parser, Subcommand};
mod commands;
mod utils;

#[derive(Parser)]
#[command(name = "mor-cli")]
#[command(about = "Mor Supply CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Setup your account (generate or import keypair)
    SetupKeypair(commands::setup_keypair::SetupKeypairArgs),
    /// Initialize a mining pool with tokens
    InitializePool {
        /// Amount of tokens to deposit into the pool (in base units)
        amount: u64,
        /// Token mint address
        #[arg(long)]
        token_mint: String,
    },
    /// Create your miner account (can only be run once)
    CreateAccount {
        /// Token mint address
        #[arg(long)]
        token_mint: String,
    },
    /// Delegate your miner to the ER
    DelegateMiner {
        /// Token mint address
        #[arg(long)]
        token_mint: String,
    },
    /// Undelegate your miner from the ER
    UndelegateMiner {
        /// Token mint address
        #[arg(long)]
        token_mint: String,
    },
    /// Get the account details of a miner
    GetAccount {
        /// Token mint address
        #[arg(long)]
        token_mint: String,
    },
    /// Mine for tokens
    Mine {
        /// Token mint address
        #[arg(long)]
        token_mint: String,
    },
    /// Claim rewards from your miner
    ClaimRewards {
        /// Token mint address
        #[arg(long)]
        token_mint: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::SetupKeypair(args) => {
            commands::setup_keypair::handle_setup_keypair(&args);
        }
        Commands::InitializePool { amount, token_mint } => {
            if let Err(e) = commands::initialize::handle_initialize_pool(amount, token_mint) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::CreateAccount { token_mint } => {
            commands::initialize_miner::handle_initialize_miner(token_mint);
        }
        Commands::GetAccount { token_mint } => {
            commands::get_account::handle_get_account(token_mint);
        }
        Commands::Mine { token_mint } => {
            if let Err(e) = commands::mine::handle_mine(token_mint) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::DelegateMiner { token_mint } => {
            commands::delegate_miner::handle_delegate_miner(token_mint);
        }
        Commands::UndelegateMiner { token_mint } => {
            commands::undelegate_miner::handle_undelegate_miner(token_mint);
        }
        Commands::ClaimRewards { token_mint } => {
            commands::claim::handle_claim_rewards(token_mint);
        }
    }
}
