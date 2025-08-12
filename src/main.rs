use steganography::Steganography;
mod steganography;
mod converter;
mod transformer;
mod traverser;

use anyhow::{Result, anyhow};

use clap::{Parser, Subcommand};
use crate::steganography::{ColorChannel, EncodingLimit};
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
        /// Value that will be encoded
        value: String,

        /// Input file name (mandatory)
        input_file: String,
        
        /// Optional key
        #[clap(short, long)]
        key: Option<String>,

        /// Optional output file name
        #[clap(short, long)]
        output: Option<String>,

        /// Default Limit is 16 Bytes
        #[clap(short, long)]
        limit: Option<EncodingLimit>,

        /// Default is rgb
        #[clap(short, long)]
        channel: Option<ColorChannel>,
        
        #[clap(short, long)]
        verbose: bool,

        /// Produces an exaggerated representation alongside the encoded output
        #[clap(short, long)]
        map: bool,
        
        /// Treats VALUE as a file
        #[clap(short, long)]
        file: bool

    },
    /// Decode a file
    Decode {
        /// Optional key
        #[clap(short, long)]
        key: Option<String>,

        /// Input file name (mandatory)
        input_file: String,

        /// Default Limit is 16 Bytes
        #[clap(short, long)]
        limit: Option<EncodingLimit>,

        /// Default is rgb
        #[clap(short, long)]
        channel: Option<ColorChannel>,

        /// If specified the result will be produced in a form of a file
        #[clap(short, long)]
        file: Option<String>
    },
}


fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encode { key, input_file: file_name, value, output, limit, verbose , map, file, channel} => {
            let steganography = Steganography::new(key.clone(), limit.clone(), channel.clone());
            let value = 
            if *file {
                transformer::file_to_b64(value)?
            }
            else {
                value.clone()
            };
            steganography.encode(file_name, &*value, output.clone(), *verbose, *map);
        }
        Commands::Decode { key, input_file: file_name, limit, file , channel} => {
            let steganography = Steganography::new(key.clone(), limit.clone(), channel.clone());
            match steganography.decode(file_name) {
                Ok(x) => {
                    match file {
                        None => {
                            println!("{}", x);
                        }
                        Some(file) => {
                            transformer::b64_to_file(&*x, file)?;
                            println!("Saved file to {}", file);
                        }
                    }
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

