/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::error::{ClientResult, ClientError};

#[derive(Debug)]
pub(crate) enum CellValueReader {
    IntWithSize(usize),
    UIntWithSize(usize),
    Grams,
    Dict(Vec<CellFieldReader>),
    // Cell(Vec<CellFieldReader>),
}


#[derive(Debug)]
pub(crate) struct CellFieldReader {
    pub value: CellValueReader,
    pub skip: bool,
    pub name: String,
}

#[derive(Debug)]
pub(crate) struct CellQuery {
    pub(crate) commands: Vec<CellFieldReader>
}

enum Token {
    Minus,
    Colon,
    Open,
    Close,
    Identifier(String),
}

impl Token {
    fn is_minus(&self) -> Option<()> {
        if let Self::Minus = self { Some(()) } else { None }
    }

    fn is_colon(&self) -> Option<()> {
        if let Self::Colon = self { Some(()) } else { None }
    }

    fn is_open(&self) -> Option<()> {
        if let Self::Open = self { Some(()) } else { None }
    }

    fn is_close(&self) -> Option<()> {
        if let Self::Close = self { Some(()) } else { None }
    }

    fn identifier(&self) -> Option<String> {
        if let Self::Identifier(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }
}

struct Parser {
    _source: String,
    tokens: Vec<Token>,
    pos: usize,
}


impl Parser {
    fn is_en_letter(c: char) -> bool {
        (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z')
    }

    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_first_ident_char(c: char) -> bool {
        Self::is_en_letter(c) || c == '_'
    }

    fn is_ident_char(c: char) -> bool {
        Self::is_first_ident_char(c) || Self::is_digit(c)
    }

    fn tokenize_error(rest: &str) -> ClientError {
        ClientError::cell_invalid_query(format!("invalid character (-> {})", rest))
    }

    fn parse_error(&self, msg: &str) -> ClientError {
        // TODO: error message must point to error position related to self.tokens[self.pos]
        ClientError::cell_invalid_query(format!("{}", msg))
    }

    fn tokenize(source: String) -> ClientResult<Self> {
        let mut tokens = Vec::new();
        let mut chars = source.chars();
        let mut next = chars.next();
        while let Some(current) = next {
            next = chars.next();
            if let Some(token) = match current {
                space if space <= ' ' => None,
                ':' => Some(Token::Colon),
                '(' => Some(Token::Open),
                ')' => Some(Token::Close),
                first_ident if Self::is_first_ident_char(first_ident) => {
                    let mut identifier = String::new();
                    identifier.push(first_ident);
                    while Self::is_ident_char(next.unwrap_or(' ')) {
                        identifier.push(next.unwrap());
                        next = chars.next();
                    }
                    Some(Token::Identifier(identifier))
                }
                _ => return Err(Self::tokenize_error(chars.as_str()))
            } {
                tokens.push(token);
            }
        }
        Ok(Self {
            _source: source,
            tokens,
            pos: 0,
        })
    }

    fn eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn pass<F, R>(&mut self, expected: F) -> Option<R>
        where F: Fn(&Token) -> Option<R>
    {
        let value = if let Some(token) = self.tokens.get(self.pos) {
            expected(token)
        } else {
            None
        };
        if value.is_some() {
            self.pos += 1;
        }
        value
    }

    fn parse_commands(&mut self) -> ClientResult<Vec<CellFieldReader>> {
        let mut commands = Vec::new();
        while let Some(command) = self.parse_command()? {
            commands.push(command);
        }
        Ok(commands)
    }

    fn parse_optional_parenthesis_enclosed_commands(&mut self) -> ClientResult<Vec<CellFieldReader>> {
        if self.pass(Token::is_open) != None {
            let commands = self.parse_commands()?;
            if self.pass(Token::is_close) != None {
                Ok(commands)
            } else {
                Err(self.parse_error(") expected"))
            }
        } else {
            Ok(Vec::new())
        }
    }

    fn parse_command(&mut self) -> ClientResult<Option<CellFieldReader>> {
        let skip = self.pass(Token::is_minus) != None;
        if let Some(identifier) = self.pass(Token::identifier) {
            let name_type = if self.pass(Token::is_colon) != None {
                if let Some(type_name) = self.pass(Token::identifier) {
                    (identifier, type_name)
                } else {
                    return Err(self.parse_error("type expected"));
                }
            } else {
                (String::new(), identifier)
            };
            Ok(Some(CellFieldReader {
                name: name_type.0.to_string(),
                value: self.parse_value_reader(&name_type.1)?,
                skip,
            }))
        } else if skip {
            Err(self.parse_error("identifier expected"))
        } else {
            Ok(None)
        }
    }

    fn parse_value_reader(&mut self, type_name: &str) -> ClientResult<CellValueReader> {
        Ok(match type_name {
            "u1" => CellValueReader::UIntWithSize(1),
            "u8" => CellValueReader::UIntWithSize(8),
            "u16" => CellValueReader::UIntWithSize(16),
            "u32" => CellValueReader::UIntWithSize(32),
            "u64" => CellValueReader::UIntWithSize(64),
            "u128" => CellValueReader::UIntWithSize(128),
            "u256" => CellValueReader::UIntWithSize(256),
            "i1" => CellValueReader::IntWithSize(1),
            "i8" => CellValueReader::IntWithSize(8),
            "i16" => CellValueReader::IntWithSize(16),
            "i32" => CellValueReader::IntWithSize(32),
            "i64" => CellValueReader::IntWithSize(64),
            "i128" => CellValueReader::IntWithSize(128),
            "i256" => CellValueReader::IntWithSize(256),
            "grams" => CellValueReader::Grams,
            "dict" => CellValueReader::Dict(self.parse_optional_parenthesis_enclosed_commands()?),
            _ => return Err(self.parse_error(&format!("unknown type [{}]", type_name)))
        })
    }
}

impl CellQuery {
    pub(crate) fn parse(source: String) -> ClientResult<Self> {
        let mut parser = Parser::tokenize(source)?;
        let commands = parser.parse_commands()?;
        if parser.eof() {
            Ok(Self {
                commands
            })
        } else {
            Err(parser.parse_error("unexpected"))
        }
    }
}
