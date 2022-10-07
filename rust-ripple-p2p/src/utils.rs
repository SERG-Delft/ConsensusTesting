use openssl::sha::sha256;

pub fn public_key_to_b58(key: &[u8]) -> String {
    let type_prefixed_key = [&[28u8], key].concat();
    let checksum = sha256(&sha256(&type_prefixed_key));
    let key = [&type_prefixed_key, &checksum[..4]].concat();
    bs58::encode(key)
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .into_string()
}
