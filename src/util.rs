/**
 * TODO use &'t str rather than String (investigate)
 */
use regex::Regex;
use std::fmt;

#[deriving(PartialEq)]
pub struct LexerString {
    src: String,
    pos: uint
}

impl LexerString {
    pub fn new(src: String) -> LexerString {
        LexerString {
            src: src,
            pos: 0
        }
    }

    #[inline]
    pub fn consume(&mut self, amt: uint) -> &str {
        self.pos += amt;
        self.src[self.pos - amt .. self.pos] // as if the consume didn't happen
    }

    #[inline]
    pub fn lookahead(&self, amt: uint) -> &str {
        self.src[self.pos .. self.pos + amt]
    }

    #[inline]
    pub fn peek(&self) -> char {
        self.src.char_at(self.pos)
    }

    /**
     * Note: regex must start with '^' (match from beginning)
     */
    pub fn scan(&mut self, re: Regex) -> Option<String> {
        assert!(re.as_str().char_at(0), '^')
        let mut consume_len = 0;
        let res = match re.captures(self.src[self.pos..]) {
            Some(captures) => {
                if captures.is_empty() {
                    return None
                }
                consume_len = captures.at(0).len();
                Some(captures.at(1).into_string())
            }
            _ => None
        };
        self.consume(consume_len);
        res
    }
}

impl fmt::Show for LexerString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LexerString {{ src: {}, pos: {} }}", self.src, self.pos)
    }
}

#[test]
fn lexerstring_new() {
    let ls = LexerString::new("function testfn() { }".to_string());
    assert_eq!(ls, LexerString {
        src: "function testfn() { }".to_string(),
        pos: 0
    })
}

#[test]
fn lexerstring_consume() {
    let test_str = "function testfn() { }";
    let mut ls = LexerString::new(test_str.to_string());
    {
        let first_consume = ls.consume(10);
        assert_eq!(first_consume, "function t");
    }
    {
        let second_consume = ls.consume(5);
        assert_eq!(second_consume, "estfn")
    }
    assert_eq!(ls, LexerString {
        src: test_str.to_string(),
        pos: 15
    })
}

#[test]
fn lexerstring_lookahead() {
    let test_str = "function testfn() { }";
    let ls = LexerString::new(test_str.to_string());
    assert_eq!(ls.lookahead(5), "funct")
}

#[test]
fn lexerstring_peek() {
    let test_str = "function testfn() { }";
    let ls = LexerString::new(test_str.to_string());
    assert_eq!(ls.peek(), 'f')
}

#[test]
fn lexerstring_scan() {
    let test_str = "function testfn() { }";
    let ls = LexerString::new(test_str.to_string());
}
