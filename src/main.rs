use clap::{Parser, Subcommand};
use git2::Repository;
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
};

use crate::age::{decrypt, encrypt};
mod age;
mod config;

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Run { config_path } => main_run(config_path),
    }
}

fn main_run(path: String) {
    let configs = parse_config(path);
    for (git_folder, appender) in configs.appenders.iter() {
        println!("git pull {:?}", git_folder);
        match Repository::open(git_folder) {
            Ok(repo) => {
                let _ = repo.fetchhead_foreach(|r, _, c, _| {
                    println!("Updated {} to {} ({})", git_folder, c, r);
                    true
                });
                for (file_path, file_appender) in appender.iter() {
                    let rw_contents = get_file_contents_as_lines(file_path).unwrap_or(Vec::new());
                    let mut final_rw_content = rw_contents;
                    let new_rw_content =
                        &mut if let Some(password_file) = file_appender.clone().password_file {
                            let ro_contents =
                                get_file_contents(&file_appender.source).unwrap_or(Vec::new());
                            let passphrase = get_file_contents(&password_file).unwrap();
                            if ro_contents.is_empty() {
                                Vec::new()
                            } else {
                                decrypt(ro_contents, String::from_utf8(passphrase).unwrap())
                            }
                        } else {
                            get_file_contents_as_lines(&file_appender.source).unwrap_or(Vec::new())
                        };
                    final_rw_content.append(new_rw_content);
                    let uniq_final_rw_content: Vec<Vec<u8>> =
                        BTreeSet::from_iter(final_rw_content).into_iter().collect();
                    write_to_file(file_path, uniq_final_rw_content.clone().join(&b'\n'));
                    let final_ro_content =
                        if let Some(password_file) = file_appender.clone().password_file {
                            let passphrase = get_file_contents(&password_file).unwrap();
                            encrypt(
                                &uniq_final_rw_content,
                                String::from_utf8(passphrase).unwrap(),
                            )
                        } else {
                            uniq_final_rw_content.join(&b'\n')
                        };
                    write_to_file(&file_appender.source, final_ro_content);
                    println!("merge {:?}", file_path);
                }
            }
            Err(e) => panic!("failed to open: {}", e),
        };
        println!("git push {:?}", git_folder);
    }
}

fn write_to_file(path: &String, content: Vec<u8>) {
    let mut file = File::create(path).unwrap();
    file.write_all(&content).unwrap();
}

fn get_file_contents_as_lines(path: &String) -> io::Result<Vec<Vec<u8>>> {
    let file = File::open(path)?;

    Ok(io::BufReader::new(file)
        .lines()
        .into_iter()
        .map(|l| l.unwrap().as_bytes().into())
        .collect())
}

fn get_file_contents(path: &String) -> Result<Vec<u8>, std::io::Error> {
    fs::read(path)
}

fn parse_config(path: String) -> config::Config {
    let file = File::open(path.clone()).expect("Cannot open path.");
    let reader = BufReader::new(file);
    let expected: config::Config =
        serde_json::from_reader(reader).expect(&format!("Invalid format {}", path));
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
                    "/home/someone/folder/.git".to_string(),
                    vec![
                        (
                            "plaintext_file".to_string(),
                            Appender {
                                source: "file_in_git".to_string(),
                                password_file: None,
                            }
                        ),
                        (
                            "other_plaintext_file".to_string(),
                            Appender {
                                source: "other_file_in_git".to_string(),
                                password_file: Some(String::from("./test")),
                            }
                        )
                    ]
                    .into_iter()
                    .collect()
                )]
                .into_iter()
                .collect()
            },
            parse_config(String::from("tests/example-config.json"))
        );
    }
}
