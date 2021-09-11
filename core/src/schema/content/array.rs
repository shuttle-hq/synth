use super::prelude::*;
use crate::graph::prelude::content::number::number_content::U64;
use crate::schema::{NumberContent, RangeStep};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct ArrayContent {
    pub length: Box<Content>,
    pub content: Box<Content>,
}

impl ArrayContent {
    pub fn from_content_default_length(content: Content) -> Self {
        Self {
            length: Box::new(Content::Number(NumberContent::U64(U64::Range(RangeStep {
                low: 1,
                high: 2,
                step: 1,
            })))),
            content: Box::new(content),
        }
    }
}

impl Compile for ArrayContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Graph> {
        let length = compiler.build("length", self.length.as_ref())?
            .into_size();
        let content = compiler.build("content", &self.content)?;
        Ok(Graph::Array(ArrayNode::new_with(length, content)))
    }
}

impl Find<Content> for ArrayContent {
    fn project<I, R>(&self, mut reference: Peekable<I>) -> Result<&Content>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>,
    {
        match reference.next() {
            Some(next) if next.as_ref() == "content" => self.content.project(reference),
            Some(next) if next.as_ref() == "length" => self.length.project(reference),
            otherwise => Err(failed!(
                target: Release,
                "expected 'content', found {}",
                otherwise.map_or_else(|| "nothing".to_string(), |s| format!("'{}'", s.as_ref()))
            )),
        }
    }

    fn project_mut<I, R>(&mut self, mut reference: Peekable<I>) -> Result<&mut Content>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>,
    {
        match reference.next() {
            Some(next) if next.as_ref() == "content" => self.content.project_mut(reference),
            Some(next) if next.as_ref() == "length" => self.length.project_mut(reference),
            otherwise => Err(failed!(
                target: Release,
                "expected 'content', found {}",
                otherwise.map_or_else(|| "nothing".to_string(), |s| format!("'{}'", s.as_ref()))
            )),
        }
    }
}
