use clap::{Parser, Subcommand};
use git2::{Repository, Signature};
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
                    let mut final_rw_content = rw_contents.clone();
                    let current_ro_content =
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
                    if current_ro_content.clone() == rw_contents.clone() {
                        continue;
                    } else {
                        println!("{:?}", current_ro_content);
                        println!("{:?}", rw_contents);
                    }
                    if !final_rw_content.is_empty() && !current_ro_content.is_empty() {
                        final_rw_content.append(current_ro_content);
                    }
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
                }
                let sig = Signature::now("Git-Append", "git@git").unwrap();
                let obj = repo
                    .head()
                    .unwrap()
                    .resolve()
                    .unwrap()
                    .peel(git2::ObjectType::Commit)
                    .unwrap();
                let parent_commit = obj
                    .into_commit()
                    .map_err(|_| git2::Error::from_str("Couldn't find commit"))
                    .unwrap();
                let mut index = repo.index().unwrap();
                index
                    .add_all(["."], git2::IndexAddOption::DEFAULT, None)
                    .unwrap();
                let oid = index.write_tree().unwrap();
                let tree = repo.find_tree(oid).unwrap();
                repo.commit(
                    Some("HEAD"),
                    &sig,
                    &sig,
                    "message",
                    &tree,
                    &[&parent_commit],
                )
                .unwrap()
            }
            Err(e) => panic!("failed to open: {}", e),
        };
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
