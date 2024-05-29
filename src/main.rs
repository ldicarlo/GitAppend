use appender::append;
use clap::{Parser, Subcommand};
use config::{GitConfig, GitLink};
use git::{commit_and_push, get_blob_from_head, signature};
use git2::Repository;
use std::{
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
    env_logger::init();
    let args = Cli::parse();

    match args.command {
        Commands::Run { config_path } => main_run(config_path),
        Commands::Decrypt {
            config_path,
            file,
            repository_location,
        } => decrypt_file(config_path, repository_location, file),
    }
}

fn decrypt_file(path: String, repository_location: String, file: String) {
    let configs = parse_config(path);
    let (git_folder, appender) = configs
        .appenders
        .iter()
        .find(|(k, _)| **k == repository_location)
        .expect("Appender not found in config");
    let (_, file_appender) = appender
        .links
        .iter()
        .find(|(_, s)| s.source_path == file)
        .expect("File not in config");
    let repo = open(&format!("{}/.git", git_folder));
    let credentials = appender.git_config.clone().map(
        |GitConfig {
             username,
             token_file,
         }| {
            (
                username,
                String::from_utf8(get_file_contents_strip_final_end_line(&token_file).unwrap())
                    .unwrap(),
            )
        },
    );
    let source_branch = file_appender
        .clone()
        .source_branch
        .unwrap_or("master".to_owned());
    fetch(&repo, credentials, source_branch);
    log::info!(
        "{}",
        String::from_utf8(get_from_appender(file_appender, &repo).join(&b'\n')).unwrap()
    );
}

fn main_run(path: String) {
    let configs = parse_config(path);
    for (git_folder, appender) in configs.appenders.iter() {
        let mut files = Vec::new();
        let repo = open(&format!("{}/.git", git_folder));
        // let c = repo.config().unwrap();
        // c.entries(None)
        //     .unwrap()
        //     .for_each(|d| log::debug!("{:?}:{:?}", d.name(), d.value()))
        //     .unwrap();
        let credentials = appender.git_config.clone().map(
            |GitConfig {
                 username,
                 token_file,
             }| {
                (
                    username,
                    String::from_utf8(get_file_contents_strip_final_end_line(&token_file).unwrap())
                        .unwrap(),
                )
            },
        );
        fetch(&repo, credentials.clone(), "master".to_owned());
        //pull(&repo, credentials.clone());
        let mut needs_commit = false;
        for (file_path, file_appender) in appender.links.iter() {
            let rw_contents = get_file_contents_as_lines(file_path).unwrap_or(Vec::new());
            let final_rw_content = rw_contents.clone();
            let current_ro_content = &mut get_from_appender(file_appender, &repo);
            //   log::debug!("rw: {:?}", final_rw_content);
            //   log::debug!("ro: {:?}", current_ro_content);

            let result = append(current_ro_content.clone(), final_rw_content.clone());
            //  log::debug!("result: {:?}", result);

            if let Some(content_to_encrypt) = result {
                needs_commit = true;

                write_to_file(file_path, &content_to_encrypt);
                let final_ro_content =
                    if let Some(password_file) = file_appender.clone().password_file {
                        let passphrase = get_file_contents(&password_file).unwrap();
                        encrypt(&content_to_encrypt, String::from_utf8(passphrase).unwrap())
                    } else {
                        content_to_encrypt
                    };
                write_to_file(
                    &(git_folder.to_owned() + &"/" + &file_appender.source_path),
                    &final_ro_content,
                );
                files.push(file_appender.source_path.clone());
            }
        }
        if needs_commit {
            let statuses = repo.statuses(None).unwrap();

            for entry in statuses.iter() {
                let status = entry.status();
                let path = entry.path().unwrap_or_default();

                log::debug!("File: {}, {:?}", path, status.is_index_modified());
            }
            let sign = signature();
            commit_and_push(&repo, credentials, &sign, files);
        }
    }
}

fn get_from_appender(file_appender: &GitLink, repo: &Repository) -> Vec<Vec<u8>> {
    let source_branch = "http-origin/".to_owned()
        + &file_appender
            .clone()
            .source_branch
            .unwrap_or("master".to_owned());

    let content = get_blob_from_head(
        repo,
        file_appender.clone().source_path,
        source_branch.to_owned(),
    );

    if let Some(password_file) = file_appender.clone().password_file {
        let ro_contents = content;
        let passphrase = get_file_contents(&password_file).unwrap();
        if ro_contents.is_empty() {
            Vec::new()
        } else {
            decrypt(ro_contents, String::from_utf8(passphrase).unwrap())
        }
    } else {
        let res = content.split(|c| c == &b'\n');
        res.map(|s| s.into()).collect()
    }
}

fn write_to_file(path: &String, content: &Vec<u8>) {
    log::debug!("writing to {}", path);
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
    log::debug!("{}", path);
    fs::read(path)
}

fn get_file_contents_strip_final_end_line(path: &String) -> Result<Vec<u8>, std::io::Error> {
    fs::read(path).map(|mut s| {
        if s.ends_with(b"\n") {
            s.pop();
            s
        } else {
            s
        }
    })
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
    #[command(arg_required_else_help = true)]
    Decrypt {
        /// Configuration file location
        #[arg(short, long)]
        config_path: String,

        /// Repository location
        #[arg(short, long)]
        repository_location: String,

        /// File to decrypt (for testing/debugging purposes)
        #[arg(short, long)]
        file: String,
    },
}

#[cfg(test)]
pub mod tests {
    use crate::{
        config::{self, GitAppender, GitLink},
        parse_config,
    };

    #[test]
    fn test_example_config() {
        assert_eq!(
            config::Config {
                settings: "hello".to_string(),
                appenders: vec![(
                    "/home/someone/folder/.git".to_string(),
                    GitAppender {
                        git_config: None,
                        links: vec![
                            (
                                "plaintext_file".to_string(),
                                GitLink {
                                    source_path: "file_in_git".to_string(),
                                    password_file: None,
                                    source_branch: None
                                }
                            ),
                            (
                                "other_plaintext_file".to_string(),
                                GitLink {
                                    source_path: "other_file_in_git".to_string(),
                                    password_file: Some(String::from("./test")),
                                    source_branch: None
                                }
                            )
                        ]
                        .into_iter()
                        .collect()
                    }
                )]
                .into_iter()
                .collect()
            },
            parse_config(String::from("tests/example-config.json"))
        );
    }
}
