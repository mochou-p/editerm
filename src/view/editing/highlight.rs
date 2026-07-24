// mochou-p/editerm/src/view/editing/highlight.rs

use spliterm::betterm;
use betterm::styled_printer::StyledPrinter;
use super::Editing;
use crate::config::Theme;


impl Editing {
    pub fn highlight(&self, sp: StyledPrinter, visible_line: &str, theme: &Theme) -> StyledPrinter {
        if let Some(os_s) = self.path.extension() && let Some(s) = os_s.to_str() {
            match s {
                "rs" => { return self.highlight_rust(sp, visible_line, theme); },
                _    => ()
            }
        }

        sp.text(visible_line)
    }

    fn highlight_rust(&self, mut sp: StyledPrinter, visible_line: &str, theme: &Theme) -> StyledPrinter {
        let mut splitter = Splitter::new(visible_line);

        while let Some(word) = splitter.next() {
            sp = match word {
                Word::Text(s) => match s {
                    "as" | "async"  | "await"  | "break" | "const" | "continue" | "crate"  | "dyn"   | "else"  | "enum"
                         | "extern" | "fn"     | "for"   | "if"    | "impl"     | "in"     | "let"   | "loop"  | "match"
                         | "mod"    | "move"   | "mut"   | "pub"   | "ref"      | "return" | "self"  | "Self"  | "static"
                         | "struct" | "super"  | "trait" | "type"  | "unsafe"   | "use"    | "where" | "while" | "raw"
                         | "union"  | "safe"

                         | "new"   | "into"    | "from"        | "try_into"       | "try_from"       | "get" | "set"
                         | "lock"  | "unwrap"  | "expect"      | "unwrap_or"      | "unwrap_or_else" | "unwrap_or_default"
                         | "map"   | "map_or"  | "map_or_else" | "map_or_default" | "unwrap_err"     | "map_err"
                         | "clone" | "display" | "borrow"      | "borrow_mut"     | "as_mut"         | "as_ref"
                    => {
                        sp.fg(theme.blue, s)
                    },
                    "bool" | "char" | "f32" | "f64" | "i8"   | "i16"   | "i32" | "i64" | "i128" | "isize" | "str" | "u8"
                           | "u16"  | "u32" | "u64" | "u128" | "usize"

                           | "String" | "Vec"        | "HashMap" | "HashSet" | "Result"    | "Option"   | "T"        | "Pin"
                           | "Arc"    | "LazyLock"   | "Mutex"   | "Once"    | "OnceLock"  | "RwLock"   | "Weak"     | "Rc"
                           | "Cell"   | "RefCell"    | "Ref"     | "Borrow"  | "BorrowMut" | "LazyCell" | "OnceCell"
                           | "RefMut" | "UnsafeCell" | "Box"
                    => {
                        sp.fg(theme.cyan, s)
                    },
                    "Clone" | "Copy"  | "Debug" | "Default" | "Eq"   | "Hash"    | "Ord"     | "PartialEq" | "PartialOrd"
                            | "Send"  | "Sync"  | "Into"    | "From" | "TryInto" | "TryFrom" | "Write"     | "Read"
                            | "Unpin" | "UnsafeUnpin"
                    => {
                        sp.fg(theme.magenta, s)
                    },
                    "false" | "true" | "Ok"  | "Err" | "Some" | "None" => {
                        sp.fg(theme.yellow, s)
                    },
                    "_" | "?" | "!" | "+" | "-" | "/" | "*" | "&" | "=" | "<" | ">" | "%"

                        | "assert"  | "assert_eq"     | "assert_ne"     | "assert_matches" | "cfg"      | "concat"
                        | "dbg"     | "print"         | "eprint"        | "println"        | "eprintln" | "format"
                        | "include" | "include_str"   | "include_bytes" | "matches"        | "panic"    | "stringify"
                        | "todo"    | "unimplemented" | "vec"           | "write"          | "writeln"  | "derive"
                        | "default" | "env"           | "main"
                    => {
                        sp.fg(theme.red, s)
                    },
                    _ => sp.fg(theme.foreground, s)
                },
                Word::Number (s) => sp.fg(theme.yellow,              s),
                Word::Str    (s) => sp.fg(theme.green,               s),
                Word::Comment(s) => sp.fg(theme.foreground_disabled, s)
            };
        }

        sp
    }
}

enum Word<'a> {
    Text   (&'a str),
    Number (&'a str),
    Str    (&'a str),
    Comment(&'a str)
}

struct Splitter<'a> {
    view: &'a str,
    len:  usize,
    i:    usize
}

impl<'a> Splitter<'a> {
    fn new(view: &'a str) -> Self {
        Self { view, len: view.chars().count(), i: 0 }
    }

    fn next(&mut self) -> Option<Word<'a>> {
        if self.i == self.len {
            return None;
        }

        let     start = self.i;
        let mut chars = self.view.chars();
        let     ch    = chars.nth(start).unwrap();

        match ch {
            'a'..='z' | 'A'..='Z' | '_' => {
                loop {
                    self.i += 1;

                    match chars.next() {
                        Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9') => (),
                        _                                             => { break; }
                    }
                }

                Some(Word::Text(&self.view[start..self.i]))
            }
            '0'..='9' => {
                loop {
                    self.i += 1;

                    match chars.next() {
                        Some('0'..='9') => (),
                        _               => { break; }
                    }
                }

                Some(Word::Number(&self.view[start..self.i]))
            },
            '"' | '\'' => {
                loop {
                    self.i += 1;

                    match chars.next() {
                        Some(c) => {
                            if c == ch {
                                self.i += 1;
                                break;
                            }
                        },
                        None => { return Some(Word::Text(&self.view[start..])); }
                    }
                }

                Some(Word::Str(&self.view[start..self.i]))
            },
            '/' => {
                self.i += 1;
                if let Some(ch) = chars.next() && ch == '/' {
                    self.i = self.len;
                    return Some(Word::Comment(&self.view[start..]));
                }

                Some(Word::Text("/"))
            },
            ' ' | '\t' => {
                loop {
                    self.i += 1;

                    match chars.next() {
                        Some(' ' | '\t') => (),
                        _                => { break; }
                    }
                }

                Some(Word::Text(&self.view[start..self.i]))
            },
            ';' | ':' | '{' | '}' | '(' | ')' | '[' | ']' | '.' | '+' | '-' | '*'
                | ',' | '<' | '>' | '=' | '!' | '?' | '&' | '|' | '~' | '#' | '%'
            => {
                self.i += 1;
                Some(Word::Text(&self.view[start..start+1]))
            }
            other => panic!("unexpected character: {other:?}")
        }
    }
}
