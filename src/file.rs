use std::fs::File;
use std::path::Path;
use std::{
    fs::{self},
    io::{self, BufRead, BufReader, Write},
};

use crate::config;

pub fn write_to_file(path: &String, content: &Vec<u8>) {
    println!("writing to {}", path);
    fs::create_dir_all(Path::new(path).parent().unwrap()).unwrap();
    let mut file = File::create(path).expect(&format!("Could not find {}", path));
    file.write_all(&content).unwrap();
}

pub fn get_file_contents_as_lines(path: &String) -> io::Result<Vec<Vec<u8>>> {
    let file = File::open(path)?;

    Ok(io::BufReader::new(file)
        .lines()
        .into_iter()
        .flat_map(|l| l.ok())
        .map(|l| l.as_bytes().into())
        .collect())
}

pub fn get_file_contents(path: &String) -> Result<Vec<u8>, std::io::Error> {
    println!("{}", path);
    fs::read(path)
}

pub fn get_file_contents_strip_final_end_line(path: &String) -> Result<Vec<u8>, std::io::Error> {
    fs::read(path).map(|mut s| {
        if s.ends_with(b"\n") {
            s.pop();
            s
        } else {
            s
        }
    })
}

pub fn parse_config(path: String) -> config::Config {
    let file = File::open(path.clone()).expect("Cannot open path.");
    let reader = BufReader::new(file);
    let expected: config::Config =
        serde_json::from_reader(reader).expect(&format!("Invalid format {}", path));
    expected
}

#[cfg(test)]
pub mod tests {
    use crate::{
        config::{self, GitAppender, GitConfig, GitLink},
        parse_config,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_example_config() {
        assert_eq!(
            config::Config {
                appenders: vec![
                    (
                        "/home/someone/repository-location".to_string(),
                        GitAppender {
                            git_config: Some(GitConfig {
                                username: "someone".to_owned(),
                                token_file: "/passwords/github_token".to_owned()
                            }),
                            links: vec![
                                (
                                    "/home/local/plaintext_file".to_string(),
                                    GitLink {
                                        source_path: "file_in_git".to_string(),
                                        password_file: None,
                                        source_branch: Some("chore/special-branch".to_owned()),
                                        remove_lines: Some(
                                            vec![String::from("first_ignored_line")]
                                                .into_iter()
                                                .collect()
                                        ),
                                        exclude_patterns: None
                                    }
                                ),
                                (
                                    "/home/local/encrypted/plaintext_file".to_string(),
                                    GitLink {
                                        source_path: "other_file_in_git".to_string(),
                                        password_file: Some(String::from("/home/password-file")),
                                        source_branch: None,
                                        remove_lines: None,
                                        exclude_patterns: None
                                    }
                                )
                            ]
                            .into_iter()
                            .collect(),
                            folder_links: std::collections::HashMap::new()
                        }
                    ),
                    (
                        "/home/some/other/repository-location".to_string(),
                        GitAppender {
                            git_config: None,
                            links: vec![(
                                "/plaintext_file".to_string(),
                                GitLink {
                                    source_path: "file_in_git".to_string(),
                                    password_file: None,
                                    source_branch: None,
                                    remove_lines: None,
                                    exclude_patterns: None,
                                }
                            ),]
                            .into_iter()
                            .collect(),
                            folder_links: vec![(
                                "/plaintext_folder".to_string(),
                                GitLink {
                                    source_path: "folder_in_git".to_string(),
                                    password_file: None,
                                    source_branch: None,
                                    remove_lines: None,
                                    exclude_patterns: None,
                                }
                            ),]
                            .into_iter()
                            .collect(),
                        }
                    )
                ]
                .into_iter()
                .collect()
            },
            parse_config(String::from("tests/example-config.json"))
        );
    }

    #[test]
    fn test_pdh_config() {
        assert_eq!(
            config::Config {
                appenders: vec![(
                    "/home/<user>/repository-location".to_string(),
                    GitAppender {
                        git_config: Some(GitConfig {
                            username: "<github-user>".to_owned(),
                            token_file: "/passwords/github_token".to_owned()
                        }),
                        links: std::collections::HashMap::new(),
                        folder_links: vec![(
                            "/home/<user>/.directory_history".to_string(),
                            GitLink {
                                source_path: ".directory_history".to_string(),
                                password_file: None,
                                source_branch: None,
                                remove_lines: None,
                                exclude_patterns: None,
                            }
                        ),]
                        .into_iter()
                        .collect(),
                    }
                ),]
                .into_iter()
                .collect()
            },
            parse_config(String::from(
                "tests/example-per-directory-history-config.json"
            ))
        );
    }
}
