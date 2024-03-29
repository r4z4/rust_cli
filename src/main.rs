//Searches a path for duplicate files
use clap::Parser;
use directories::ProjectDirs;
use serde::Deserialize;
use std::{time::SystemTime, fs};

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

#[derive(Deserialize, Debug)]
struct Config {
    name: String,
    port: Option<i16>,
}

#[derive(Parser)]
enum Commands {
    Search {
        #[clap(long, default_value = ".")]
        path: String,
        #[clap(long, default_value = "")]
        pattern: String,
    },
    Session {
        #[clap(long, default_value = ".")]
        path: String,
        #[clap(long, default_value = "")]
        pattern: String,
        #[clap(long, default_value = "60")]
        time: String,
    },
    StartSession {
        #[clap(long, default_value = "")]
        flag: String,
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

fn main() {
    // Get Config File
    // Linux:   /home/<user>/.config/rust-cli
    // Windows: C:\Users\<User>\AppData\Roaming\r4z4\rust-cli
    // macOS:   /Users/<User>/Library/Application Support/dev.r4z4.rust-cli
    if let Some(dirs) = ProjectDirs::from("dev","r4z4", "rust-cli")
    {
        let config_dir = dirs.config_dir();
        let config_file = fs::read_to_string(
            config_dir.join("config.toml")
        );
        let config =
            match config_file {
                Ok(file) => toml::from_str(&file).unwrap(),
                Err(_) => Config {
                    name: "Me".to_string(),
                    port: Some(4000),
                }
            };
        dbg!(config);
    }
    let _now = SystemTime::now();
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Search { path, pattern }) => {
            println!("Searching for files in {} matching {}", path, pattern);
            let files = rust_cli::walk(&path).unwrap();
            let files = rust_cli::find(files, &pattern);
            println!("Found {} files matching {}", files.len(), pattern);
            for file in files {
                println!("{}", file);
            }
        }
        Some(Commands::Dedupe { path, pattern }) => {
            // Dedupe files matching a pattern
            println!("Deduplicating files in {} matching {}", path, pattern);
            let result = rust_cli::run(&path, &pattern);
            match result {
                Ok(_) => println!("Deduplicating complete"),
                Err(e) => println!("Error: {}", e),
            }
        }
        Some(Commands::StartSession { flag }) => {
            // Dedupe files matching a pattern
            println!("Starting new session w/ {} flag", flag);
            let result = rust_cli::start_session(&flag);
            match result {
                Ok(_) => println!("Session Created"),
                Err(e) => println!("Error: {}", e),
            }
        }
        Some(Commands::Session { path, pattern , time}) => {
            // Remove Session Files
            println!("Removing files from current session {} matching {} within the last {}", path, pattern, time);
            let result = rust_cli::run_session(&path, &pattern, &time);
            match result {
                Ok(_) => println!("Session clear complete"),
                Err(e) => println!("Error: {}", e),
            }
        }
        Some(Commands::Count { path, pattern }) => {
            // Count files matching a pattern
            println!("Counting files in {} matching {}", path, pattern);
            let files = rust_cli::walk(&path).unwrap();
            let files = rust_cli::find(files, &pattern);
            println!("Found {} files matching {}", files.len(), pattern);
        }

        None => {
            println!("No command given");
        }
    }
}