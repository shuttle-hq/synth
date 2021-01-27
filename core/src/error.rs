use serde::{Deserialize, Serialize};

macro_rules! generate_error_variants {
    {
	$(#[$attr:meta])*
	$vis:vis enum $id:ident {
	    $(
		$(#[$variant_attr:meta])*
		$variant:ident -> $func_name:ident,
	    )*
	}
    } => {
	$(#[$attr])*
	$vis enum $id {
	    $(
		$(#[$variant_attr])*
		$variant,
	    )*
	}

	impl Error {
	    $(
		#[inline]
		pub fn $func_name<R: AsRef<str>>(msg: R) -> Self {
	            Self::new_target_debug($id::$variant, msg)
		}
	    )*
	}
    }
}

/// This macro is a convenience for error handling logic in synth.
/// The crux of the situation is the synth contract is that no sensitive information is ever stored
/// except when explicitly given permission by the user. This includes logs.
/// Our definition of sensitive information is:
///     1) User defined values in data. For example in { "name" : "John" }, "John" is considered to be sensitive where as "age" is not
///         and the fact it is an 'Object' is not
/// Our definition of non-sensitive is:
///     1) The schema (namespace names, collection names, fields names, field types etc.)
///     2) The generator graph (distributions, detected cycles, malformed generators, etc.)
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Error {
    pub msg: Option<String>,
    pub kind: ErrorKind,
    pub target: Target,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Target {
    Debug,
    Release,
}

impl Error {
    pub fn cast_error(error: &(dyn std::error::Error + 'static)) -> Self {
        if let Some(crate_error) = error.downcast_ref::<Error>() {
            return crate_error.clone();
        }

        if let Some(error) = error.downcast_ref::<serde_json::Error>() {
            let crate_error: Error = error.clone().into();
            return crate_error.clone();
        }

        Error::unspecified(error.to_string())
    }
}

impl Error {
    pub fn new_with_target<R: AsRef<str>>(kind: ErrorKind, msg: R, target: Target) -> Self {
        Self {
            msg: Some(msg.as_ref().to_string()),
            kind,
            target,
        }
    }

    pub fn new_target_debug<R: AsRef<str>>(kind: ErrorKind, msg: R) -> Self {
        Self {
            msg: Some(msg.as_ref().to_string()),
            kind,
            target: Target::Debug,
        }
    }

    pub fn new_target_release<R: AsRef<str>>(kind: ErrorKind, msg: R) -> Self {
        Self {
            msg: Some(msg.as_ref().to_string()),
            kind,
            target: Target::Release,
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn sensitive(&self) -> &bool {
        match self.target {
            Target::Debug => &true,
            Target::Release => &false,
        }
    }
}

impl From<&serde_json::Error> for Error {
    fn from(sje: &serde_json::Error) -> Self {
        Error::bad_request(sje.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(msg) = self.msg.as_ref() {
            write!(f, ": {}", msg)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

generate_error_variants!(
    #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum ErrorKind {
        NotFound -> not_found,
        BadRequest -> bad_request,
        Compilation -> compilation,
        Serialization -> serialization,
        Unspecified -> unspecified,
        Inference -> inference,
        Optionalise -> optionalise,
        Override -> _override, // override is a reserved keyword :(
        Conflict -> conflict,
    }
);

impl Into<tide::StatusCode> for ErrorKind {
    fn into(self) -> tide::StatusCode {
        match self {
            Self::NotFound => tide::StatusCode::NotFound,
            Self::BadRequest => tide::StatusCode::BadRequest,
            Self::Serialization | Self::Compilation | Self::Unspecified => {
                tide::StatusCode::InternalServerError
            }
            Self::Inference => tide::StatusCode::BadRequest,
            Self::Override => tide::StatusCode::BadRequest,
            Self::Optionalise => tide::StatusCode::BadRequest,
            Self::Conflict => tide::StatusCode::Conflict,
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ErrorKind {}
