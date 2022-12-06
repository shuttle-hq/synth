#![allow(clippy::assertions_on_result_states)]
use std::collections::VecDeque;
use std::iter::{Chain, IntoIterator, Once};
use std::str::FromStr;

use anyhow::Result;
use regex::Regex;
use serde::{
    de::{self, Error as DeError},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value as JsonValue;

pub mod inference;
pub use inference::{MergeStrategy, OptionalMergeStrategy, ValueMergeStrategy};

pub mod optionalise;

pub mod namespace;
pub use namespace::Namespace;

pub mod content;
pub use content::*;

pub mod scenario;
pub use scenario::Scenario;

lazy_static! {
    pub static ref SLAT_REGEX: Regex = Regex::new("(?:^|\\.)(\"([^\"]+)\"|[^\"\\.]+)").unwrap();
}

pub trait ValueKindExt {
    fn kind(&self) -> &'static str;
}

impl ValueKindExt for JsonValue {
    fn kind(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::String(_) => "string",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
            Self::Number(_) => "number",
            Self::Null => "null",
        }
    }
}

#[allow(dead_code)]
pub fn bool_from_str<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<bool, D::Error> {
    let as_str = String::deserialize(d)?;
    as_str
        .parse()
        .map_err(|e| D::Error::custom(format!("not a boolean: {e}")))
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct FieldRef {
    collection: String,
    fields: Vec<String>,
}

impl Serialize for FieldRef {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for FieldRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl FromStr for FieldRef {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lexer = Lexer::lex(s.to_string());

        let parser = Parser::new(lexer);

        parser.parse()
    }
}

impl IntoIterator for FieldRef {
    type IntoIter = Chain<Once<String>, <Vec<String> as IntoIterator>::IntoIter>;
    type Item = String;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self.collection).chain(self.fields.into_iter())
    }
}

impl std::fmt::Display for FieldRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, chunk) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ".")?
            }

            if chunk.contains('.') {
                write!(f, "\"{chunk}\"")?
            } else {
                write!(f, "{chunk}")?
            }
        }
        Ok(())
    }
}

impl FieldRef {
    #[allow(dead_code)]
    pub fn new<R: AsRef<str>>(s: R) -> Result<Self> {
        Self::from_str(s.as_ref())
    }

    pub fn from_collection_name(collection: String) -> Result<Self> {
        check_collection_name_is_valid(&collection).map(|_| Self {
            collection,
            fields: Vec::new(),
        })
    }

    pub fn collection(&self) -> &str {
        &self.collection
    }

    pub fn iter_fields(&self) -> impl Iterator<Item = &str> {
        self.fields.iter().map(String::as_str)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.collection.as_ref()).chain(self.iter_fields())
    }

    pub(crate) fn parent(&self) -> Option<FieldRef> {
        if self.is_top_level() {
            return None;
        }
        let parent = FieldRef {
            fields: self.fields.as_slice()[0..self.fields.len() - 1].to_vec(),
            collection: self.collection.clone(),
        };
        Some(parent)
    }

    pub(crate) fn last(&self) -> String {
        match self.fields.last() {
            Some(field) => field.clone(),
            None => self.collection.clone(),
        }
    }

    pub(crate) fn is_top_level(&self) -> bool {
        self.fields.is_empty()
    }
}

lazy_static! {
    static ref COLLECTION_NAME_REGEX: Regex = Regex::new("^[A-Za-z_0-9]+$").unwrap();
}

fn check_collection_name_is_valid(name: &str) -> Result<()> {
    if COLLECTION_NAME_REGEX.is_match(name) {
        Ok(())
    } else {
        Err(anyhow!("illegal collection name: {}", name))
    }
}

#[derive(PartialEq)]
enum ParseState {
    Collection,
    Chunk,
    Eof,
}

struct Lexer {
    tokens: VecDeque<Token>,
}

impl Lexer {
    fn lex(s: String) -> Self {
        let tokens: VecDeque<Token> = s
            .split("")
            .map(|char| match char {
                "." => Token::Dot,
                "\"" => Token::Quote,
                char => Token::Char(char.to_string()),
            })
            .collect();

        Self { tokens }
    }

    fn peek(&self) -> &Token {
        self.tokens.front().unwrap_or(&Token::Eof)
    }

    fn eat_next(&mut self) -> Token {
        self.tokens.pop_front().unwrap_or(Token::Eof)
    }
}

struct Parser {
    lex: Lexer,
    state: ParseState,
    collection: String,
    field_chunks: Vec<String>,
    curr_chunk: String,
}

impl Parser {
    fn new(lex: Lexer) -> Self {
        Self {
            lex,
            state: ParseState::Collection,
            collection: "".to_string(),
            field_chunks: vec![],
            curr_chunk: "".to_string(),
        }
    }

    fn parse(mut self) -> Result<FieldRef> {
        while self.state != ParseState::Eof {
            let token = self.lex.eat_next();
            match self.state {
                ParseState::Collection => self.parse_coll(token)?,
                ParseState::Chunk => self.parse_chunk(token)?,
                ParseState::Eof => {
                    unreachable!();
                }
            };
        }

        check_collection_name_is_valid(&self.collection).map(|_| FieldRef {
            collection: self.collection,
            fields: self.field_chunks,
        })
    }

