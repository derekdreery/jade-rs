#[deriving(PartialEq)]
pub enum BlockType {
    Append,
    Prepend,
    Replace
}

#[deriving(PartialEq)]
pub enum TokenType {
    Text,
    Eos,
    Outdent,
    PipelessText,
    Yield,
    Doctype,
    Interpolation,
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
#[deriving(PartialEq)]
pub struct Token {
    token_type: TokenType,
    line_number: uint,
    value: String
}

