use std::io::{BufRead, Write};

use age::secrecy::SecretString;

pub fn encrypt(plaintext: &Vec<u8>, passphrase: Box<str>) -> Vec<u8> {
    let encryptor = age::Encryptor::with_user_passphrase(SecretString::new(passphrase));
    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
    writer.write_all(&plaintext).unwrap();
    writer.finish().unwrap();
    encrypted
}

pub fn decrypt(encrypted: Vec<u8>, passphrase: Box<str>) -> Vec<Vec<u8>> {
    let reader = age::decrypt(
        &age::scrypt::Identity::new(SecretString::new(passphrase)),
        &encrypted,
    )
    .unwrap();

    let decrypted = reader.lines();
    decrypted
        .into_iter()
        .map(|l| l.unwrap().as_bytes().into())
        .collect()
}
