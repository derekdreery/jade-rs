
use tokens;
use regex;
use std::fmt;

/**
 * The Lexer struct
 *
 * This struct takes an input string, and returns
 * it as a sequence of tokens for parsing/compiling
 */
#[deriving(PartialEq)]
pub struct Lexer {
    input: String,
    filename: Option<String>,
    position: uint,
    deferred_tokens: Vec<tokens::Token>,
    last_indents: uint,
    line_number: uint,
    stash: Vec<tokens::Token>,
    indent_stack: (),
    indent_regex: (),
    pipeless: bool
}

impl Lexer {
    /// Allows for filename to be specified - use new or new_with_filename
    #[inline]
    fn new_with_option(input: String, filename: Option<String>) -> Lexer {
        Lexer {
            input: input,
            filename: filename,
            position: 0,
            deferred_tokens: Vec::new(),
            last_indents: 0,
            line_number: 1,
            stash: Vec::new(),
            indent_stack: (),
            indent_regex: (),
            pipeless: false
        }
    }

    /// New lexer from input, with a filename for error reports
    #[inline]
    pub fn new_with_filename(input: String, filename: String) -> Lexer {
        Lexer::new_with_option(input, Some(filename))
    }

    /// New lexer from input
    #[inline]
    pub fn new(input: String) -> Lexer {
        Lexer::new_with_option(input, None)
    }

    /// Consume amt number of bytes of the input, returning
    /// it as a slice
    #[inline]
    pub fn consume(&mut self, amt: uint) -> &str {
        self.position += amt;
        self.input[self.position - amt .. self.position] // as if the consume didn't happen
    }


    /// Return the next char
    #[inline]
    pub fn peek(&self) -> char {
        self.input.char_at(self.position)
    }

    /**
     * Scan for a regex and create a simple token on match
     *
     */
    pub fn scan(&mut self, re: regex::Regex) -> Option<String> {
        let (res, consume_len) = match re.captures(self.input[self.position..]) {
            // Fail if match failed
            Some(captures) if captures.is_empty() => {
                (None, 0)
            },
            Some(captures) => {
                match captures.pos(0) {
                    // Fail if not matched from beginning
                    Some((x, _)) if x > 0 => {
                        (None, 0)
                    },
                    Some((x, y)) if x == 0 => {
                        (
                            Some(captures.at(1).into_string()),
                            captures.at(0).len()
                        )
                    },
                    _ => (None, 0)
                }
            },
            None => {(None, 0)}
        };
        self.consume(consume_len);
        res
    }

    /// Push token onto stack for later use
    pub fn defer(&mut self, tok: tokens::Token) {
        self.deferred_tokens.push(tok)
    }

    /// Return amt tokens
    #[inline]
    pub fn lookahead(&self, amt: uint) -> &str {
        self.input[self.position .. self.position + amt]
    }
}

impl fmt::Show for Lexer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lexer {{ input: {}, position: {} }}", self.input, self.position)
    }
}

// Tests
// =====

#[cfg(test)]
mod tests {
    use lexer::Lexer;

    fn jade_block() -> String {
        "doctype html\
         html(lang=\"en\")\
             head\
                 title This is the title\
             body\
                 h1 This is a heading\
                 p This is a paragraph".to_string()
    }

    #[test]
    fn lexer_new() {
        let ls = Lexer::new(jade_block());
        assert_eq!(ls, Lexer {
                input: jade_block(),
                filename: None,
                position: 0,
                deferred_tokens: Vec::new(),
                last_indents: 0,
                line_number: 1,
                stash: Vec::new(),
                indent_stack: (),
                indent_regex: (),
                pipeless: false
        })
    }

    #[test]
    fn lexer_consume() {
        let test_str = "function testfn() { }";
        let mut ls = Lexer::new(test_str.to_string());
        assert_eq!(ls.consume(10), "function t");
        assert_eq!(ls.consume(5), "estfn")
        assert_eq!(ls.position, 15)
    }

    #[test]
    #[ignore]
    fn lexer_lookahead() {
        let test_str = "function testfn() { }";
        let ls = Lexer::new(test_str.to_string());
        assert_eq!(ls.lookahead(5), "funct")
    }

    #[test]
    #[ignore]
    fn lexer_peek() {
        let test_str = "function testfn() { }";
        let ls = Lexer::new(test_str.to_string());
        assert_eq!(ls.peek(), 'f')
    }

    #[test]
    fn lexer_scan() {
        let test_str = "function testfn() { }";
        let ls = Lexer::new(test_str.to_string());

    }

    #[test]
    fn lexer_complex() {
    }
}
