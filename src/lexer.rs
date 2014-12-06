enum BlockType {
    Append,
    Prepend,
    Replace
}

enum IncludeType {

}

enum TokenType {
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
    Block{ blockType: BlockType },
    MixinBlock,
    Include(String, ),
    Attrs(Vec<String>


}

struct Token {
    tokType: TokenType,
    lineNo: uint,
    val: ()
}

pub struct Lexer {
    input: String,
    filename: Option<String>,
    deferredTokens: Vec<Token>,
    lastIndents: uint,
    lineNo: uint,
    stash: Vec<Token>,
    indentStack: (),
    indentRe: (),
    pipeless: bool
}

impl Lexer {
    pub fn new(input: String, filename: Option<String>) -> Lexer {
        Lexer {
            input: input,
            filename: filename,
            deferredTokens: Vec::new(),
            lastIndents: 0,
            lineNo: 1,
            stash: Vec::new(),
            indentStack: (),
            indentRe: (),
            pipeless: false
        }
    }

    pub fn tok(&self, tokType: Token, val: String) {
        (atype, self.lineNo, val)
    }

    pub fn consume(len: uint) {
        input
    }
}