    fn parse_coll(&mut self, token: Token) -> Result<()> {
        match token {
            Token::Char(c) => {
                self.collection.push_str(&c);
            }
            Token::Dot => {
                if self.collection.is_empty() {
                    return Err(failed!(
                        target: Release,
                        BadRequest =>
                        "cannot have an empty collection name"
                    ));
                }
                self.state = ParseState::Chunk;
            }
            Token::Eof => {
                if self.collection.is_empty() {
                    return Err(failed!(
                        target: Release,
                        BadRequest =>
                        "cannot have an empty collection name"
                    ));
                }
                self.state = ParseState::Eof;
            }
            Token::Quote => {
                return Err(
                    failed!(target: Release, BadRequest => "cannot put quotes in collection names"),
                )
            }
        }
        Ok(())
    }

    fn parse_chunk(&mut self, token: Token) -> Result<()> {
        match token {
            Token::Char(c) => {
                self.curr_chunk.push_str(&c);
            }
            Token::Dot => {
                if self.lex.peek() == &Token::Eof {
                    return Err(
                        failed!(target: Release, BadRequest => "cannot have a field ref that ends in a '.'"),
                    );
                }
                if self.curr_chunk.is_empty() {
                    return Err(
                        failed!(target: Release, BadRequest => "cannot have an empty chunk in field ref"),
                    );
                }
                self.field_chunks.push(self.curr_chunk.clone());
                self.curr_chunk = "".to_string();
            }
            Token::Quote => {
                if !self.curr_chunk.is_empty() {
                    return Err(
                        failed!(target: Release, BadRequest => "cannot have partially quoted field references"),
                    );
                }
                self.parse_quote_content()?;
            }
            Token::Eof => {
                if self.curr_chunk.is_empty() {
                    return Err(
                        failed!(target: Release, BadRequest => "cannot have an empty chunk in field ref"),
                    );
                }
                self.field_chunks.push(self.curr_chunk.clone());
                self.state = ParseState::Eof;
            }
        }
        Ok(())
    }

    fn parse_quote_content(&mut self) -> Result<()> {
        let mut done = false;
        while !done {
            match self.lex.eat_next() {
                Token::Char(c) => self.curr_chunk.push_str(&c),
                Token::Dot => self.curr_chunk.push('.'),
                Token::Eof => {
                    return Err(
                        failed!(target: Release, BadRequest => "cannot have unclosed quotes"),
                    )
                }
                Token::Quote => {
                    if self.curr_chunk.is_empty() {
                        return Err(
                            failed!(target: Release, BadRequest => "cannot have an empty chunk in quotes"),
                        );
                    }
                    done = true;
                }
            }
        }
        Ok(())
    }
}

#[derive(PartialEq)]
enum Token {
    Char(String),
    Dot,
    Eof,
    Quote,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use super::content::tests::USER_SCHEMA;

    #[test]
    fn test_new() {
        let reference: FieldRef = "users.address.postcode".parse().unwrap();
        println!("{:?}", reference);
        assert_eq!("users".to_string(), *reference.collection());
        let mut fields = reference.iter_fields();
        assert_eq!("address", fields.next().unwrap());
        assert_eq!("postcode", fields.next().unwrap());

        let reference: FieldRef = "users.\"address.postcode\"".parse().unwrap();
        assert_eq!("users".to_string(), *reference.collection());
        let mut fields = reference.iter_fields();
        assert_eq!("address.postcode", fields.next().unwrap());
    }

    #[test]
    fn test_format_validation() {
        assert!("users.".parse::<FieldRef>().is_err());
        assert!(".users.".parse::<FieldRef>().is_err());
        assert!("users.some_field".parse::<FieldRef>().is_ok());
        assert!("us@%ers.some_field".parse::<FieldRef>().is_err());
    }

    #[test]
    fn test_display() {
        let reference: FieldRef = "users.address.postcode".parse().unwrap();
        assert_eq!("users.address.postcode", reference.to_string());

        let reference: FieldRef = "users.\"address.postcode\"".parse().unwrap();
        assert_eq!("users.\"address.postcode\"", reference.to_string());
    }

    #[test]
    fn test_empty_strings() {
        assert!(FieldRef::from_str("users.\"\".postcode").is_err())
    }

    #[test]
    fn test_serde() {
        let str = "\"users.address.postcode\"";
        let reference: FieldRef = serde_json::from_str(str).unwrap();
        assert_eq!(str, serde_json::to_string(&reference).unwrap())
    }

    #[test]
    fn test_parent() {
        let reference = FieldRef::new("users.address.postcode").unwrap();
        let parent = FieldRef::new("users.address").unwrap();
        let grandparent = FieldRef::new("users").unwrap();
        assert_eq!(parent, reference.parent().unwrap());
        assert_eq!(grandparent, reference.parent().unwrap().parent().unwrap());
        // Reference has no grandgrandparent
        assert!(reference
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .is_none());
    }

    #[test]
    fn do_not_allow_empty_collection_names() {
        assert!(FieldRef::from_str("").is_err());
        println!("{:?}", FieldRef::from_str(".some_field"));

        assert!(FieldRef::from_str(".some_field").is_err());
    }

    #[test]
    fn do_not_allow_empty_field_ref_chunks() {
        assert!(FieldRef::from_str("users..some_field").is_err());
        assert!(FieldRef::from_str("users.some_field.").is_err());
    }

    #[test]
    fn is_top_level() {
        assert!(FieldRef::from_str("users").unwrap().is_top_level());
        assert!(!FieldRef::from_str("users.address").unwrap().is_top_level());
    }

    #[test]
    fn last_includes_collections() {
        assert_eq!(
            FieldRef::from_str("users").unwrap().last(),
            String::from("users")
        )
    }

    lazy_static! {
        pub static ref USER_NAMESPACE: Namespace = {
            let mut n = Namespace::new();
            n.put_collection("users".to_string(), USER_SCHEMA.clone())
                .unwrap();
            n
        };
    }
}
