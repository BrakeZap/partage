use clap::Args;
use clap::Parser;
use clap::Subcommand;
use rand::Rng;
#[derive(Subcommand)]
enum Commands {
    /// Create a new lobby with a random code
    Create,

    /// Join a lobby with a specified code
    Join(JoinArgs),
}

#[derive(Args)]
struct JoinArgs {
    /// A valid id to join
    #[arg(value_parser = valid_id)]
    join: String,
}

#[derive(Parser)]
#[command(version, about="A simple cli chat client built in Rust!", long_about=None)]
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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Create => println!(
            "You have successfully created a new room with the id: {}",
            generate_id()
        ),
        Commands::Join(join_args) => todo!(),
    }
}
