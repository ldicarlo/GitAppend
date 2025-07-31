use std::collections::{BTreeSet, HashSet};
use std::str;

use git2::Repository;
use regex::Regex;

use crate::config::Feature;
use crate::{age::decrypt, config::GitLink, file::get_file_contents, git::get_blob_from_head};

pub fn get_from_appender(
    file_appender: &GitLink,
    repo: &Repository,
    repo_file_path: &String,
) -> Vec<Vec<u8>> {
    let source_branch = "http-origin/".to_owned()
        + &file_appender
            .clone()
            .source_branch
            .unwrap_or("master".to_owned());

    let content = get_blob_from_head(repo, repo_file_path, source_branch.to_owned());

    if let Some(password_file) = file_appender.clone().password_file {
        let ro_contents = content;
        let passphrase = get_file_contents(&password_file).unwrap();
        if ro_contents.is_empty() {
            Vec::new()
        } else {
            decrypt(
                ro_contents,
                String::from_utf8(passphrase).unwrap().into_boxed_str(),
            )
        }
    } else {
        let res = content.split(|c| c == &b'\n');
        res.map(|s| s.into()).collect()
    }
}

pub fn append(
    remote_file: Vec<Vec<u8>>,
    local_file: Vec<Vec<u8>>,
    remove_lines: HashSet<String>,
    exclude_patterns: HashSet<String>,
    features: HashSet<Feature>,
) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    let local_hash_set = BTreeSet::from_iter(
        apply_feature_rmb(
            local_file,
            features.contains(&Feature::RemoveMultilinesBash),
        )
        .into_iter()
        .filter(|line| !line.is_empty())
        .clone(),
    );
    let remote_hash_set = BTreeSet::from_iter(
        apply_feature_rmb(
            remote_file,
            features.contains(&Feature::RemoveMultilinesBash),
        )
        .into_iter()
        .filter(|line| !line.is_empty())
        .clone(),
    );

    let mut sum = BTreeSet::new();

    sum.append(&mut remote_hash_set.clone());
    sum.append(&mut local_hash_set.clone());

    let exclude_patterns: Vec<Regex> = exclude_patterns
        .into_iter()
        .map(|ep| Regex::new(&ep).expect(&format!("Fail to read regex: {}", ep)))
        .collect();

    let rm_lines_bytes: Vec<Vec<u8>> = remove_lines
        .into_iter()
        .map(|line| line.as_bytes().to_owned())
        .collect();
    sum = sum
        .into_iter()
        .filter(|line| !line.is_empty())
        .filter(|line| !rm_lines_bytes.contains(line))
        .filter(|line| !line.iter().all(|c| c == &0u8))
        .filter(|line| str::from_utf8(line).is_ok())
        .filter(|line| {
            let str = str::from_utf8(line);
            match str {
                Ok(new_string) => !exclude_patterns.iter().any(|re| re.is_match(new_string)),
                Err(_) => false,
            }
        })
        .collect();

    // if local_hash_set == remote_hash_set {
    //     println!("No changes");
    //     return (None, None);
    // }

    let joined_sum: Vec<Vec<u8>> = sum.clone().into_iter().collect();
    let sum_with_endline = last_char(joined_sum.join(&b'\n'));

    let local_result = if local_hash_set == sum {
        None
    } else {
        Some(sum_with_endline.clone())
    };

    let remote_result = if remote_hash_set == sum {
        None
    } else {
        Some(sum_with_endline)
    };

    (local_result, remote_result)
}

fn last_char(mut content: Vec<u8>) -> Vec<u8> {
    if let Some(char) = content.last() {
        if char != &b'\n' {
            content.push(b'\n');
        }
    }
    content
}

fn apply_feature_rmb(content: Vec<Vec<u8>>, feature: bool) -> Vec<Vec<u8>> {
    if feature {
        return feature_remove_multilines_bash(content);
    }
    content
}

