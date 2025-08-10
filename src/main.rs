use steganography::Steganography;
mod steganography;
mod converter;
use anyhow::{Result, anyhow};

use clap::{Parser, Subcommand};
use crate::steganography::EncodingLimit;
#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode a file
    Encode {
        value: String,

        /// Input file name (mandatory)
        file_name: String,


        /// Optional key
        #[clap(short, long)]
        key: Option<String>,


        /// Optional output file name
        #[clap(short, long)]
        output: Option<String>,

        #[clap(short, long)]
        limit: Option<EncodingLimit>,

        #[clap(short, long)]
        verbose: bool,

        #[clap(short, long)]
        map: bool,

    },
    /// Decode a file
    Decode {
        /// Optional key
        #[clap(short, long)]
        key: Option<String>,

        /// Input file name (mandatory)
        file_name: String,

        #[clap(short, long)]
        limit: Option<EncodingLimit>
    },
}


fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encode { key, file_name, value, output, limit, verbose , map} => {
            let steganography = Steganography::new(key.clone(), limit.clone());
            steganography.encode(file_name, value, output.clone(), *verbose, *map);
        }
        Commands::Decode { key, file_name, limit } => {
            let steganography = Steganography::new(key.clone(), limit.clone());
            match steganography.decode(file_name) {
                Ok(x) => {
                    println!("{}", x);
                }
                Err(e) => {
                    eprintln!("Decode error: {:?}", e);
                    return Err(anyhow!(e));
                }
            }
        }
    }

    Ok(())
}

