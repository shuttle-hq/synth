use bindlang::{bindlang, bindlang_main};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[bindlang]
pub fn bound(_b: bool, _i: i64, _f: f32, _u: u8) {
    // nothing to see here
}

#[bindlang]
pub fn anotherone() {
    // still nothing
}

#[bindlang]
#[derive(Clone, Debug, Default)]
pub struct MyType;

//#[bindlang]
impl Display for MyType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

#[bindlang]
impl MyType {
    pub fn new() -> Self { MyType }
    
    pub fn answer(&self) -> usize {
        42
    }
}

bindlang_main!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut koto = koto::Koto::default();
    bindlang_init(&mut koto.prelude());
    koto.compile("import MyType\nMyType.new().answer()")?;
    println!("{}", koto.run()?);
    Ok(())
}
