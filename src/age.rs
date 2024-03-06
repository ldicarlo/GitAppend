use std::io::{BufRead, BufReader, Write};

use age::secrecy::Secret;

pub fn encrypt(plaintext: &Vec<Vec<u8>>, passphrase: String) -> Vec<u8> {
    let encryptor = age::Encryptor::with_user_passphrase(Secret::new(passphrase.to_owned()));

    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
    writer.write_all(&plaintext.concat()).unwrap();
    writer.finish().unwrap();
    encrypted
}

pub fn decrypt(encrypted: Vec<u8>, passphrase: String) -> Vec<Vec<u8>> {
    let decryptor = match age::Decryptor::new(&encrypted[..]).unwrap() {
        age::Decryptor::Passphrase(d) => d,
        _ => unreachable!(),
    };

    let reader = decryptor
        .decrypt(&Secret::new(passphrase.to_owned()), None)
        .unwrap();
    let decrypted = BufReader::new(reader).lines();

    decrypted
        .into_iter()
        .map(|l| l.unwrap().as_bytes().into())
        .collect()
}
