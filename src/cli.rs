use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "MyApp", about = "An example of StructOpt usage.")]
struct Cli {
    #[structopt(short = "c", long = "config", help = "Set a custom config file")]
    config: String,
}

pub fn run() {
    let args = Cli::from_args();
    println!("Using config file: {}", args.config);
}
