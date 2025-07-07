use crate::{
    appender::append,
    core::{decrypt_file, process_file},
    file::{get_file_contents_as_lines, get_file_contents_strip_final_end_line, parse_config},
    git::{fetch, open},
};
use clap::{Parser, Subcommand};
use config::GitConfig;
use git::{commit_and_push, signature};
use glob::glob;
use log::{debug, LevelFilter};
use std::collections::HashSet;
mod age;
mod appender;
mod config;
mod core;
mod encryption;
mod file;
mod git;

fn main() {
    env_logger::init();
    std::env::set_var("RUST_LOG", "debug");
    let _ = systemd_journal_logger::JournalLog::new().unwrap().install();
    log::set_max_level(LevelFilter::Debug);
    let args = Cli::parse();
    debug!("LOG test");
    match args.command {
        Commands::Run { config_path } => main_run(config_path),
        Commands::Cat {
            config_path,
            file,
            repository_location,
        } => decrypt_file(config_path, repository_location, file),
        Commands::CatAppend { file_one, file_two } => {
            let file_one = get_file_contents_as_lines(&file_one).unwrap_or(Vec::new());
            let file_two = get_file_contents_as_lines(&file_two).unwrap_or(Vec::new());

            let (result, _) = append(file_one, file_two, HashSet::new());
            println!("{:?}", result);
        }
    }
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
                    String::from_utf8(
                        get_file_contents_strip_final_end_line(&token_file)
                            .expect(&format!("Could not find {}", token_file)),
                    )
                    .expect(&format!("Error converting from utf8")),
                )
            },
        );
        fetch(&repo, credentials.clone(), "master".to_owned());
        //pull(&repo, credentials.clone());
        let mut needs_commit = false;
        for (file_path, file_appender) in appender.links.iter() {
            let (new_files, new_needs_commit) = process_file(
                file_appender,
                file_path,
                file_appender.source_path.to_owned(),
                git_folder,
                &repo,
            );
            needs_commit = needs_commit || new_needs_commit;
            files.extend(new_files);
        }
        for (file_path, folder_appender) in appender.folder_links.iter() {
            for entry in glob(&format!("{}/**/*", file_path)).expect("Failed to read glob pattern")
            {
                let (new_files, new_needs_commit) = match entry {
                    Ok(path) => {
                        if path.is_file() {
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
                            println!("Ignored folder or link: {:?}", path);
                            (Vec::new(), false)
                        }
                    }
                    Err(e) => {
                        println!("Ignored: {:?}", e);
                        (Vec::new(), false)
                    }
                };
                needs_commit = needs_commit || new_needs_commit;
                files.extend(new_files);
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
