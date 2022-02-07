extern crate proc_macro;
use std::{collections::HashMap, ffi::OsStr};

use ignore::{DirEntry, WalkBuilder};
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, LitBool, LitStr, Token,
};
use tokenstream2_tmpl::{interpolate, Interpolate};

mod kw {
    syn::custom_keyword!(exclude_dir);
    syn::custom_keyword!(filter_extension);
}

fn is_extension(extension: &str, dir_entry: &DirEntry) -> bool {
    is_dir(dir_entry) || dir_entry.path().extension() == Some(OsStr::new(extension))
}

fn is_dir(dir_entry: &DirEntry) -> bool {
    dir_entry.path().is_dir()
}

#[cfg_attr(test, derive(Debug, PartialEq))]
struct IgnoreInput {
    path: LitStr,
    exclude_dir: bool,
    filter_extension: Option<String>,
}

impl IgnoreInput {
    fn get_files(&self) -> Vec<File> {
        let filter_extension = self.filter_extension.clone();

        WalkBuilder::new(self.path.value())
            .filter_entry(move |entry| {
                if let Some(extension) = &filter_extension {
                    is_extension(extension, entry)
                } else {
                    true
                }
            })
            .build()
            .filter_map(std::result::Result::ok)
            .filter(|entry| !self.exclude_dir || !is_dir(entry))
            .map(|entry| {
                let path = entry.path().display().to_string();
                let path = path.trim_start_matches("./");

                File {
                    path: path.to_string(),
                    path_ident: path
                        .replace('.', "_dot_")
                        .replace('/', "_")
                        .replace('-', "_")
                        .to_lowercase(),
                }
            })
            .collect()
    }
}

impl Parse for IgnoreInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;
        let mut exclude_dir = false;
        let mut filter_extension = None;

        while !input.is_empty() {
            input.parse::<Token!(,)>()?;
            let lookahead = input.lookahead1();

            if lookahead.peek(kw::exclude_dir) {
                input.parse::<kw::exclude_dir>()?;
                input.parse::<Token!(=)>()?;

                exclude_dir = input.parse::<LitBool>()?.value;
            } else if lookahead.peek(kw::filter_extension) {
                input.parse::<kw::filter_extension>()?;
                input.parse::<Token!(=)>()?;

                filter_extension = Some(input.parse::<LitStr>()?.value());
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(Self {
            path,
            exclude_dir,
            filter_extension,
        })
    }
}

impl Interpolate for IgnoreInput {
    fn interpolate(&self, stream: TokenStream) -> TokenStream {
        let mut output = TokenStream::new();

        let files = self
            .get_files()
            .into_iter()
            .map(|file| file.interpolate(stream.clone()));

        output.extend(files);

        output
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
struct File {
    path_ident: String,
    path: String,
}

impl Interpolate for File {
    fn interpolate(&self, stream: TokenStream) -> TokenStream {
        let mut replacements: HashMap<_, &dyn ToTokens> = HashMap::new();

        let path = LitStr::new(&self.path, Span::call_site());
        let path_ident = Ident::new(&self.path_ident, Span::call_site());

        replacements.insert("PATH", &path);
        replacements.insert("PATH_IDENT", &path_ident);

        interpolate(stream, &replacements)
    }
}

/// Use `ignore` to find files or directories at compile time, then apply the supplied template to
/// each file or directory found
/// The template can include the following two placeholders:
/// - PATH_IDENT: a rust safe identifier for the path
/// - PATH: a string literal to the path
///
/// # Arguments
/// * String path to search in
/// * `exclude_dir` - Should directories be excluded from results
/// * `filter_extension` - Filter results to only those with the given file extension
///
/// # Examples
/// The following creates a function for each toml file
/// ```
/// use test_macros::tmpl_ignore;
///
/// #[tmpl_ignore("./", exclude_dir = true, filter_extension = "toml")]
/// fn PATH_IDENT() -> String {
///     PATH.to_string()
/// }
///
/// fn main() {
///     assert_eq!(cargo_dot_toml(), "Cargo.toml");
/// }
/// ```
#[proc_macro_attribute]
pub fn tmpl_ignore(
    tokens: proc_macro::TokenStream,
    template: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as IgnoreInput);

    input.interpolate(template.into()).into()
}

#[cfg(test)]
mod tests {
    use super::{File, IgnoreInput};
    use syn::parse_str;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn parse() -> Result<()> {
        let actual: IgnoreInput = parse_str("\"foo\"")?;
        let expected = IgnoreInput {
            path: parse_str("\"foo\"")?,
            exclude_dir: false,
            filter_extension: None,
        };

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn parse_exclude_dir() -> Result<()> {
        let actual: IgnoreInput = parse_str("\"foo\", exclude_dir = true")?;
        let expected = IgnoreInput {
            path: parse_str("\"foo\"")?,
            exclude_dir: true,
            filter_extension: None,
        };

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn parse_filter_extension() -> Result<()> {
        let actual: IgnoreInput = parse_str("\"foo\", filter_extension = \"json\"")?;
        let expected = IgnoreInput {
            path: parse_str("\"foo\"")?,
            exclude_dir: false,
            filter_extension: Some("json".to_string()),
        };

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn parse_all_options() -> Result<()> {
        let actual: IgnoreInput =
            parse_str("\"foo\", filter_extension = \"json\", exclude_dir = true")?;
        let expected = IgnoreInput {
            path: parse_str("\"foo\"")?,
            exclude_dir: true,
            filter_extension: Some("json".to_string()),
        };

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "expected string literal")]
    fn parse_missing_path() {
        parse_str::<IgnoreInput>("filter_extension = \"json\", exclude_dir = true").unwrap();
    }

    #[test]
    fn get_files() -> Result<()> {
        let options = IgnoreInput {
            path: parse_str("\"./\"")?,
            exclude_dir: true,
            filter_extension: Some("toml".to_string()),
        };

        let actual = options.get_files();
        let expected = vec![File {
            path: "Cargo.toml".to_string(),
            path_ident: "cargo_dot_toml".to_string(),
        }];

        assert_eq!(actual, expected);

        Ok(())
    }
}
