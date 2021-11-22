mod definitions_fields;

pub fn deserialize(blob: &[u8]) {
    let mut blob_iterator = BlobIterator { blob };
    // println!("{:?}", blob_iterator.next_n_bytes(50));
}

///
///
/// # Arguments
///
/// * `blob`:
///
/// returns: (type code as u8, field code as u8)
///
/// # Examples
///
/// ```
///
/// ```
fn get_type_field_code(blob: &[u8]) -> (u8, u8) {
    let low_bits = blob[0] & 0b00001111;
    let high_bits = blob[0] & 0b11110000 >> 4;

    return match (high_bits == 0, low_bits == 0) {
        (true, true) => { (blob[1], blob[2]) }
        (false, true) => { (high_bits, blob[1]) }
        (true, false) => { (blob[1], low_bits) }
        (false, false) => { (high_bits, low_bits) }
    }
}

struct BlobIterator<'a> {
    blob: &'a[u8]
}

impl BlobIterator<'_> {

    pub fn next_n_bytes(&mut self, n: usize) -> &[u8] {
        if self.blob.len() < n { panic!("slice out of bounds") };
        let split = self.blob.split_at(n);
        self.blob = split.1;
        split.0
    }

}