fn feature_remove_multilines_bash(content: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    content
        .into_iter()
        .fold(
            (Vec::new(), None) as (Vec<Vec<u8>>, Option<Vec<u8>>),
            |(mut acc, maybe_line), mut current| {
                let current_is_multi = current.ends_with(&[b'\\']);
                if current_is_multi {
                    current.remove(current.len() - 1);
                    current.push(b' ');
                }

                if let Some(mut line) = maybe_line {
                    let _ = &line.append(&mut current.clone());

                    if current_is_multi {
                        (acc, Some(line))
                    } else {
                        acc.push(line);
                        (acc, None)
                    }
                } else if current_is_multi {
                    (acc, Some(current))
                } else {
                    acc.push(current);
                    (acc, None)
                }
            },
        )
        .0
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashSet;

    use pretty_assertions::assert_eq;

    use crate::{
        appender::{append, feature_remove_multilines_bash},
        file::get_file_contents_as_lines,
    };

    #[test]
    fn hashset_eq() {
        assert_eq!(
            vec![String::from("Hello")]
                .into_iter()
                .collect::<HashSet<String>>(),
            vec![String::from("Hello")]
                .into_iter()
                .collect::<HashSet<String>>()
        );
    }

    #[test]
    fn test_content() {
        assert_eq!(
            (None, None),
            append(
                Vec::new(),
                Vec::new(),
                HashSet::new(),
                HashSet::new(),
                HashSet::new()
            )
        );
    }

    #[test]
    fn test_content_1() {
        assert_eq!(
            (Some(vec![b'a', b'\n', b'b', b'c', b'\n',]), None),
            append(
                vec![vec![b'a'], vec![b'b', b'c',]],
                vec![vec![b'b', b'c'],],
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            )
        );
    }

    #[test]
    fn test_content_2() {
        assert_eq!(
            (None, Some(vec![b'b', b'c', b'\n'])),
            append(
                vec![],
                vec![vec![b'b', b'c']],
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            )
        );
    }

    #[test]
    fn test_content_3() {
        assert_eq!(
            (Some(vec![b'a', b'\n', b'b', b'c', b'\n']), None),
            append(
                vec![vec![b'a'], vec![b'b', b'c',]],
                vec![],
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            )
        );
    }
    #[test]
    fn test_content_4() {
        assert_eq!(
            (None, None),
            append(
                vec![vec![b'b'], vec![b'a']],
                vec![vec![b'a'], vec![b'b'],],
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            )
        );
    }

    #[test]
    fn test_content_5() {
        assert_eq!(
            (
                Some(vec![b'a', b'\n', b'b', b'c', b'\n',]),
                Some(vec![b'a', b'\n', b'b', b'c', b'\n',])
            ),
            append(
                vec![vec![b'a'], vec![b'b', b'c',], vec![b'f']],
                vec![vec![b'b', b'c'],],
                vec![String::from("f")].into_iter().collect(),
                HashSet::new(),
                HashSet::new(),
            )
        );
    }

    #[test]
    fn test_content_6() {
        assert_eq!(
            (
                Some(vec![b'b', b'c', b'\n',]),
                Some(vec![b'b', b'c', b'\n',])
            ),
            append(
                vec![vec![b'd', b'\\',], vec![b'b', b'c',], vec![b'f']],
                vec![vec![b'e', b'\\',], vec![b'b', b'c'],],
                vec![String::from("f")].into_iter().collect(),
                vec![String::from(".*\\\\$")].into_iter().collect(),
                HashSet::new(),
            )
        );
    }

    #[test]
    fn test_remove_multilines_feature() {
        let input = get_file_contents_as_lines(&String::from("tests/multilines")).unwrap();

        let result = vec![
            ": this is a multiline command".as_bytes().to_owned(),
            ": this is not".as_bytes().to_owned(),
        ];

        assert_eq!(
            result
                .clone()
                .into_iter()
                .map(|s| std::str::from_utf8(&s).unwrap().to_string())
                .collect::<Vec<String>>(),
            feature_remove_multilines_bash(input.clone())
                .into_iter()
                .map(|s| std::str::from_utf8(&s).unwrap().to_string())
                .collect::<Vec<String>>(),
        );
        assert_eq!(result, feature_remove_multilines_bash(input));
    }
}
