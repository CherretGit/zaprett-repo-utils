use clap::Parser;
use zaprett_repo_utils::cli::Cli;
use zaprett_repo_utils::run;

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("{}", e)
    }
}
