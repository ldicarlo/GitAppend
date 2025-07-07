use git2::Repository;

use crate::{
    age::encrypt,
    appender::{append, get_from_appender},
    config::{GitConfig, GitLink},
    file::{get_file_contents, get_file_contents_strip_final_end_line, write_to_file},
    git::{fetch, open},
    parse_config,
};

pub fn decrypt_file(path: String, repository_location: String, file: String) {
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
    // println!(
    //     "{}",
    //     String::from_utf8(get_from_appender(file_appender, &repo).join(&b'\n')).unwrap()
    // );
}
use crate::get_file_contents_as_lines;
use std::collections::HashSet;

pub fn process_file(
    file_appender: &GitLink,
    file_path: &String,
    repo_file_path: String,
    git_folder: &String,
    repo: &Repository,
) -> (Vec<String>, bool) {
    log::info!("Processing: {}", file_path);
    let mut needs_commit = false;
    let mut files = Vec::new();
    let rm_lines = file_appender.clone().remove_lines.unwrap_or(HashSet::new());
    let rw_contents = get_file_contents_as_lines(file_path).unwrap_or(Vec::new());
    let final_rw_content = rw_contents.clone();
    let current_ro_content = &mut get_from_appender(file_appender, &repo, &repo_file_path);

    let (local_result, remote_result) = append(
        current_ro_content.clone(),
        final_rw_content.clone(),
        rm_lines.clone(),
    );
    log::info!(
        "Result was (local {},remote {})",
        local_result.is_some(),
        remote_result.is_some()
    );

    // println!("result: {:?}", result.clone().map(|r| String::from_utf8(r)));
    if let Some(local_content) = local_result {
        write_to_file(file_path, &local_content.clone());
    }
    if let Some(content_to_encrypt) = remote_result {
        needs_commit = true;

        let final_ro_content = if let Some(password_file) = file_appender.clone().password_file {
            let passphrase = get_file_contents(&password_file).unwrap();
            encrypt(
                &content_to_encrypt,
                String::from_utf8(passphrase).unwrap().into_boxed_str(),
            )
        } else {
            content_to_encrypt
        };
        write_to_file(
            &(git_folder.to_owned() + "/" + &repo_file_path),
            &final_ro_content,
        );
        files.push(repo_file_path.clone());
    }
    (files, needs_commit)
}
