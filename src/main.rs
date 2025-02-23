use clap::Args;
use clap::Parser;
use clap::Subcommand;
use colored::Colorize;
use rand::Rng;
use reqwest::blocking::multipart;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::File;
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
    ///Ip to use
    ip: String,

    /// File to upload
    file: String,
}

#[derive(Args)]
struct DownloadArgs {
    ///Ip to use
    ip: String,

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
struct FileProp {
    file_name: String,
    file_size: String,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Create(create_args) => {
            let path = Path::new(&create_args.file);

            if !path.is_file() {
                println!("{}", "Specified file is not a file!".red());
                return;
            }
            //TODO: Compress file before hand?

            //TODO: Encrypt file beforehand

            let mut file = match File::open(path) {
                Ok(f) => f,
                Err(_) => {
                    println!("{}", "Could not open file".red());
                    return;
                }
            };

            let mut vec: Vec<u8> = Vec::new();

            println!("Reading file...");
            let _ = file.read_to_end(&mut vec);

            let mut hasher = Sha256::new();
            hasher.update(&vec[..]);
            let hash = hasher.finalize();
            let h: Vec<u8> = hash[..].to_vec();

            println!("Uploading file...");
            let client = reqwest::blocking::Client::new();
            let id = generate_id();

            //TODO: Send file in chunks if it is large

            const CHUNK_SIZE: usize = 1024 * 1024; //1 MB

            let mut buffer: Vec<u8> = vec![0; CHUNK_SIZE];

            let mut chunk_num: usize = 0;

            let total_chunks: usize = vec.len().div_ceil(CHUNK_SIZE);

            let mut reader = std::io::Cursor::new(vec.clone());
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            loop {
                let read = reader.read(&mut buffer).unwrap();

                if read == 0 {
                    break;
                    //return;
                }

                let form = multipart::Form::new()
                    .text("file_name", name.to_string())
                    .text("id", id.clone())
                    .text("chunk_num", chunk_num.to_string())
                    .text("total_chunks", total_chunks.to_string())
                    .part("hash", multipart::Part::bytes(h.to_vec()))
                    .part("file", multipart::Part::bytes(buffer[..read].to_vec()));

                let res = client
                    .post(create_args.ip.clone() + "/create")
                    .multipart(form)
                    .send();

                match res {
                    Ok(r) => {
                        if !r.status().is_success() {
                            println!(
                                "{} {:?}",
                                "Error with uploading the file:".red(),
                                r.status()
                            );
                            return;
                        }
                    }
                    Err(_) => {
                        println!(
                            "{}",
                            "Error with uploading the file... Please try again.".bright_red()
                        );
                        return;
                    }
                };

                chunk_num += 1;
            }

            println!(
                "{} {} {}",
                "You have successfully uploaded a new file with the id:"
                    .bold()
                    .green(),
                id,
                "Feel free to share it with friends!".bold().green(),
            )
        }
        Commands::Download(download_args) => {
            //Send get request to check if file with specified id exists on the server
            //
            //Response: File name and size
            //
            //
            //Send a set of get requests that specify the range of bytes wanted by the client
            //Response: Receive that set of bytes of the file
            //
            //After all bytes are sent, check hash of byte array
            //
            //Write all bytes into the file
            //
            let res = match reqwest::blocking::get(format!(
                "{}/download/{}",
                download_args.ip, download_args.download
            )) {
                Ok(r) => r,
                Err(_) => {
                    println!("Error, requesting server!");
                    return;
                }
            };

            let json_file_prop = match res.json::<FileProp>() {
                Ok(j) => j,
                Err(_) => {
                    println!("No file with that id found!");
                    return;
                }
            };

            let mut file = match File::create(json_file_prop.file_name) {
                Ok(f) => f,
                Err(_) => {
                    println!("{}", "Error creating the file".red());
                    return;
                }
            };

            let file_size: usize = json_file_prop.file_size.parse::<usize>().unwrap();

            const CHUNK_SIZE: usize = 1024 * 1024;
            let mut current: usize = 0;
            loop {
                if current >= file_size {
                    break;
                }

                let mut r = match reqwest::blocking::get(format!(
                    "{}/download/{}/{}/{}",
                    download_args.ip,
                    download_args.download,
                    current,
                    current + CHUNK_SIZE
                )) {
                    Ok(r) => r,
                    Err(_) => {
                        println!("Error with retrieving the range");
                        return;
                    }
                };

                if !r.status().is_success() {
                    println!("Error fetching range data: {}", r.status());
                    return;
                }

                let mut buf: Vec<u8> = vec![];
                let _ = r.copy_to(&mut buf).unwrap();
                let _ = file.write_all(&buf);

                current += CHUNK_SIZE;
            }

            println!("{}", "Completed!".green());
        }
    }
}
