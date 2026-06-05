/// Represents a parsed node in the AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Thing {
    Null,
    Bool(bool),
    /// Non-string scalar symbol, may be:
    /// - variable
    /// - param
    /// - function name
    Atom(String),
    /// String symbol.
    String(Vec<StringPart>),
    /// Number, we represent it as a string to avoid float shenanigans.
    Number(String),
    /// List of `()`.
    List(Vec<Thing>),
    /// List of `[]`.
    BracketedList(Vec<Thing>),
    /// Attrset within `{}`.
    /// FIX
    Attrset(Vec<(Thing, Thing)>),

    /// Of form:
    /// ```winter
    /// (. x y)
    /// ```
    /// Which will compile to:
    /// ```nix
    /// x.${y}
    /// ```
    Selection(Box<(Thing, Thing)>),

    /// Function like:
    /// (fn <name?> [x y z ...]
    ///    body)
    Function {
        name: Option<String>,
        return_type: Option<ThingType>,
        params: Vec<FunctionParam>,
    },

    Let {
        entries: Vec<VariableBinding>,
        body: Box<Thing>,
    },
    /// winter-only abstraction that auto-resolves to a `let` binding through scope.
    Locals {
        entries: Vec<VariableBinding>,
    },
    /// inherit shorthand, may be used as a VariableBinding as well
    Inherit(InheritBinding),

    /// winter-only type definition of the following form:
    /// ```winter
    /// (type Person
    ///     name@str
    ///     age@num)
    /// ```
    TypeDeclaration {
        name: String,
        fields: Vec<(Thing, Option<ThingType>)>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringPart {
    Literal(String),
    Expression(Thing),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThingType {
    // built-ins
    Any,
    Null,
    Bool,
    String,
    Num,
    Optional(Box<ThingType>),
    List(Box<ThingType>),
    Attrset(Box<ThingType>, Box<ThingType>),
    Union(Box<ThingType>, Box<ThingType>),
    // customs
    /// refers to 'some' type of a given identifiable name
    Some(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableBinding {
    Variable {
        identifier: String,
        assignment: Thing,
        var_type: Option<ThingType>,
    },
    Inherit(InheritBinding),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InheritBinding {
    items: Vec<InheritBindingItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InheritBindingItem {
    source: String,
    inherited: Vec<Thing>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionParam {
    Param {
        name: Option<String>,
        param_type: Option<ThingType>,
        /// In case param is 'unpacked' like in:
        /// ```winter
        /// (fn [a b {x y z}])
        /// ```
        /// If param is 'unpacked' and also named:
        /// ```winter
        /// (fn [a b c.{x y z}])
        /// ```
        attr_keys: Option<Vec<FunctionParam>>,
    },
    /// Of form `...`
    Ellipsis,
}

pub struct Parser<'a> {
    source: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }

    pub fn parse(mut self) -> Result<Thing, String> {
        todo!()
    }

    fn parse_thing(&mut self) -> Result<Thing, String> {
        self.skip_ignored();

        match self.peek() {
            Some('(') => self.parse_list(),
            Some('"') => self.parse_string(),
            Some(')') => Err(self.error("unexpected ')'")),
            Some(_) => self.parse_atom(),
            None => Err(self.error("expected a thing")),
        }
    }

    fn parse_atom(&mut self) -> Result<Thing, String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || ch == '(' || ch == ')' || ch == ';' {
                break;
            }
            self.next();
        }

        if start == self.pos {
            return Err(self.error("expected atom"));
        }

        Ok(Thing::Atom(self.source[start..self.pos].to_string()))
    }

    fn parse_string(&mut self) -> Result<Thing, String> {
        self.next(); // opening quote
        let mut value = String::new();

        loop {
            match self.next() {
                Some('"') => return Ok(Thing::String(value)),
                Some('\\') => match self.next() {
                    Some('n') => value.push('\n'),
                    Some('r') => value.push('\r'),
                    Some('t') => value.push('\t'),
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some(other) => {
                        return Err(self.error(&format!("unsupported escape sequence \\{other}")))
                    }
                    None => return Err(self.error("unterminated escape sequence")),
                },
                Some(ch) => value.push(ch),
                None => return Err(self.error("unterminated string")),
            }
        }
    }

    fn parse_list(&mut self) -> Result<Thing, String> {
        self.next(); // '('
        let mut things = Vec::new();

        loop {
            self.skip_ignored();

            match self.peek() {
                Some(')') => {
                    self.next();
                    return Ok(Thing::List(things));
                }
                Some(_) => things.push(self.parse_thing()?),
                None => return Err(self.error("unterminated list")),
            }
        }
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn error(&self, msg: &str) -> String {
        format!("{msg} at byte {}", self.pos)
    }

    // skips whitespace and comments
    fn skip_ignored(&mut self) {
        loop {
            while self.peek().is_some_and(char::is_whitespace) {
                self.next();
            }

            if self.peek() != Some(';') {
                return;
            }

            while let Some(ch) = self.next() {
                if ch == '\n' {
                    break;
                }
            }
        }
    }
}
