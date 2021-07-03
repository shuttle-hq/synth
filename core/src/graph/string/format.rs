use anyhow::Result;
use crate::graph::prelude::*;
use crate::graph::StringNode;
use std::collections::VecDeque;
use crate::graph::string::format::Token::CloseCurly;


pub struct Format {
    args: Vec<Graph>,
    formatter: NaiveFormatter,
}

impl Generator for Format {
    type Yield = String;
    type Return = Result<Never, Error>;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        let outputs = self.args
            .iter_mut()
            .map(|arg| arg.complete(rng))
            .collect::<Result<Vec<Value>, Error>>();

        match outputs {
            Ok(outputs) => {
                let strings = outputs.into_iter().map(|value| match value {
                    Value::Null(_) => Ok("null".to_string()),
                    Value::Bool(b) => Ok(b.to_string()),
                    Value::Number(n) => Ok(n.to_string()),
                    Value::String(s) => Ok(s),
                    Value::DateTime(_) => unimplemented!(),
                    Value::Object(_) => unimplemented!(),
                    Value::Array(_) => unimplemented!(),
                }).collect::<Result<Vec<String>, Error>>();
                match strings {
                    Ok(strings) => GeneratorState::Yielded(self.formatter.format(strings)),
                    Err(e) => GeneratorState::Complete(Err(e))
                }
            }
            Err(e) => GeneratorState::Complete(Err(e))
        }
    }
}

impl Format {
    pub fn new(format: String, args: Vec<Graph>) -> Result<Self> {
        // Validate that the number of args are equal to
        // the number fo generators in the format string.
        let formatter = NaiveFormatter::parse(format);
        if formatter.number_of_args() != args.len() {
            return Err(anyhow!("Malformed formatter. Format string has {} arguments but {} argument generators were supplied.", formatter.number_of_args(), args.len()));
        }

        Ok(Self { args, formatter })
    }
}

#[derive(PartialEq)]
enum ChunkOrArg {
    Chunk(String),
    Arg,
}

struct NonEmptyChunker(Vec<ChunkOrArg>);

impl NonEmptyChunker {
    fn push_chunk(&mut self, chunk: String) {
        if !chunk.is_empty() {
            self.0.push(ChunkOrArg::Chunk(chunk))
        }
    }

    fn push_special(&mut self) {
        self.0.push(ChunkOrArg::Arg)
    }
}

struct NaiveFormatter(Vec<ChunkOrArg>);

impl NaiveFormatter {
    fn number_of_args(&self) -> usize {
        self.0.iter().filter(|c| c == &&ChunkOrArg::Arg).count()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn parse(format: String) -> Self {
        let mut chunks = NonEmptyChunker(vec![]);
        let mut chunk = "".to_string();
        let mut lex = Lexer::lex(format);

        loop {
            match lex.eat_next() {
                Token::OpenCurly => {
                    if lex.peak() == &CloseCurly {
                        chunks.push_chunk(chunk.clone());
                        chunks.push_special();
                        chunk = "".to_string();
                        lex.eat_next();
                    } else {
                        chunk.push_str("{");
                    }
                }
                Token::CloseCurly => {
                    chunk.push_str("}");
                }
                Token::Char(c) => {
                    chunk.push_str(&c);
                }
                Token::EOF => {
                    chunks.push_chunk(chunk);
                    break;
                }
            }
        }
        Self(chunks.0)
    }

    fn format(&self, mut args: Vec<String>) -> String {
        let mut s = "".to_string();
        for cor in &self.0 {
            match cor {
                ChunkOrArg::Chunk(c) => s.push_str(c),
                ChunkOrArg::Arg => s.push_str(&args.remove(0))
            }
        }
        s
    }
}

#[derive(PartialEq)]
enum Token {
    Char(String),
    OpenCurly,
    CloseCurly,
    EOF,
}

struct Lexer {
    tokens: VecDeque<Token>,
}

impl Lexer {
    fn lex(s: String) -> Self {
        let tokens: VecDeque<Token> = s
            .split("")
            .map(|char| match char {
                "{" => Token::OpenCurly,
                "}" => Token::CloseCurly,
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


#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_naive_format() {
        let expected = "Hello my name is Joe and I am 100";
        let format = "{} my name is {} and I am {}";
        let formatter = NaiveFormatter::parse(format.to_string());
        let args = vec![String::from("Hello"), String::from("Joe"), String::from("100")];
        assert_eq!(expected, formatter.format(args).as_str())
    }


    #[test]
    fn test_naive_format_braces() {
        let expected = "Hello } name is { and I am { } 100";
        let format = "Hello } name is { and I am { } {}";
        let formatter = NaiveFormatter::parse(format.to_string());
        let args = vec![String::from("100")];
        assert_eq!(expected, formatter.format(args).as_str())
    }
}

