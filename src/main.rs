use std::{fs::File, io::BufReader};

use clap::{Parser, Subcommand};

mod config;

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Run { config_path } => main_run(config_path),
    }
}

fn main_run(path: String) {
    let configs = parse_config(path);
    for config in configs.appenders.iter() {
        println!("git pull {:?}", config);
        println!("decrypt {:?}", config);
        println!("merge {:?}", config);
        println!("git push {:?}", config);
    }
}

fn parse_config(path: String) -> config::Config {
    let file = File::open(path.clone()).expect("Cannot open path.");
    let reader = BufReader::new(file);
    let expected: config::Config =
        serde_json::from_reader(reader).expect(&format!("Invalid format {}", path));
    println!("{:?}", expected);
    expected
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Run {
        /// Configuration file location
        #[arg(short, long)]
        config_path: String,
    },
}

#[cfg(test)]
pub mod tests {
    use crate::{
        config::{self, Appender},
        parse_config,
    };

    #[test]
    fn test_example_config() {
        assert_eq!(
            config::Config {
                settings: "hello".to_string(),
                appenders: vec![(
                    "/etc/file/to/sync".to_string(),
                    Appender {
                        git_folder_location: "/home/someone/folder/.git".to_string()
                    }
                )]
                .into_iter()
                .collect()
            },
            parse_config(String::from("tests/example-config.json"))
        );
    }
}
