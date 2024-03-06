use std::io::{Read, Write};

use age::secrecy::Secret;

pub fn encrypt(plaintext: &Vec<u8>, passphrase: String) -> Vec<u8> {
    let encryptor = age::Encryptor::with_user_passphrase(Secret::new(passphrase.to_owned()));

    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
    writer.write_all(plaintext).unwrap();
    writer.finish().unwrap();
    encrypted
}

pub fn decrypt(encrypted: Vec<u8>, passphrase: String) -> Vec<u8> {
    let decryptor = match age::Decryptor::new(&encrypted[..]).unwrap() {
        age::Decryptor::Passphrase(d) => d,
        _ => unreachable!(),
    };

    let mut decrypted = vec![];
    let mut reader = decryptor
        .decrypt(&Secret::new(passphrase.to_owned()), None)
        .unwrap();
    reader.read_to_end(&mut decrypted).unwrap();

    decrypted
}
