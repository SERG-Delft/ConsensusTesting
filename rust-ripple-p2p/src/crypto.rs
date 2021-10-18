use std::convert::TryInto;
use sha2::{Sha512, Digest};
use bs58;
use ripple_address_codec::{encode_seed, Secp256k1};

pub fn derive_seed(pass_phrase: &String) -> [u8; 16] {
    Sha512::digest(pass_phrase.as_bytes())[..16]
        .try_into()
        .expect("Unable to convert to array with size 16")
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use ripple_address_codec::Entropy;
    use super::*;

    struct Setup {
        genesis_passphrase: String,
        genesis_seed: String,
        genesis_private_key: String,
        genesis_address: String,
        account_id: String,
        master_key: String,
        master_seed: String,
        master_seed_hex: String,
        public_key: String,
        public_key_hex: String
    }

    impl Setup {
        fn new() -> Self {
            Self {
                genesis_passphrase: "masterpassphrase".to_string(),
                genesis_seed: "snoPBrXtMeMyMHUVTgbuqAfg1SUTb".to_string(),
                genesis_private_key: "".to_string(),
                genesis_address: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
                account_id : "rE4DHSdcXafD7DkpJuFCAvc3CvsgXHjmEJ".to_string(),
                master_key : "BUSY MARS SLED SNUG OBOE REID SUNK NEW GYM LAD LICE FEAT".to_string(),
                master_seed : "saNSJMEBKisBr6phJtGXUcV85RBZ3".to_string(),
                master_seed_hex : "FDDE6A91607445E59C6F7CF07AF7B661".to_string(),
                public_key : "aBQsqGF1HEduKrHrSVzNE5yeCTJTGgrsKgyjNLgabS2Rkq7CgZiq".to_string(),
                public_key_hex : "03137FF01C82A1CF507CC243EBF629A99F2256FA43BCB7A458F638AF9A5488CD87".to_string()
            }
        }
    }

    #[test]
    fn derive_seed() {
        let setup = Setup::new();
        let entropy: Entropy = super::derive_seed(&setup.genesis_passphrase);
        let seed = ripple_address_codec::encode_seed(&entropy,&Secp256k1);
        assert_eq!(seed, setup.genesis_seed);
    }
}