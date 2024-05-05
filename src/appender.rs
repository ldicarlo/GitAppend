use std::collections::BTreeSet;

pub fn append(remote_file: Vec<Vec<u8>>, local_file: Vec<Vec<u8>>) -> Option<Vec<u8>> {
    if remote_file == local_file {
        return None;
    }

    let mut r = remote_file.clone();

    r.append(&mut local_file.clone());
    let uniq_final_rw_content: Vec<Vec<u8>> = BTreeSet::from_iter(r).into_iter().collect();
    let end_line_content = last_char(uniq_final_rw_content.join(&b'\n'));
    Some(end_line_content)
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
        assert_eq!(None, append(Vec::new(), Vec::new()));
    }

    #[test]
    fn test_content_1() {
        assert_eq!(
            Some(vec![b'a', b'\n', b'b', b'c', b'\n']),
            append(vec![vec![b'a'], vec![b'b', b'c',]], vec![vec![b'b', b'c']])
        );
    }

    #[test]
    fn test_content_2() {
        assert_eq!(
            Some(vec![b'b', b'c', b'\n']),
            append(vec![], vec![vec![b'b', b'c']])
        );
    }

    #[test]
    fn test_content_3() {
        assert_eq!(
            Some(vec![b'a', b'\n', b'b', b'c', b'\n']),
            append(vec![vec![b'a'], vec![b'b', b'c',]], vec![])
        );
    }
}
