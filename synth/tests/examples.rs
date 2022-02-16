use anyhow::Result;
use paste::paste;

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

macro_rules! test_scenarios {
    ($($name:ident / $ns:ident,)*) => {
        $(
        paste!{
        #[async_std::test]
        async fn [<$name _scenario>]() -> Result<()> {
            let actual = generate_scenario(concat!(
                "../examples/",
                stringify!($name),
                "/",
                stringify!($ns)
            ), Some("users-only".to_string()))
            .await?;

            let expected = include_str!(concat!("examples/", stringify!($name), "/scenarios/users-only.json"))
                .replace("\r\n", "\n");

            assert_eq!(actual, expected);

            Ok(())
        }
        }
        )*
    };
}

test_scenarios!(bank / bank_db,);
