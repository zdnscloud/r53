use std::str::from_utf8;

pub struct Parser<'a> {
    raw: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(raw: &'a str) -> Self {
        debug_assert!(raw.len() > 0);

        Parser {
            raw: raw.as_bytes(),
            pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            if self.is_eos() {
                break;
            }
            if self.raw[self.pos].is_ascii_whitespace() {
                self.pos += 1
            } else {
                break;
            }
        }
    }

    pub fn next_txt(&mut self) -> Vec<Vec<u8>> {
        self.skip_whitespace();
        let mut data = Vec::new();
        if self.raw[self.pos] == b'"' {
            let mut last_pos = self.pos + 1;
            let mut in_quote = true;
            let mut start_escape = false;
            self.pos += 1;
            while self.is_eos() == false {
                let c = self.raw[self.pos];
                if c == b'\\' {
                    start_escape = true;
                } else {
                    if c == b'"' && start_escape == false {
                        if in_quote {
                            data.push(self.raw[last_pos..self.pos].to_vec());
                            in_quote = false;
                        } else {
                            in_quote = true;
                            last_pos = self.pos + 1;
                        }
                    }
                    start_escape = false;
                }
                self.pos += 1;
            }
        } else {
            while let Some(s) = self.next_string() {
                data.push(s.as_bytes().to_vec());
            }
        }
        data
    }

    pub fn next_string(&mut self) -> Option<&'a str> {
        self.skip_whitespace();
        let start = self.pos;
        loop {
            if self.is_eos() {
                break;
            }
            if !self.raw[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            None
        } else {
            Some(from_utf8(&self.raw[start..self.pos]).unwrap())
        }
    }

    pub fn into_str(self) -> Option<&'a str> {
        if self.is_eos() {
            None
        } else {
            Some(from_utf8(&self.raw[self.pos..]).unwrap())
        }
    }

    fn is_eos(&self) -> bool {
        self.pos == self.raw.len()
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        self.next_string()
    }
}

#[cfg(test)]
mod test {
    use super::Parser;
    #[test]
    fn test_parser_iterator() {
        let s = " example.org. 100 IN SOA xxx.net. ns.example.org. 100 1800 900 604800 86400    ";
        let mut iter = Parser::new(s);
        let mut split_white = s.split_whitespace();
        let mut label_count = 0;
        loop {
            if let Some(label) = iter.next() {
                assert_eq!(label, split_white.next().unwrap());
                label_count += 1;
            } else {
                break;
            }
        }
        assert_eq!(label_count, 11);
    }

    #[test]
    fn test_into_string() {
        let s = " example.org. 100 IN SOA xxx.net. ns.example.org. 100 1800 900 604800 86400    ";
        let mut iter = Parser::new(s);
        iter.next();
        iter.next();
        assert_eq!(
            iter.into_str().unwrap(),
            " IN SOA xxx.net. ns.example.org. 100 1800 900 604800 86400    "
        );
    }

    #[test]
    fn test_next_txt() {
        let s = " abc edf";
        let data = Parser::new(s).next_txt();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], "abc".as_bytes().to_vec());
        assert_eq!(data[1], "edf".as_bytes().to_vec());

        let s = " \"abc edf\"";
        let data = Parser::new(s).next_txt();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], "abc edf".as_bytes().to_vec());

        let s = " \"abc\\\"c\" \"edf\"";
        let data = Parser::new(s).next_txt();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], "abc\\\"c".as_bytes().to_vec());
        assert_eq!(data[1], "edf".as_bytes().to_vec());
    }
}
