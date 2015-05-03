
use regex;
use std::fmt;

/// Represents block types
#[derive(PartialEq, Debug, Clone)]
pub enum BlockType {
    Append,
    Prepend,
    Replace
}

/// Represents token value types
#[derive(PartialEq, Debug)]
pub enum ValueType {
    String(String)
}

/// Represets token types
#[derive(PartialEq, Debug)]
pub enum TokenType {
    /// Some(Nothing) = no-op, but restart looking
    /// None = carry on looking)
    Nothing,
    /// No more input and in outermost indent
    EndOfSource,
    /// Move in a level
    Indent,
    /// Move out a level
    Outdent,
    /// Simple text token
    Text(String),
    /// Comment token with contents of comment
    /// (buffer = true <=> render comment in html)
    Comment(Option<String>, bool), // message, buffer
    /// TODO not sure :P
    Interpolation(String),
    PipelessText,
    Yield,
    Doctype,
    Case,
    When,
    Default,
    Extends,
    Block{ block_type: BlockType },
    MixinBlock,
    Include(String),
    Attrs(Vec<String>)
}

/// A parsed token from input
#[derive(PartialEq, Debug)]
pub struct Token {
    token_type: TokenType,
    line_number: u32,
}

impl Token {
    /// quick constructor
    pub fn new(token_type: TokenType, line_number: u32) -> Token {
        Token {
            token_type: token_type,
            line_number: line_number
        }
    }
}

/// A struct to pass the necessary information to the lexer
/// from a token matcher method
#[derive(PartialEq, Debug)]
struct TokenResult {
    /// The matched token
    token: Token,
    /// The amount of input to consume
    input_increment: usize,
    /// The number of lines to consume
    line_increment: u32
}

impl TokenResult {
    /// A quick constructor function
    #[inline]
    fn new(token: Token, input_increment: usize, line_increment: u32) -> TokenResult {
        TokenResult {
            token: token,
            input_increment: input_increment,
            line_increment: line_increment
        }
    }
}

/**
 * The Lexer struct
 *
 * This struct takes an input string, and returns
 * it as a sequence of tokens for parsing/compiling
 */
#[derive(PartialEq)]
pub struct Lexer<'a> {
    input: &'a str,
    filename: Option<String>,
    position: usize,
    deferred_tokens: Vec<Token>,
    last_indents: u32,
    line_number: u32,
    stash: Vec<Token>,
    indent_stack: Vec<Token>,
    indent_regex: (),
    pipeless: bool
}

impl<'a> Lexer<'a> {
    /// Allows for filename to be specified - use new or new_with_filename
    #[inline]
    fn new_with_option(input: &'a str, filename: Option<String>) -> Lexer {
        Lexer {
            input: input,
            filename: filename,
            position: 0,
            deferred_tokens: Vec::new(),
            last_indents: 0,
            line_number: 1,
            stash: Vec::new(),
            indent_stack: Vec::new(),
            indent_regex: (),
            pipeless: false
        }
    }

    /// New lexer from input, with a filename for error reports
    #[inline]
    pub fn new_with_filename(input: &'a str, filename: String) -> Lexer {
        Lexer::new_with_option(input, Some(filename))
    }

    /// New lexer from input
    #[inline]
    pub fn new(input: &'a str) -> Lexer {
        Lexer::new_with_option(input, None)
    }

    /// Get remaining input as slice
    #[inline]
    fn get_input(&self) -> &str {
        &self.input[self.position..]
    }

    /// Create a new token, with line number
    /// The sole purpose of this function is to add line number
    #[inline]
    fn tok(&self, token_type: TokenType) -> Token {
        Token::new(token_type, self.line_number)
    }

    /// Consume amt number of bytes of the input, returning
    /// it as a slice
    #[inline]
    pub fn consume(&mut self, amt: usize) -> &str {
        self.position += amt;
        &self.input[self.position - amt .. self.position] // as if the consume didn't happen
    }


