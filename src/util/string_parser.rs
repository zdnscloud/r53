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

    pub fn next_char(&mut self) -> Option<char> {
        if self.is_eos() {
            None
        } else {
            Some(self.unsafe_next_char())
        }
    }

    fn unsafe_next_char(&mut self) -> char {
        let c = self.raw[self.pos];
        self.pos += 1;
        c as char
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

    fn into_str(self) -> Option<&'a str> {
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
}
