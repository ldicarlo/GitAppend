use std::collections::BTreeSet;

pub fn append(
    remote_file: Vec<Vec<u8>>,
    local_file: Vec<Vec<u8>>,
) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    let local_hash_set = BTreeSet::from_iter(local_file.clone());
    let remote_hash_set = BTreeSet::from_iter(remote_file.clone());

    // println!("{:?}", String::from_utf8(remote_file.clone().join(&b'\n')));
    // println!("{:?}", String::from_utf8(local_file.clone().join(&b'\n')));

    let mut sum = BTreeSet::new();

    sum.append(&mut remote_hash_set.clone());
    sum.append(&mut local_hash_set.clone());

    if local_hash_set == remote_hash_set {
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
    use crate::appender::append;

    #[test]
    fn test_content() {
        assert_eq!((None, None), append(Vec::new(), Vec::new()));
    }

    #[test]
    fn test_content_1() {
        assert_eq!(
            (Some(vec![b'a', b'\n', b'b', b'c', b'\n']), None),
            append(vec![vec![b'a'], vec![b'b', b'c',]], vec![vec![b'b', b'c']])
        );
    }

    #[test]
    fn test_content_2() {
        assert_eq!(
            (None, Some(vec![b'b', b'c', b'\n'])),
            append(vec![], vec![vec![b'b', b'c']])
        );
    }

    #[test]
    fn test_content_3() {
        assert_eq!(
            (Some(vec![b'a', b'\n', b'b', b'c', b'\n']), None),
            append(vec![vec![b'a'], vec![b'b', b'c',]], vec![])
        );
    }
    #[test]
    fn test_content_4() {
        assert_eq!(
            (None, None),
            append(vec![vec![b'b'], vec![b'a']], vec![vec![b'a'], vec![b'b'],],)
        );
    }
}
