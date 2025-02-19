use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about="A simple cli chat client built in Rust!", long_about=None)]
struct Args {
    #[arg(default_value = "Bob")]
    name: String,
}

fn main() {
    let args = Args::parse();
    println!("Hi {}!", args.name)
}
