use anyhow::Result;

// Skipping fmt is needed until this fix is released
// https://github.com/rust-lang/rustfmt/pull/5142
#[rustfmt::skip]
mod helpers;

use helpers::generate;

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

            let expected = include_str!(concat!("examples/", stringify!($name), "/output.json"));

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
