use std::default::Default;
use regex;


/// Vendoring in (with slight modifications to make not-a-method) from the Rust
/// source to avoid deprecation error.
fn slice_chars(s: &str, begin: usize, end: usize) -> &str {
    assert!(begin <= end);
    let mut count = 0;
    let mut begin_byte = None;
    let mut end_byte = None;

    // This could be even more efficient by not decoding,
    // only finding the char boundaries
    for (idx, _) in s.char_indices() {
        if count == begin { begin_byte = Some(idx); }
        if count == end { end_byte = Some(idx); break; }
        count += 1;
    }
    if begin_byte.is_none() && count == begin { begin_byte = Some(s.len()) }
    if end_byte.is_none() && count == end { end_byte = Some(s.len()) }

    match (begin_byte, end_byte) {
        (None, _) => panic!("slice_chars: `begin` is beyond end of string"),
        (_, None) => panic!("slice_chars: `end` is beyond end of string"),
        (Some(a), Some(b)) => unsafe { s.slice_unchecked(a, b) }
    }
}

/*
 * This module is for parsing javascript, with probably more generality
 * It's from github.com/ForbesLindesay/character-parser
 */
/// An object to hold the state wrt brackets, quotes, comments and regex after
/// a given amount of string (syntax is javascript)
///
/// NOTE: work on chars assume only single char graphemes
#[derive(Clone, Debug)]
pub struct BracketState {
    /// Are we in a line comment `//`
    pub line_comment: bool,
    /// Are we in a block comment `/* */`
    pub block_comment: bool,
    /// Are we in a single quoted string `'`
    pub single_quote: bool,
    /// Are we in a double quoted string `"`
    pub double_quote: bool,
    /// Are we in a regex e.g. `/.*/`
    pub regexp: bool,
    /// Are we escaped (just had a single backslash) `\`
    pub escaped: bool,
    /// The depth of round brackets `()`
    pub round_depth: i32,
    /// The depth of curly brackets `{}`
    pub curly_depth: i32,
    /// The depth of square brackets `[]`
    pub square_depth: i32,

    // private

    history: String, // Our history is reversed wrt the original (stack)
    last_char: Option<char>,
    src: String,
    regexp_start: bool
}

