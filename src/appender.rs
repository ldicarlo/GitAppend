use std::collections::{BTreeSet, HashSet};
use std::str;

use git2::Repository;

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
) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    let local_hash_set = BTreeSet::from_iter(local_file.clone());
    let remote_hash_set = BTreeSet::from_iter(remote_file.clone());

    // println!("{:?}", String::from_utf8(remote_file.clone().join(&b'\n')));
    // println!("{:?}", String::from_utf8(local_file.clone().join(&b'\n')));

    let mut sum = BTreeSet::new();

    sum.append(&mut remote_hash_set.clone());
    sum.append(&mut local_hash_set.clone());

    let rm_lines_bytes: Vec<Vec<u8>> = remove_lines
        .into_iter()
        .map(|line| line.as_bytes().to_owned())
        .collect();
    sum = sum
        .into_iter()
        .filter(|line| !rm_lines_bytes.contains(line))
        .filter(|line| !line.iter().all(|c| c == &0u8))
        .filter(|line| str::from_utf8(line).is_ok())
        .collect();

    if local_hash_set == remote_hash_set {
        println!("No changes");
        return (None, None);
    }

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

#[cfg(test)]
pub mod tests {
    use std::collections::HashSet;

    use crate::appender::append;

    #[test]
    fn test_content() {
        assert_eq!((None, None), append(Vec::new(), Vec::new(), HashSet::new()));
    }

    #[test]
    fn test_content_1() {
        assert_eq!(
            (Some(vec![b'a', b'\n', b'b', b'c', b'\n',]), None),
            append(
                vec![vec![b'a'], vec![b'b', b'c',]],
                vec![vec![b'b', b'c'],],
                HashSet::new()
            )
        );
    }

    #[test]
    fn test_content_2() {
        assert_eq!(
            (None, Some(vec![b'b', b'c', b'\n'])),
            append(vec![], vec![vec![b'b', b'c']], HashSet::new())
        );
    }

    #[test]
    fn test_content_3() {
        assert_eq!(
            (Some(vec![b'a', b'\n', b'b', b'c', b'\n']), None),
            append(vec![vec![b'a'], vec![b'b', b'c',]], vec![], HashSet::new())
        );
    }
    #[test]
    fn test_content_4() {
        assert_eq!(
            (None, None),
            append(
                vec![vec![b'b'], vec![b'a']],
                vec![vec![b'a'], vec![b'b'],],
                HashSet::new()
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
                vec![String::from("f")].into_iter().collect()
            )
        );
    }
}