    /// Return the next char
    #[inline]
    pub fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    /**
     * Scan for a regex and create a simple token on match
     * TODO I think I can remove this
     */
    pub fn scan(&mut self, re: regex::Regex) -> Option<String> {
        let (res, consume_len) = match re.captures(&self.input[self.position..]) {
            // Fail if match failed
            Some(captures) => {
                match captures.pos(0) {
                    // Fail if not matched from beginning
                    Some((x, _)) if x > 0 => {
                        (None, 0)
                    },
                    Some((x, y)) if x == 0 => {
                        match captures.at(1) {
                            Some(cap) => (
                                Some(cap.to_string()),
                                // We have already tested for existence,
                                // so safe to unwrap
                                captures.at(0).unwrap().len()
                            ),
                            _ => (None, 0)
                        }
                    },
                    _ => (None, 0)
                }
            },
            None => (None, 0)
        };
        self.consume(consume_len);
        res
    }

    /// Push token onto stack for later use
    #[inline]
    pub fn defer(&mut self, tok: Token) {
        self.deferred_tokens.push(tok)
    }

    /// Return amt tokens
    pub fn lookahead(&mut self, amt: usize) -> &Token {
        let len = amt - self.stash.len();
        for _ in 1..len {
            let next = self.next();
            self.stash.push(next);
        }
        &self.stash[amt-1]
    }

    /// Get the contents of a bracketed expression
    pub fn bracket_expression(&self, skip: u32) {
        
    }

    /// Pop off the token stash
    #[inline]
    pub fn stashed(&mut self) -> Option<Token> {
        self.stash.pop()
    }

    /// Pop off the deferred token stack
    #[inline]
    pub fn deferred(&mut self) -> Option<Token> {
        self.deferred_tokens.pop()
    }

    /// Get the next token
    pub fn next(&mut self) -> Token {
        self.tok(TokenType::Outdent) // TODO placeholder
    }

    /// Test the input against a rule
    fn test(&mut self, f: fn(&str) -> Option<TokenResult>) -> Option<Token> {
        match f(self.get_input()) {
            Some(res) => {
                self.consume(res.input_increment);
                self.line_number = self.line_number + res.line_increment;
                Some(res.token)
            },
            _ => None
        }
    }

    // Tokens
    // ======

    /// End of source. Need mut ref to pop indent_stack
    fn eos(&mut self) -> Option<TokenResult> {
        if self.position != self.input.len() {
            None
        } else {
            if self.indent_stack.len() > 0 {
                self.indent_stack.pop();
                Some(TokenResult::new(self.tok(TokenType::Outdent), 0, 0))
            } else {
                Some(TokenResult::new(self.tok(TokenType::EndOfSource), 0, 0))
            }
        }
    }

    /// Blank line
    fn blank(&self) -> Option<TokenResult> {
        match regex!(r"^\n *\n").find(self.get_input()) {
            Some((0, end)) => {
                let res = &self.get_input()[..end];
                if self.pipeless {
                    Some(TokenResult::new(self.tok(TokenType::Text("".to_string())), end-1, 1))
                } else {
                    Some(TokenResult::new(self.tok(TokenType::Nothing), end-1, 1))
                }
            },
            _ => None
        }
    }

    /// Comment ('//-' is not output in html)
    fn comment(&mut self) -> Option<TokenResult> {
        let mut pipeless = self.pipeless;
        let res = match regex!(r"^//(-)?([^\n]*)").captures(self.get_input()) {
            Some(capture) => {
                pipeless = true;
                let comment = match capture.at(2) {
                    Some(msg) => Some(msg.to_string()),
                    None => None
                };
                Some(TokenResult::new(
                    self.tok(TokenType::Comment(comment, capture.at(1) == None)),
                    capture.at(0).unwrap().len(), // must be Some<>
                    0
                ))

            }
            None => None
        };
        self.pipeless = pipeless;
        res
    }

    // TODO what is this?
    // TODO doing bracket matching is hard. Jade.js uses a lib, should I?
    /*
    fn interpolation(&self) -> Option<TokenResult> {
        match regex!(r"^#\{").is_match(self.get_input()) {
            true => {
                // TODO I've just stopped mid line :P
            },
            false => None
        }
    }*/

}

impl <'a> fmt::Debug for Lexer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lexer {{ input: {}, position: {} }}", self.input, self.position)
    }
}

// ======================
//
// Tests
//
// ======================