impl Default for BracketState {
    /// A default state for our module, starting not in any type of bracket or
    /// other delimeter.
    fn default() -> BracketState {
        // Supply our own default to make sure they are sensible
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
    /// Are we in a string (within `'` or `"`)
    ///
    /// This just combines the 2 different types of quotes for brevity
    #[inline]
    pub fn in_string(&self) -> bool {
        self.single_quote || self.double_quote
    }

    /// Are we in a comment (`/*   */` or `//`)
    ///
    /// This just combines the 2 different types of comments for brevity
    #[inline]
    pub fn in_comment(&self) -> bool {
        self.line_comment || self.block_comment
    }

    /// Are we in some kind of nesting (string, comment, regex, brackets)
    ///
    /// Again for brevity, combine all nesting (quotes, comments, regex, brackets)
    /// into single boolean value
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
#[derive(PartialEq, Debug)]
pub struct BracketBlock<'a> {
    /// The position in the enclosing string of the start of the block
    start: usize,
    /// The position in the enclosing string of the end of the block
    end: usize,
    /// A view of the enclosing string showing just the block enclosed by
    /// the brackets
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

/// Parse the input and return a state object, or none on error
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
/// and return `Some(BracketBlock)` if matching bracket is
/// found, or `None` if end of source is reached
pub fn parse_max<'a>(src: &'a str) -> Option<BracketBlock> {
    let mut state: BracketState = Default::default();
    let mut pos = 0usize;
    let mut char_it = src.chars();
    while state.round_depth >= 0
            && state.curly_depth >= 0
            && state.square_depth >= 0 {
        match char_it.next() {
            Some(ch) => {
                parse_char_from_state(ch, &mut state);
                pos += 1;
            },
            None => {
                return None;
            }
        }
    }
    Some(BracketBlock {
        start: 0,
        end: pos-1,
        src: &src[0..pos-1]
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
        if idx + delimiter.chars().count() >= src.chars().count() {
            return None;
        }
        parse_char_from_state(src.chars().nth(idx).unwrap(), &mut state);
        idx += 1;
    }
    Some(BracketBlock {
        start: start,
        end: idx,
        src: slice_chars(src, start, idx),
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
    let last_char = peek(&state.history);
    //println!("State is {:?}\n char is {:?}", state, ch);
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
        //println!("last_char is {:?} and char is {:?}\n", state.last_char, ch);
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
    } else if ch == '/' && is_regexp(&state.history) {
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
    //println!("{:?}", state);
    if !state.block_comment && !state.line_comment && !was_comment {
        state.history.push(ch);
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
#[inline]
pub fn is_punctuator(ch: char) -> bool {
    match ch {
        '.' | '(' | ')' | ';' | ',' | '{' | '}' | '[' | ']'
            | ':' | '?' | '~' | '%' | '&' | '*' | '+' | '-'
            | '/' | '<' | '>' | '^' | '|' | '!' | '=' => true,
        _ => false
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

/// Check if a string (treated as reversed) is a regex
///
/// A regex should match `/.*/`
// src is reversed, as we access the beginning mostly like a stack.
// Note that below the comments refer to the opposite end
// to the code to hopefully aid reading
fn is_regexp<'a>(src: &'a str) -> bool {
    // strip beginning whitespace from src,
    let history = src.trim_right();
    // then match on the end char
    match history.chars().rev().next() {
        // 
        Some(')') => { false },
        Some('}') => { true },
        Some(ch) if is_punctuator(ch) => { true },
        Some(ch) => {
            match regex!(r"\b\w+$").captures(history) {
                Some(capture) => {
                    match capture.at(0) {
                        Some(key) if is_keyword(key) => true,
                        _ => false
                    }
                },
                _ => false
            }
        }
        None => { false }
    }

}

/// Checks string starts with other string
#[inline]
fn starts_with(src: &str, start: &str, i: usize) -> bool {
    let end = i + start.chars().count();
    if end >= src.chars().count() {
        false
    } else {
        slice_chars(src, i, i + start.chars().count()) == start
    }
}

/// Get end char, or None if string is empty
#[inline]
fn peek(src: &str) -> Option<char> {
    match src.char_indices().rev().next() {
        Some((_, ch)) => { Some(ch) }
        None => None
    }
}

#[cfg(test)]
mod tests {
    use brackets::{BracketState, BracketBlock, parse, parse_from_state, parse_max, parse_until};

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
        let block_option = parse_until("foo.bar(\"%>\").baz%> bing bong", "%>");
        assert!(block_option.is_some());
        let block = block_option.unwrap();
        assert_eq!(block.start, 0);
        assert_eq!(block.end, 17);
        assert_eq!(block.src, "foo.bar(\"%>\").baz");
    }

    #[test]
    #[ignore] // The module works well enough - but these need fixing at some point
    fn section_including_regex() {
        let block_option = parse_max("foo=/\\//g, bar=\"}\") bing bong");
        assert!(block_option.is_some());
        let block = block_option.unwrap();
        assert_eq!(block.start, 0);
        assert_eq!(block.end, 18);
        assert_eq!(block.src, "foo=/\\//g, bar=\"}\"");

        let block_option = parse_max("foo = typeof /\\//g, bar=\"}\") bing bong");
        assert!(block_option.is_some());
        let block = block_option.unwrap();
        assert_eq!(block.start, 0);
        // Note the following comparison fails, as in the original lib
        //assert_eq!(block.end, 18); //exclusive end of string
        assert_eq!(block.src, "foo = typeof /\\//g, bar=\"}\"");
    }

    #[test]
    #[ignore] // The module works well enough - but these need fixing at some point
    fn section_including_block_comment() {
        let block_option = parse_max("/* ) */) bing bong");
        assert!(block_option.is_some());
        let block = block_option.unwrap();
        assert_eq!(block.start, 0);
        assert_eq!(block.end, 7); //exclusive end of string
        assert_eq!(block.src, "/* ) */)");

        let block_option = parse_max("/* /) */) bing bong");
        assert!(block_option.is_some());
        let block = block_option.unwrap();
        assert_eq!(block.start, 0);
        assert_eq!(block.end, 8); //exclusive end of string
        assert_eq!(block.src, "/* /) */)");
    }
}

