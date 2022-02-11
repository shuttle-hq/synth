use anyhow::Result;
use test_macros::{file_stem, parent, parent2, tmpl_ignore};

// Skipping fmt is needed until this fix is released
// https://github.com/rust-lang/rustfmt/pull/5142
#[rustfmt::skip]
mod helpers;

use helpers::{generate, generate_scenario};

macro_rules! test_examples {
    ($($name:ident / $ns:ident,)*) => {
        $(
        #[async_std::test]
        async fn $name() -> Result<()> {
            let actual = generate(concat!(
                "../examples/",
                stringify!($name),
                "/",
                stringify!($ns)
            ))
            .await?;

            let expected = include_str!(concat!("examples/", stringify!($name), "/output.json"))
                .replace("\r\n", "\n");

            assert_eq!(actual, expected);

            Ok(())
        }
        )*
    };
}

test_examples!(
    bank / bank_db,
    message_board / synth,
    random_variants / random,
);

#[tmpl_ignore(
    "examples/bank/bank_db/scenarios",
    exclude_dir = true,
    filter_extension = "json"
)]
#[async_std::test]
async fn PATH_IDENT() -> Result<()> {
    let actual = generate_scenario(
        concat!("../", parent2!(PATH)),
        Some(file_stem!(PATH).to_string()),
    )
    .await;

    assert!(
        actual.is_ok(),
        "did not expect error: {}",
        actual.unwrap_err()
    );

    let expected =
        include_str!(concat!(parent!(PATH), "/", file_stem!(PATH), ".json")).replace("\r\n", "\n");

    assert_eq!(actual.unwrap(), expected);

    Ok(())
}
