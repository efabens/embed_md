use clap::Parser;
use embed_md::generate;

fn main() {
    let args = Args::parse();
    let id = match args.id {
        None => None,
        Some(x) if x.is_empty() => None,
        Some(_) => Some(args.id.clone().unwrap()),
    };
    generate(args.path.as_str(), id);
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // ID of the embedding to run. If omitted runs all
    #[arg(short, long)]
    id: Option<String>,

    #[arg(default_value = "./")]
    path: String,
}
