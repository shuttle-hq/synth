use super::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct ArrayContent {
    pub length: Box<Content>,
    pub content: Box<Content>,
}

impl Compile for ArrayContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Graph> {
	let length = compiler.build("length", self.length.as_ref())?;
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
                otherwise
                    .map(|s| format!("'{}'", s.as_ref()))
                    .unwrap_or("nothing".to_string())
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
                otherwise
                    .map(|s| format!("'{}'", s.as_ref()))
                    .unwrap_or("nothing".to_string())
            )),
        }
    }
}
