use std::io::Read;

pub struct CharReader<R: Read> {
    reader: R,
    buffer: Vec<char>,
    last: char,
}

impl<R: Read> CharReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: vec![],
            last: '\0',
        }
    }
}

impl<R: Read> Iterator for CharReader<R> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let mut byte = [0u8];
        match self.reader.read_exact(&mut byte) {
            Ok(_) => {
                self.last = byte[0] as char;
                Some(self.last)
            }
            Err(_) => None,
        }
    }
}
