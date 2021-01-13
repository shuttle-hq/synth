use serde::{Deserialize, Serialize};

pub use synth::error::*;

macro_rules! failed {
    (target: $target:ident, $lit: literal$(, $arg:expr)*) => {
	failed!(target: $target, BadRequest => $lit$(, $arg)*)
    };
    (target: $target:ident, $variant:ident => $lit:literal$(, $arg:expr)*) => {
	anyhow::Error::from(
	    crate::error::Error::new_with_target(
		crate::error::ErrorKind::$variant,
		format!($lit$(, $arg)*),
		crate::error::Target::$target
	    )
	)
    };
}

impl From<Error> for UserError {
    fn from(crate_error: Error) -> Self {
        UserError {
            msg: vec![crate_error.msg.unwrap_or("".to_string())],
            kind: crate_error.kind,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserError {
    msg: Vec<String>,
    kind: ErrorKind,
}

impl UserError {
    fn extend(&mut self, msg: &str) {
        self.msg.push(msg.to_string())
    }
}

impl From<&(dyn std::error::Error + 'static)> for UserError {
    fn from(original: &(dyn std::error::Error + 'static)) -> Self {
        let mut final_error: Option<UserError> = None;
        let mut chain = original.chain().collect::<Vec<_>>();
        chain.reverse();
        for error in chain {
            match &mut final_error {
                None => final_error = Some(Error::cast_error(error).into()),
                Some(ferr) => ferr.extend(&error.to_string()),
            }
        }
        let mut final_error = final_error.unwrap_or(UserError {
            msg: vec![],
            kind: ErrorKind::Unspecified,
        });
        final_error.msg.reverse();
        final_error
    }
}

impl Into<tide::Response> for UserError {
    fn into(self) -> tide::Response {
        let value = serde_json::to_value(&self).unwrap();
        let status_code: tide::StatusCode = self.kind.into();
        let mut resp = tide::Response::builder(status_code).body(value).build();
        let as_anyhow: anyhow::Error = self.into();
        let tide_error = tide::Error::new(status_code, as_anyhow);
        resp.set_error(tide_error);
        resp
    }
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;
        write!(f, ": {:#?}", self.msg)?;
        Ok(())
    }
}

impl std::error::Error for UserError {}
