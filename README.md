jade-rs
=======

Nowhere near useable jade lib in rust.

Eventually it will take in jade as a string, and output either

  - a rust function, taking a hashmap (or something else) and outputting
    html
  - a string of javascript able to render the jade template as html
  - or a string of pure html

At the moment it does none of these things however.

## Jade EBNF
```
letter = 'a'|'b'|'c' | ... | 'z' | 'A' | ... | 'Z'
digit = '0' | '1' | '2' | '3' | '4' | ... | '9'
symbol = '!' | '"' | '$' | ...
singlequote = '''
quote = ''' | '"'
alphanum = letter | digit
character = letter | digit | symbol
identifier = letter (letter | digit)*

jade_template = doctype_statement?

doctype_statement = 'doctype ' doctype_option '\n'
doctype_option = 'html' | 'xml' | 'transitional' | 'strict' | 'frameset' |
                 '1.1' | 'basic' | 'mobile' | doctype_custom
doctype_custom = character+
```
