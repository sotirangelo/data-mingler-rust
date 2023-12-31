use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    datasources_path: String,
    query_path: String,
    #[arg(short, long, default_value_t = String::from("NONE"))]
    output: String,
    #[arg(short, long, default_value_t = String::from("ALL"))]
    mode: String,
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
