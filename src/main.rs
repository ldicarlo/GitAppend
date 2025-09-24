use crate::{
    appender::append,
    core::{decrypt_file, process_file},
    file::{get_file_contents_as_lines, get_file_contents_strip_final_end_line, parse_config},
    git::{open, pull},
};
use clap::{Parser, Subcommand};
use config::GitConfig;
use git::{commit_and_push, signature};
use glob::glob;
use std::collections::HashSet;
mod age;
mod appender;
mod config;
mod core;
mod encryption;
mod file;
mod git;

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Run {
            config_path,
            include_appender,
        } => main_run(config_path, include_appender),
        Commands::Cat {
            config_path,
            file,
            repository_location,
        } => decrypt_file(config_path, repository_location, file),
        Commands::CatAppend { file_one, file_two } => {
            let file_one_content = get_file_contents_as_lines(&file_one).unwrap_or(Vec::new());
            let file_two_content = get_file_contents_as_lines(&file_two).unwrap_or(Vec::new());

            let (local, remote) = append(
                file_one_content,
                file_two_content,
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            );
            println!("{}: {:?}", file_one, local);
            println!("{}: {:?}", file_two, remote);
        }
    }
}

fn main_run(path: String, maybe_include_appender: Option<String>) {
    let configs = parse_config(path);

    let appenders = maybe_include_appender
        .map(|include_appender| {
            configs
                .appenders
                .clone()
                .into_iter()
                .filter(|(k, _)| k == &include_appender)
                .collect()
        })
        .unwrap_or(configs.appenders);

    for (git_folder, appender) in appenders.iter() {
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
                    String::from_utf8(
                        get_file_contents_strip_final_end_line(&token_file)
                            .expect(&format!("Could not find {}", token_file)),
                    )
                    .expect(&format!("Error converting from utf8")),
                )
            },
        );
        pull(&repo, credentials.clone(), "master".to_owned());

        for (file_path, file_appender) in appender.links.iter() {
            let new_files = process_file(
                file_appender,
                file_path,
                file_appender.source_path.to_owned(),
                git_folder,
                &repo,
            );

            files.extend(new_files);
        }
        for (file_path, folder_appender) in appender.folder_links.iter() {
            for entry in glob(&format!("{}/**/*", file_path)).expect("Failed to read glob pattern")
            {
                let new_files = match entry {
                    Ok(path) => {
                        if path.is_file() && !path.to_str().unwrap().contains(".git") {
                            let local_path = path.strip_prefix(file_path).unwrap();

                            process_file(
                                folder_appender,
                                &format!("{}", path.display()),
                                format!(
                                    "{}/{}",
                                    folder_appender.source_path.to_owned(),
                                    local_path.display(),
                                ),
                                git_folder,
                                &repo,
                            )
                        } else {
                            println!("Ignored folder or link: {:?} (or in .git folder)", path);
                            Vec::new()
                        }
                    }
                    Err(e) => {
                        println!("Ignored: {:?}", e);
                        Vec::new()
                    }
                };

                files.extend(new_files);
            }
        }

        if !files.is_empty() {
            let sign = signature();
            commit_and_push(&repo, credentials.clone(), &sign);
            pull(&repo, credentials.clone(), "master".to_owned());
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Debug, Subcommand)]
enum Commands {
    /// This is the default command to run, from where you want, as a service (see `tests/example-config.json`).
    #[command(arg_required_else_help = true)]
    Run {
        /// Configuration file location (see `tests/example-config.json`).
        #[arg(short, long)]
        config_path: String,

        #[arg(long)]
        include_appender: Option<String>,
    },
    /// Read a file as the run command would read it, to see what it contains, from your config file.
    #[command(arg_required_else_help = true)]
    Cat {
        /// Configuration file location (see `tests/example-config.json`).
        #[arg(short, long)]
        config_path: String,

        /// Repository location
        #[arg(short, long)]
        repository_location: String,

        /// File to decrypt (for testing/debugging purposes)
        #[arg(short, long)]
        file: String,
    },

    /// Output the result of the append merge between two files.
    #[command(arg_required_else_help = true)]
    CatAppend {
        /// File 1 (for testing/debugging purposes)
        #[arg(long)]
        file_one: String,

        /// File 2 (for testing/debugging purposes)
        #[arg(long)]
        file_two: String,
    },
}
