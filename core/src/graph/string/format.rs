use crate::graph::prelude::*;
use anyhow::Result;

use dynfmt::Format as DynFormat;

type Formatted = TryOnce<Unwrap<Yield<Result<String, Error>>>>;

type FormatFn = Box<dyn Fn(FormatArgs<String>) -> Formatted>;

derive_generator! {
    yield String,
    return Result<String, Error>,
    pub struct Format(AndThenTry<FormatArgs<Graph>, FormatFn, Formatted>);
}

impl Format {
    pub fn new(fmt: &str, args: FormatArgs<Graph>) -> Self {
        let fmt = fmt.to_string();
        let format = move |args: FormatArgs<String>| {
            let formatted = dynfmt::SimpleCurlyFormat
                .format(fmt.as_str(), args)
                .map(|res| res.into_owned())
                .map_err(|err| failed_crate!(target: Release, "formatting error: {:?}", err));
            Yield::wrap(formatted).unwrap().try_once()
        };
        Self(args.and_then_try(Box::new(format)))
    }
}

pub struct FormatArgs<G> {
    pub unnamed: Vec<G>,
    pub named: HashMap<String, G>,
}

impl<G> Default for FormatArgs<G> {
    fn default() -> Self {
        Self {
            unnamed: Vec::new(),
            named: HashMap::new(),
        }
    }
}

impl<G> Generator for FormatArgs<G>
where
    G: Generator<Return = Result<Value, Error>>,
{
    type Yield = String;

    type Return = Result<FormatArgs<String>, Error>;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        GeneratorState::Complete(
            try {
                FormatArgs {
                    unnamed: self
                        .unnamed
                        .iter_mut()
                        .map(|unnamed| unnamed.complete(rng).and_then(|value| value.try_into()))
                        .collect::<Result<_, Error>>()?,
                    named: self
                        .named
                        .iter_mut()
                        .map(|(key, named)| {
                            Ok((
                                key.clone(),
                                named.complete(rng).and_then(|value| value.try_into())?,
                            ))
                        })
                        .collect::<Result<_, Error>>()?,
                }
            },
        )
    }
}

impl dynfmt::FormatArgs for FormatArgs<String> {
    fn get_index(&self, index: usize) -> std::result::Result<Option<dynfmt::Argument<'_>>, ()> {
        Ok(self.unnamed.get(index).map(|arg| arg as dynfmt::Argument))
    }

    fn get_key(&self, key: &str) -> std::result::Result<Option<dynfmt::Argument<'_>>, ()> {
        Ok(self.named.get(key).map(|arg| arg as dynfmt::Argument))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::graph::{
        DateTimeNode, NumberNode, RandFaker, RandomDateTime, RandomI64,
        RandomString, StringNode,
    };
    use chrono::naive::NaiveDate;

    fn faker_graph(name: &str) -> Graph {
        Graph::String(StringNode::from(RandomString::from(
            RandFaker::new(name, Default::default()).unwrap(),
        )))
    }

    #[test]
    fn format_with_named_args() {
        let mut rng = rand::thread_rng();

        let args = FormatArgs {
            named: HashMap::from([
                ("name".to_string(), faker_graph("username")),
                ("email".to_string(), faker_graph("safe_email")),
            ]),
            ..Default::default()
        };
        let formatted = Format::new("my email is {email} and my username is {name}", args);
        formatted
            .repeat(1024)
            .complete(&mut rng)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
    }

    #[test]
    fn format_with_unnamed_args() {
        let mut rng = rand::thread_rng();

        let args = FormatArgs {
            unnamed: vec![faker_graph("username"), faker_graph("safe_email")],
            ..Default::default()
        };
        let formatted = Format::new("my email is {} and my username is {}", args);
        formatted
            .repeat(1024)
            .complete(&mut rng)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
    }

    #[test]
    fn format_with_number_args() {
        let mut rng = rand::thread_rng();

        let args = FormatArgs {
            named: HashMap::from([(
                "id".to_string(),
                Graph::Number(NumberNode::from(RandomI64::constant(42))),
            )]),
            ..Default::default()
        };
        let formatted = Format::new("{id}_suffix", args);
        let gen = formatted
            .repeat(1024)
            .complete(&mut rng)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(gen[0], "42_suffix");
    }

    #[test]
    fn format_with_date_args() {
        let mut rng = rand::thread_rng();

        let args = FormatArgs {
            named: HashMap::from([(
                "date".to_string(),
                Graph::DateTime(DateTimeNode::from(RandomDateTime::new(
                    ChronoValue::NaiveDate(NaiveDate::from_ymd_opt(2021, 10, 4).unwrap())
                        ..ChronoValue::NaiveDate(NaiveDate::from_ymd_opt(2021, 10, 4).unwrap()),
                    "%Y-%m-%d",
                ))),
            )]),
            ..Default::default()
        };
        let formatted = Format::new("{date}.png", args);
        let gen = formatted
            .repeat(1024)
            .complete(&mut rng)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(gen[0], "2021-10-04.png");
    }
}
