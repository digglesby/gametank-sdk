pub mod builder;

use clap::{Parser, Subcommand};

use crate::builder::RomBuilder;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Configure {
        /// gtrom.toml to configure llvm shit. 
        /// By default checks for a rustup mos toolchain, then checks for a podman or docker container
        config_file: Option<String> 
    },
    
    Build {

    }
}

fn main() {
    let cli = Cli::parse();

    // TODO: check for 

    match cli.command {
        Commands::Configure { config_file } => println!("not implemented"),
        Commands::Build {  } => {
            let rb = RomBuilder::init("/home/dewbrite/code/personal/gametank-sdk/rom".to_string());
        },
    }
}
