use appender::append;
use clap::{Parser, Subcommand};
use git::{add, commit, signature};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
};
mod appender;
use crate::{
    age::{decrypt, encrypt},
    git::{fetch, open},
};
mod age;
mod config;
mod git;

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Run { config_path } => main_run(config_path),
    }
}

fn main_run(path: String) {
    let configs = parse_config(path);
    for (git_folder, appender) in configs.appenders.iter() {
        let repo = open(git_folder);
        fetch(&repo);
        let mut needs_commit = false;
        for (file_path, file_appender) in appender.iter() {
            let rw_contents = get_file_contents_as_lines(file_path).unwrap_or(Vec::new());
            let final_rw_content = rw_contents.clone();
            let current_ro_content = &mut if let Some(password_file) =
                file_appender.clone().password_file
            {
                let ro_contents = get_file_contents(&file_appender.source).unwrap_or(Vec::new());
                let passphrase = get_file_contents(&password_file).unwrap();
                if ro_contents.is_empty() {
                    Vec::new()
                } else {
                    decrypt(ro_contents, String::from_utf8(passphrase).unwrap())
                }
            } else {
                get_file_contents_as_lines(&file_appender.source).unwrap_or(Vec::new())
            };

            let result = append(current_ro_content.clone(), final_rw_content.clone());

            if let Some(content_to_encrypt) = result {
                needs_commit = true;

                // !  write_to_file(file_path, &end_line_content);
                let final_ro_content =
                    if let Some(password_file) = file_appender.clone().password_file {
                        let passphrase = get_file_contents(&password_file).unwrap();
                        encrypt(&content_to_encrypt, String::from_utf8(passphrase).unwrap())
                    } else {
                        content_to_encrypt
                    };
                write_to_file(&file_appender.source, &final_ro_content);
                add(&repo, file_appender.source.clone());
            } else {
                continue;
            }
        }
        if needs_commit {
            let statuses = repo.statuses(None).unwrap();

            for entry in statuses.iter() {
                let status = entry.status();
                let path = entry.path().unwrap_or_default();

                println!("File: {}, {:?}", path, status.is_index_modified());
            }
            let sign = signature();
            commit(&repo, &sign);
        }
    }
}

fn write_to_file(path: &String, content: &Vec<u8>) {
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
