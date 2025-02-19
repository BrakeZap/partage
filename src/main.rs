use clap::Args;
use clap::Parser;
use clap::Subcommand;

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
const ID_RANGE: std::ops::RangeInclusive<char> = 'a'..='Z';
fn valid_id(id: &str) -> Result<String, String> {
    match id.chars().all(|x| ID_RANGE.contains(&x)) {
        true => Ok(id.to_string()),
        false => Err("Invalid id.".to_string()),
    }
}

fn main() {
    let args = Cli::parse();
}
