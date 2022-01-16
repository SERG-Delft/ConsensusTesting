pub struct BlobIterator<'a> {
    blob: &'a [u8],
}

impl<'b> BlobIterator<'b> {
    pub fn len(&self) -> usize {
        self.blob.len()
    }

    pub fn new(blob: &'b [u8]) -> Self {
        BlobIterator { blob }
    }

    pub fn has_next(&self) -> bool {
        0 < self.blob.len()
    }

    pub fn peek(&self) -> u8 {
        self.blob[0]
    }

    pub fn next_byte(&mut self) -> u8 {
        self.next_n_bytes(1)[0]
    }

    pub fn next_n_bytes(&mut self, n: usize) -> &[u8] {
        if self.blob.len() < n { panic!("slice out of bounds") };
        let split = self.blob.split_at(n);
        self.blob = split.1;
        split.0
    }

    pub fn last_byte(&mut self) -> u8 {
        self.last_n_bytes(1)[0]
    }

    pub fn last_n_bytes(&mut self, n: usize) -> &[u8] {
        if self.blob.len() < n { panic!("slice out of bounds") };
        let split = self.blob.split_at(self.blob.len() - n);
        self.blob = split.0;
        split.1
    }
}
