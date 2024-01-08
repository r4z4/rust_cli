//Searches a path for duplicate files
use clap::Parser;

#[derive(Parser)]
// Extend / Custom help info
#[clap(
    version = "1.0",
    author = "r4z4",
    about = "Find session files and remove",
    after_help = "Example: rust_cli remove --path . --pattern .txt"
)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser)]
enum Commands {
    Search {
        #[clap(long, default_value = ".")]
        path: String,
        #[clap(long, default_value = "")]
        pattern: String,
    },
    Dedupe {
        #[clap(long, default_value = ".")]
        path: String,
        #[clap(long, default_value = "")]
        pattern: String,
    },
    //create count with path and pattern defaults for both
    Count {
        #[clap(long, default_value = ".")]
        path: String,
        #[clap(long, default_value = "")]
        pattern: String,
    },
}

