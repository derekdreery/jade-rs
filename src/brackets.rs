
use std::default::Default;
use core::str::StrExt;

/*
 * This module is for parsing javascript, with probably more generality
 * It's from github.com/ForbesLindesay/character-parser
 */
/// The results of parsing some input
/// NOTE: I haven't really thought about unicode properly
/// (specifically graphemes)
#[derive(Clone)]
pub struct BracketState {
    pub line_comment: bool,
    pub block_comment: bool,

    pub single_quote: bool,
    pub double_quote: bool,
    pub regexp: bool,

    pub escaped: bool,

    pub round_depth: i32,
    pub curly_depth: i32,
    pub square_depth: i32,

    // private

    history: String, // Our history is reversed wrt the original (stack)
    last_char: Option<char>,
    src: String,
    regexp_start: bool
}

impl Default for BracketState {
    /// Supply our own default to make sure they are sensible
    fn default() -> BracketState {
        BracketState {
            line_comment: false,
            block_comment: false,

            single_quote: false,
            double_quote: false,
            regexp: false,

            escaped: false,

            round_depth: 0,
            curly_depth: 0,
            square_depth: 0,

            history: String::new(),
            last_char: None,
            src: String::new(),
            regexp_start: false
        }
    }
}

impl BracketState {
    /// Are we in a string (within ' or ")
    #[inline]
    pub fn in_string(&self) -> bool {
        self.single_quote || self.double_quote
    }

    /// Are we in a comment
    #[inline]
    pub fn in_comment(&self) -> bool {
        self.line_comment || self.block_comment
    }

    /// Are we in some kind of nesting (string, comment, regex, brackets)
    #[inline]
    pub fn in_nesting(&self) -> bool {
        self.in_string()
            || self.in_comment()
            || self.regexp
            || self.round_depth > 0
            || self.curly_depth > 0
            || self.square_depth > 0
    }
}

/// Contains a block of text contained in brackets
#[derive(PartialEq, Show)]
pub struct BracketBlock<'a> {
    start: usize,
    end: usize,
    src: &'a str
}

/// Parse the input and mutate the state object, given the starting state
/// returns true on success, false on error
pub fn parse_from_state<'a>(src: &'a str, state: &mut BracketState) -> bool {
    for ch in src.chars() {
        if state.round_depth < 0 || state.curly_depth < 0 || state.square_depth < 0 {
            return false;
        }
        parse_char_from_state(ch, state);
    }
    true
}

/// Parse the input and return a state object
#[inline]
pub fn parse<'a>(src: &'a str) -> Option<BracketState> {
    let mut state = Default::default();
    if parse_from_state(src, &mut state) {
        Some(state)
    } else {
        None
    }
}

/// Parse until an unmatched (round, curly or square) bracket
/// None if end of source is reached
pub fn parse_max<'a>(src: &'a str) -> Option<BracketBlock> {
    let mut state: BracketState = Default::default();
    let mut pos = 0us;
    let mut char_it = src.chars();
    while state.round_depth >= 0
            && state.curly_depth >= 0
            && state.square_depth >= 0 {
        let next_ch = char_it.next();
        if next_ch == None {
            return None; // Bail out on failure
        }
        // Shouldn't fail because of above check
        parse_char_from_state(next_ch.unwrap(), &mut state);
        pos += 1;
    }
    Some(BracketBlock {
        start: 0,
        end: pos-1,
        src: src.slice_chars(0, pos-1)
    })
}

/// Get the start, end, and substring on the next occurence of delimiter
///
/// # Arguments
///
/// - src - The string to search (haystack)
/// - delimiter - The string to collect until (needle)
/// - start - Char to start searching at (essentially discard beginning of src)
/// - line_comments - True to ignore delimiter if found in line comment
///
/// # Example
///
pub fn parse_until_with_options<'a>(src: &'a str,
                                    delimiter: &str,
                                    start: usize,
                                    line_comments: bool) -> Option<BracketBlock<'a>>
{
    let mut idx = start;
    let mut state: BracketState = Default::default();
    while state.in_string()
        || state.regexp
        || state.block_comment
        || (!line_comments && state.line_comment)
        || !starts_with(src, delimiter, idx)
    {
        if idx + delimiter.char_len() >= src.char_len() {
            return None;
        }
        parse_char_from_state(src.char_at(idx), &mut state);
        idx += 1;
    }
    Some(BracketBlock {
        start: start,
        end: idx,
        src: src.slice_chars(start, idx)
    })
}

/// Get the state on the next occurence of 'delilmeter'
#[inline]
pub fn parse_until<'a>(src: &'a str, delimiter: &str) -> Option<BracketBlock<'a>> {
    parse_until_with_options(src, delimiter, 0, false)
}