#[cfg(test)]
mod tests {
    use lexer::{Token, TokenType, TokenResult, Lexer};
    use regex;

    fn jade_block<'a>() -> &'a str {
        concat!(
            "doctype html",
            "html(lang=\"en\")",
            "  head",
            "    title= pageTitle",
            "    script(type='text/javascript').",
            "      if (foo) {",
            "         bar(1 + 5)",
            "      }",
            "  body",
            "    h1 Jade - node template engine",
            "    #container.col",
            "      if youAreUsingJade",
            "        p You are amazing",
            "      else",
            "        p Get on it!",
            "      p.",
            "        Jade is a terse and simple",
            "        templating language with a",
            "        strong focus on performance",
            "        and powerful features.")
    }

    #[test]
    fn new() {
        let ls = Lexer::new(jade_block());
        assert_eq!(ls, Lexer {
                input: jade_block(),
                filename: None,
                position: 0,
                deferred_tokens: Vec::new(),
                last_indents: 0,
                line_number: 1,
                stash: Vec::new(),
                indent_stack: Vec::new(),
                indent_regex: (),
                pipeless: false
        })
    }

    #[test]
    fn consume() {
        let test_str = "function testfn() { }";
        let mut ls = Lexer::new(test_str);
        assert_eq!(ls.consume(10), "function t");
        assert_eq!(ls.consume(5), "estfn");
        assert_eq!(ls.position, 15);
    }

    #[test]
    #[ignore] // need tokens working to test this
    fn lookahead() {
        let test_str = "function testfn() { }";
        let ls = Lexer::new(test_str);
    }

    #[test]
    #[ignore] // need tokens working to test this
    fn peek() {
        let test_str = "function testfn() { }";
        let ls = Lexer::new(test_str);
    }

    // TODO use token-like strings to test
    #[test]
    fn scan() {
        let test_str = "function testfn() { }";
        let mut ls = Lexer::new(test_str);
        // first regex should match "function" and capture "unc"
        let re = regex::Regex::new(r"[fg](unc)tion").unwrap();
        assert_eq!(ls.scan(re), Some("unc".to_string()));
        // second regex should fail
        let re2 = regex::Regex::new(r" ?(\(\)) ").unwrap();
        assert_eq!(ls.scan(re2), None);
        // third regex should match " testfn" and capture "testfn"
        let re3 = regex::Regex::new(r" ?(t?e?s?t?t?f?n+)").unwrap();
        assert_eq!(ls.scan(re3), Some("testfn".to_string()));
    }

    #[test]
    fn eos() {
        let mut true1 = Lexer::new("");
        let mut false1 = Lexer::new("notend");
        assert_eq!(true1.eos(), Some(TokenResult::new(
            true1.tok(TokenType::EndOfSource), 0, 0
        )));
        assert_eq!(false1.eos(), None);
    }

    #[test]
    fn blank() {
        let true1 = Lexer::new("\n        \n");
        let true2 = Lexer::new("\n\n");
        let false1 = Lexer::new("\nSome text this line\n");
        assert_eq!(true1.blank(), Some(TokenResult::new(
            true1.tok(TokenType::Nothing), 9, 1
        )));
        assert_eq!(true2.blank(), Some(TokenResult::new(
            true2.tok(TokenType::Nothing), 1, 1
        )));
        assert_eq!(false1.blank(), None);
    }

    #[test]
    fn comment() {
        let mut true1 = Lexer::new("// This is a comment\nThis is another line");
        let mut true2 = Lexer::new("//- This is an unbuffered comment");
        let mut false1 = Lexer::new("This is not a comment // this is not the next token");
        assert_eq!(true1.comment(), Some(TokenResult::new(
            true1.tok(TokenType::Comment(Some(" This is a comment".to_string()), true)),
            "// This is a comment".len(),
            0
        )));
        assert_eq!(true2.comment(), Some(TokenResult::new(
            true1.tok(TokenType::Comment(
                Some(" This is an unbuffered comment".to_string()),
                false
            )),
            "//- This is an unbuffered comment".len(),
            0
        )));
        assert_eq!(false1.comment(), None);
    }

    #[test]
    fn complex() {
    }
}
