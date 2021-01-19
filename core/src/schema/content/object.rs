use super::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ObjectContent {
    #[serde(flatten)]
    pub fields: HashMap<String, FieldContent>,
}

impl ObjectContent {
    pub fn get(&self, field: &str) -> Result<&FieldContent> {
        let suggest = suggest_closest(self.fields.keys(), field).unwrap_or_default();
        self.fields.get(field).ok_or_else(|| {
            failed!(target: Release,
                NotFound => "no such field: '{}'{}",
                field,
                suggest
            )
        })
    }

    pub fn accepts(&self, obj: &JsonObject) -> Result<()> {
        // There is probably a more efficient way of doing this
        // But it's linear time

        // First check if JSON has all the required fields
        for (k, v) in self.iter() {
            if v.optional {
                if let Some(value) = obj.get(k) {
                    v.content.accepts(value)?;
                }
            } else {
                let json_value =
                    obj.get(k)
                        .ok_or(failed!(target: Release, "could not find field: '{}'", k))?;
                v.content
                    .accepts(json_value)
                    .context(anyhow!("in a field: '{}'", k))?;
            }
        }

        // Then check if fields contains all the json keys
        for (k, _) in obj {
            if !self.fields.contains_key(k) {
                return Err(failed!(
                    target: Release,
                    "field '{}' is not recognized in the schema",
                    k
                ));
            }
        }

        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &FieldContent)> {
        self.fields.iter()
    }

    pub fn get_mut(&mut self, field: &str) -> Result<&mut FieldContent> {
        let suggest = suggest_closest(self.fields.keys(), field).unwrap_or_default();
        self.fields.get_mut(field).ok_or_else(
            || failed!(target: Release, NotFound => "no such field: '{}'{}", field, suggest),
        )
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FieldContent {
    #[serde(default)]
    pub optional: bool,
    #[serde(flatten)]
    pub content: Box<Content>,
}

impl FieldContent {
    pub fn new<I: Into<Content>>(content: I) -> Self {
        FieldContent {
            optional: false,
            content: Box::new(content.into()),
        }
    }

    pub fn optional(&mut self, optional: bool) {
        self.optional = optional;
    }
}

impl Default for ObjectContent {
    fn default() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
}

impl Find<Content> for ObjectContent {
    fn project<I, R>(&self, mut reference: Peekable<I>) -> Result<&Content>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>,
    {
        let next_ = reference.next().ok_or(failed!(
            target: Release,
            "expected a field name, found nothing"
        ))?;
        let next = next_.as_ref();
        self.get(next)?
            .content
            .project(reference)
            .context(anyhow!("in a field: {}", next))
    }

    fn project_mut<I, R>(&mut self, mut reference: Peekable<I>) -> Result<&mut Content>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>,
    {
        let next_ = reference.next().ok_or(failed!(
            target: Release,
            "expected a field name, found nothing"
        ))?;
        let next = next_.as_ref();
        self.get_mut(next)?
            .content
            .project_mut(reference)
            .context(anyhow!("in a field named {}", next))
    }
}

impl Compile for ObjectContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Model> {
        let generator = self
            .iter()
            .map(|(name, field_content)| {
                let mut built = compiler.build(name, &field_content.content)?;
                if field_content.optional {
                    let src = vec![built, Model::null()];
                    let gen = src
                        .into_iter()
                        .map(|inner| Box::new(inner))
                        .collect::<OneOf<_>>();
                    built = Model::Optional(gen);
                }
                Ok(built.with_key(name.to_string().yield_token()))
            })
            .collect::<Result<Chain<_>>>()?
            .into_map(Some(self.len()));
        Ok(Model::Object(generator))
    }
}