/// Parse the next character, given a current state
pub fn parse_char_from_state(ch: char, state: &mut BracketState) {
    state.src.push(ch);
    let was_comment = state.in_comment();
    // NOTE I suspect this will become an option anyway allowing me to remove this
    let last_char = match state.history.len() {
        l if l > 0 => {
            Some(state.history.char_at_reverse(0))
        },
        _ => None
    };
    if state.regexp_start {
        if ch == '/' || ch == '*' {
            state.regexp = false;
        }
        state.regexp_start = false;
    }
    if state.line_comment {
        if ch == '\n' {
            state.line_comment = false;
        }
    } else if state.block_comment {
        if state.last_char == Some('*') && ch == '/' {
            state.block_comment = false;
        }
    } else if state.single_quote {
        if ch == '\'' && !state.escaped {
            state.single_quote = false;
        } else if ch == '\\' && !state.escaped {
            state.escaped = true;
        } else {
            state.escaped = false;
        }
    } else if state.double_quote {
        if ch == '"' && !state.escaped {
            state.double_quote = false;
        } else if ch == '\\' && !state.escaped {
            state.escaped = true;
        } else {
            state.escaped = false;
        }
    } else if state.regexp {
        if ch == '/' && !state.escaped {
            state.regexp = false;
        } else if ch == '\\' && !state.escaped {
            state.escaped = true;
        } else {
            state.escaped = false;
        }
    } else if last_char == Some('/') && ch == '/' {
        state.history.pop();
        state.line_comment = true;
    } else if last_char == Some('/') && ch == '*' {
        state.history.pop();
        state.block_comment = true;
    } else if ch == '/' && is_regexp(state.history.as_slice()) {
        state.regexp = true;
        state.regexp_start = true;
    } else if ch == '\'' {
        state.single_quote = true;
    } else if ch == '"' {
        state.double_quote = true;
    } else if ch == '(' {
        state.round_depth += 1;
    } else if ch == ')' {
        state.round_depth -= 1;
    } else if ch == '{' {
        state.curly_depth += 1;
    } else if ch == '}' {
        state.curly_depth -= 1;
    } else if ch == '[' {
        state.square_depth += 1;
    } else if ch == ']' {
        state.square_depth -= 1;
    }
}

/// Parse a character with default state
#[inline]
pub fn parse_char(ch: char) -> BracketState {
    let mut state = Default::default();
    parse_char_from_state(ch, &mut state);
    state
}

/// Is the character a punctuator?
pub fn is_punctuator(ch: Option<char>) -> bool {
    match ch {
        None => true,
        Some(c) => {
            match c {
                '.' | '(' | ')' | ';' | ',' | '{' | '}' | '[' | ']'
                    | ':' | '?' | '~' | '%' | '&' | '*' | '+' | '-'
                    | '/' | '<' | '>' | '^' | '|' | '!' | '=' => true,
                _ => false
            }
        }
    }
}

/// Is string slice a keyword?
pub fn is_keyword<'a>(src: &'a str) -> bool {
    src == "if"
        || src == "in"
        || src == "do"
        || src == "var"
        || src == "for"
        || src == "new"
        || src == "try"
        || src == "let"
        || src == "this"
        || src == "else"
        || src == "case"
        || src == "vosrc"
        || src == "with"
        || src == "enum"
        || src == "while"
        || src == "break"
        || src == "catch"
        || src == "throw"
        || src == "const"
        || src == "yield"
        || src == "class"
        || src == "super"
        || src == "return"
        || src == "typeof"
        || src == "delete"
        || src == "switch"
        || src == "export"
        || src == "import"
        || src == "default"
        || src == "finally"
        || src == "extends"
        || src == "function"
        || src == "continue"
        || src == "debugger"
        || src == "package"
        || src == "private"
        || src == "interface"
        || src == "instanceof"
        || src == "implements"
        || src == "protected"
        || src == "public"
        || src == "static"
        || src == "yield"
        || src == "let"
}

// Utility
// =======

/// Check if a string is a regex
fn is_regexp<'a>(src: &'a str) -> bool {
    let history = regex!(r"\s*$").replace(src, "");
    let end = match history.len() {
        l if l > 0 => {
            Some(history.char_at_reverse(0))
        },
        _ => None
    };
    if end == Some(')') {
        false
    } else if end == Some('}') || is_punctuator(end) {
        true
    } else {
        match regex!(r"\b\w+$").captures(history.as_slice()) {
            Some(capture) => {
                match capture.at(0) {
                    Some(key) if is_keyword(key) => true,
                    _ => false
                }
            },
            _ => false
        }
    }

}

/// Checks string starts with other string
#[inline]
fn starts_with(src: &str, start: &str, i: usize) -> bool {
    let end = i + start.char_len();
    if end >= src.char_len() {
        false
    } else {
        src.slice_chars(i, i + start.char_len()) == start
    }
}

#[cfg(test)]
mod tests {
    use brackets::{BracketState, BracketBlock, parse, parse_from_state, parse_max};

    #[test]
    fn depth_change_calc() {
        let state_option = parse("foo(arg1, arg2, {\n  foo: [a, b\n");
        assert!(state_option.is_some());
        let mut state = state_option.unwrap();
        assert_eq!(state.round_depth, 1);
        assert_eq!(state.curly_depth, 1);
        assert_eq!(state.square_depth, 1);
        assert!(parse_from_state("    c, d]\n   })", &mut state));
        assert_eq!(state.round_depth, 0);
        assert_eq!(state.curly_depth, 0);
        assert_eq!(state.square_depth, 0);
    }

    #[test]
    fn get_bracketed_section() {
        let block_option = parse_max("foo=\"(\", bar=\"}\") bing bong");
        assert!(block_option.is_some());
        let block = block_option.unwrap();
        assert_eq!(block.start, 0);
        assert_eq!(block.end, 16);
        assert_eq!(block.src, "foo=\"(\", bar=\"}\"");
    }

    #[test]
    fn get_to_delimeter() {
        //let block_option = parser.parse_until //TODO mid-line
    }
}

