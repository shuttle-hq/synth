use super::prelude::*;

use super::Weight;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct OneOfContent {
    pub variants: Vec<VariantContent>,
}

impl PartialEq for OneOfContent {
    fn eq(&self, other: &Self) -> bool {
        for left in self.variants.iter() {
            if !other.variants.contains(left) {
                return false;
            }
        }
        for right in other.variants.iter() {
            if !self.variants.contains(right) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VariantContent {
    #[serde(default)]
    weight: Weight,
    #[serde(flatten)]
    pub content: Box<Content>,
}

impl PartialEq for VariantContent {
    fn eq(&self, other: &Self) -> bool {
        self.content.eq(&other.content)
    }
}

impl VariantContent {
    pub fn new(content: Content) -> Self {
        VariantContent {
            weight: Weight::default(),
            content: Box::new(content),
        }
    }
}

impl FromIterator<Content> for OneOfContent {
    fn from_iter<T: IntoIterator<Item = Content>>(iter: T) -> Self {
        let mut one_of = OneOfContent { variants: vec![] };

        iter.into_iter().for_each(|content| one_of.update(content));

        one_of
    }
}

impl<'t> FromIterator<&'t Value> for OneOfContent {
    fn from_iter<T: IntoIterator<Item = &'t Value>>(iter: T) -> Self {
        let mut out = Self {
            variants: Vec::new(),
        };
        let strategy = OptionalMergeStrategy;
        iter.into_iter()
            .for_each(|value| out.insert_with(strategy, value));
        out
    }
}

impl OneOfContent {
    fn update(&mut self, candidate: Content) {
        match self
            .variants
            .iter_mut()
            .find(|variant| *variant.content == candidate)
        {
            None => self.add_variant(candidate),
            Some(master) => master.weight.0 += 1.0,
        }
    }

    pub fn as_nullable(&self) -> Option<&Content> {
        if self.variants.len() == 2 {
            let mut non_null = self
                .variants
                .iter()
                .filter(|variant| !variant.content.is_null())
                .map(|vc| vc.content.as_ref());
            let content = non_null.next()?;
            if non_null.next().is_none() {
                return Some(content);
            }
        }
        None
    }

    pub fn is_nullable(&self) -> bool {
        self.as_nullable().is_some()
    }

    fn add_variant(&mut self, variant: Content) {
        self.variants.push(VariantContent::new(variant))
    }

    pub fn insert_with<M>(&mut self, strategy: M, what: &Value)
    where
        M: MergeStrategy<Self, Value> + MergeStrategy<Content, Value> + Copy,
    {
        let res: Vec<_> = self
            .iter_mut()
            .map(|variant| strategy.try_merge(variant, what))
            .collect();
        if !res.iter().any(|r| r.is_ok()) {
            self.variants.push(VariantContent::new(what.into()))
        }
    }

    pub fn accepts(&self, arr: &[Value]) -> Result<()> {
        // try each value exhaustively against the allowed types in
        // the schema until there is a match
        for json_value in arr {
            let is_acceptable = self
                .variants
                .iter()
                .any(|allowed_type| allowed_type.content.accepts(json_value).is_ok());
            if !is_acceptable {
                return Err(failed!(
                    target: Release,
                    "value '{}' not allowed in array",
                    json_value
                ));
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Content> {
        self.variants.iter().map(|variant| variant.content.as_ref())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Content> {
        self.variants
            .iter_mut()
            .map(|variant| variant.content.as_mut())
    }
}

impl Find<Content> for OneOfContent {
    fn project<I, R>(&self, mut reference: Peekable<I>) -> Result<&Content>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>,
    {
        let next_ = reference
            .next()
            .ok_or_else(|| Error::bad_request("expected a field name, found nothing"))?;

        let index: usize = next_.as_ref().parse().map_err(|_| {
            Error::bad_request(format!(
                "expected integer as array index, instead found '{}'",
                next_.as_ref()
            ))
        })?;

        match self.variants.get(index) {
            None => Err(Error::not_found(format!(
                "Could not find element at index: '{}'. Valid indices are between 0 and '{}'",
                index,
                self.variants.len() - 1,
            ))
            .into()),
            Some(next) => next.content.project(reference),
        }
    }

    fn project_mut<I, R>(&mut self, mut reference: Peekable<I>) -> Result<&mut Content>
    where
        I: Iterator<Item = R>,
        R: AsRef<str>,
    {
        let next_ = reference
            .next()
            .ok_or_else(|| Error::bad_request("expected a field name, found nothing"))?;

        let index: usize = next_.as_ref().parse().map_err(|_| {
            Error::bad_request(format!(
                "expected integer as array index, instead found '{}'",
                next_.as_ref()
            ))
        })?;

        let length = self.variants.len();
        match self.variants.get_mut(index) {
            None => Err(Error::not_found(format!(
                "Could not find element at index: '{}'. Valid indices are between 0 and '{}'",
                index,
                length - 1,
            ))
            .into()),
            Some(next) => next.content.project_mut(reference),
        }
    }
}

impl Compile for OneOfContent {
    fn compile<'a, C: Compiler<'a>>(&'a self, mut compiler: C) -> Result<Graph> {
        let one_of_node = self
            .variants
            .iter()
            .enumerate()
            .map(|(idx, variant)| {
                compiler
                    .build(&idx.to_string(), &variant.content)
                    .map(|graph| (variant.weight.0, graph))
            })
            .collect::<Result<OneOfNode>>()?;
        Ok(Graph::OneOf(one_of_node))
    }
}
