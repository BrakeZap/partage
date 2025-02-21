use clap::Args;
use clap::Parser;
use clap::Subcommand;
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
#[derive(Subcommand)]
enum Commands {
    /// Create and upload a file with a random id
    Create(CreateArgs),

    /// Download a file from an id
    Download(DownloadArgs),
}

#[derive(Args)]
struct CreateArgs {
    /// File to upload
    file: String,
}

#[derive(Args)]
struct DownloadArgs {
    /// A valid id to download
    #[arg(value_parser = valid_id)]
    download: String,
}

#[derive(Parser)]
#[command(version, about="A simple cli to share files easily", long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
const LOWER_CHAR_RANGE: std::ops::RangeInclusive<char> = 'a'..='z';
const UPPER_CHAR_RANGE: std::ops::RangeInclusive<char> = 'A'..='Z';
const NUM_RANGE: std::ops::RangeInclusive<u32> = 0..=9;
fn valid_id(id: &str) -> Result<String, String> {
    match id.chars().all(|x| x.is_alphanumeric()) {
        true => {
            match id.chars().all(|x| {
                id.len() == 6
                    && (LOWER_CHAR_RANGE.contains(&x)
                        || UPPER_CHAR_RANGE.contains(&x)
                        || NUM_RANGE.contains(&x.to_digit(10).unwrap()))
            }) {
                true => Ok(id.to_string()),
                false => Err("Id not in range or invalid length".to_string()),
            }
        }
        false => Err("Invalid characters in id.".to_string()),
    }
}

fn generate_id() -> String {
    let mut new_string = String::with_capacity(6);
    let mut rng = rand::rng();
    for _ in 0..6 {
        new_string.push(rng.sample(rand::distr::Alphanumeric) as char);
    }
    new_string
}
#[derive(Deserialize)]
struct ResponseFile {
    file_name: String,
    file: Vec<u8>,
    hash: Vec<u8>,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Create(create_args) => {
            let path = Path::new(&create_args.file);

            if !path.is_file() {
                panic!("Specified file is not a file!");
            }

            //TODO: Compress file before hand?

            //TODO: Encrypt file beforehand

            let mut file = match File::open(path) {
                Ok(f) => f,
                Err(err) => panic!("Could not open file: {}", err),
            };

            let file_name = match path.file_name().unwrap().to_str() {
                Some(name) => name,
                None => panic!("Invalid file name!"),
            };

            println!("Generating checksum...");
            let mut hasher = Sha256::new();
            let _ = io::copy(&mut file, &mut hasher);

            let hash = hasher.finalize();
            let h: Vec<u8> = hash[..].to_vec();
            let mut vec: Vec<u8> = Vec::new();

            println!("Reading file...");
            let _ = file.seek(io::SeekFrom::Start(0));
            let _ = file.read_to_end(&mut vec);

            println!("Uploading file...");
            let client = reqwest::blocking::Client::new();
            let id = generate_id();
            let res = client
                .post("http://localhost:3030/create")
                .json(&json!({"id": id, "file": vec, "hash": h, "file_name": file_name}))
                .send();

            match res {
                Ok(_) => {
                    println!(
                "You have successfully uploaded a new file with the id: {}. Feel free to share it with friends!",
                id
                )
                }
                Err(err) => panic!("Error with uploading the file: {}", err),
            };
        }
        Commands::Download(download_args) => {
            println!("Downloading the file...");

            let response = match reqwest::blocking::get(
                "http://localhost:3030/download/".to_owned() + &download_args.download,
            ) {
                Ok(res) => res,
                Err(err) => panic!("Error downloading the file: {}", err),
            };

            let json = match response.json::<ResponseFile>() {
                Ok(j) => j,
                Err(err) => panic!("A file with that id does not exist: {}", err),
            };

            println!("Writing to file...");

            let mut file = match File::create(json.file_name) {
                Ok(f) => f,
                Err(err) => panic!("Error creating the file, please try again: {}", err),
            };

            let _ = file.write_all(&json.file);
            //TODO: Check file hash
            println!("Completed!");
        }
    }
}
