use std::collections::VecDeque;
use std::iter::{Chain, IntoIterator, Once};
use std::str::FromStr;

use anyhow::Result;
use regex::Regex;
use serde::{
    de::{self, Error as DeError},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value;

#[allow(unused_macros)]
macro_rules! from_json {
    {
	$($inner:tt)*
    } => {
	{
	    let as_value = serde_json::json!( $($inner)* );
	    serde_json::from_value(as_value).unwrap()
	}
    }
}

use crate::error::Error;

pub mod inference;
pub use inference::{MergeStrategy, OptionalMergeStrategy, ValueMergeStrategy};

pub mod optionalise;
pub mod s_override;

pub mod namespace;
pub use namespace::Namespace;

pub mod content;
pub use content::*;

lazy_static! {
    pub static ref SLAT_REGEX: Regex = Regex::new("(?:^|\\.)(\"([^\"]+)\"|[^\"\\.]+)").unwrap();
}

pub trait ValueKindExt {
    fn kind(&self) -> &'static str;
}

impl ValueKindExt for Value {
    fn kind(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::String(_) => "string",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
            Self::Number(_) => "number",
            Value::Null => "null",
        }
    }
}

const NAME_RE: &str = "[A-Za-z_0-9]+";

#[allow(dead_code)]
pub fn bool_from_str<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<bool, D::Error> {
    let as_str = String::deserialize(d)?;
    as_str
        .parse()
        .map_err(|e| D::Error::custom(format!("not a boolean: {}", e)))
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Ord, PartialOrd)]
pub struct Name(String);

impl Name {
    #[allow(dead_code)]
    pub fn distance(&self, other: &Name) -> usize {
        strsim::levenshtein(&self.0, &other.0)
    }
}

impl FromStr for Name {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(NAME_RE).unwrap();
        let s = s.to_string();
        if re.is_match(&s) {
            Ok(Self(s))
        } else {
            Err(Self::Err::bad_request(format!("illegal name: {}", s)))
        }
    }
}

impl std::convert::AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let as_str = String::deserialize(deserializer)?;
        Self::from_str(&as_str).map_err(|err| {
            let msg = format!("invalid name: {}", err);
            D::Error::custom(msg)
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct FieldRef {
    collection: Name,
    fields: Vec<String>,
}

impl From<Name> for FieldRef {
    fn from(collection: Name) -> Self {
        Self {
            collection,
            fields: Vec::new(),
        }
    }
}

impl FieldRef {
    #[allow(dead_code)]
    pub fn new<R: AsRef<str>>(s: R) -> Result<Self> {
        Self::from_str(s.as_ref())
    }

    pub fn collection(&self) -> &Name {
        &self.collection
    }

    pub fn iter_fields(&self) -> impl Iterator<Item = &str> {
        self.fields.iter().map(|value| value.as_str())
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.collection.as_ref()).chain(self.iter_fields())
    }

    pub(crate) fn parent(&self) -> Option<FieldRef> {
        if self.is_top_level() {
            return None;
        }
        let parent = FieldRef {
            fields: self.fields.as_slice()[0..self.fields.len() - 1]
                .to_vec()
                .clone(),
            collection: self.collection.clone(),
        };
        Some(parent)
    }

    pub(crate) fn last(&self) -> String {
        match self.fields.last() {
            Some(field) => field.clone(),
            None => self.collection.clone().to_string(),
        }
    }

    pub(crate) fn is_top_level(&self) -> bool {
        self.fields.is_empty()
    }
}

#[derive(PartialEq)]
enum ParseState {
    Collection,
    Chunk,
    EOF,
}

struct Lexer {
    tokens: VecDeque<Token>,
}

impl Lexer {
    fn lex(s: String) -> Self {
        let tokens: VecDeque<Token> = s
            .to_string()
            .split("")
            .map(|char| match char {
                "." => Token::Dot,
                "\"" => Token::Quote,
                char => Token::Char(char.to_string()),
            })
            .collect();

        Self { tokens }
    }

    fn peak(&self) -> &Token {
        self.tokens.front().unwrap_or(&Token::EOF)
    }

    fn eat_next(&mut self) -> Token {
        self.tokens.pop_front().unwrap_or(Token::EOF)
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
        while self.state != ParseState::EOF {
            let token = self.lex.eat_next();
            match self.state {
                ParseState::Collection => self.parse_coll(token)?,
                ParseState::Chunk => self.parse_chunk(token)?,
                ParseState::EOF => {
                    unreachable!();
                }
            };
        }

        Ok(FieldRef {
            collection: Name::from_str(&self.collection)?,
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
            Token::EOF => {
                if self.collection.is_empty() {
                    return Err(failed!(
                        target: Release,
                        BadRequest =>
                        "cannot have an empty collection name"
                    ));
                }
                self.state = ParseState::EOF;
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
                if self.lex.peak() == &Token::EOF {
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
            Token::EOF => {
                if self.curr_chunk.is_empty() {
                    return Err(
                        failed!(target: Release, BadRequest => "cannot have an empty chunk in field ref"),
                    );
                }
                self.field_chunks.push(self.curr_chunk.clone());
                self.state = ParseState::EOF;
            }
        }
        Ok(())
    }

    fn parse_quote_content(&mut self) -> Result<()> {
        let mut done = false;
        while !done {
            match self.lex.eat_next() {
                Token::Char(c) => self.curr_chunk.push_str(&c),
                Token::Dot => self.curr_chunk.push_str("."),
                Token::EOF => {
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
    EOF,
    Quote,
}

impl Serialize for FieldRef {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", &self))
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
        std::iter::once(self.collection.to_string()).chain(self.fields.into_iter())
    }
}

impl std::fmt::Display for FieldRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, chunk) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ".")?
            }

            if chunk.contains(".") {
                write!(f, "\"{}\"", chunk)?
            } else {
                write!(f, "{}", chunk)?
            }
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use super::content::tests::USER_SCHEMA;

    #[test]
    fn test_new() {
        let reference: FieldRef = "users.address.postcode".parse().unwrap();
        println!("{:?}", reference);
        assert_eq!("users".parse::<Name>().unwrap(), *reference.collection());
        let mut fields = reference.iter_fields();
        assert_eq!(&"address".to_string(), fields.next().unwrap());
        assert_eq!(&"postcode".to_string(), fields.next().unwrap());

        let reference: FieldRef = "users.\"address.postcode\"".parse().unwrap();
        assert_eq!("users".parse::<Name>().unwrap(), *reference.collection());
        let mut fields = reference.iter_fields();
        assert_eq!("address.postcode".to_string(), fields.next().unwrap());
    }

    #[test]
    fn test_format_validation() {
        assert!("users.".parse::<FieldRef>().is_err());
        assert!(".users.".parse::<FieldRef>().is_err());
        assert!("users.some_field".parse::<FieldRef>().is_ok());
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
        let reference = FieldRef::new("users.address.postcode".to_string()).unwrap();
        let parent = FieldRef::new("users.address".to_string()).unwrap();
        let grandparent = FieldRef::new("users".to_string()).unwrap();
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

    #[allow(unused_macros)]
    macro_rules! hashmap {
	($( $key: expr => $val: expr ),*) => {{
            let mut map = ::std::collections::BTreeMap::new();
            $( map.insert($key, $val); )*
		map
	}}
    }

    lazy_static! {
        pub static ref USER_NAMESPACE: Namespace = Namespace {
            collections: hashmap!("users".parse().unwrap() => USER_SCHEMA.clone())
        };
    }
}
