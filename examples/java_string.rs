use genco::prelude::*;

fn main() -> Result<(), genco::fmt::Error> {
    let tokens: python::Tokens = quote!($[str](Hello World));
    assert_eq!("\"Hello World\"", tokens.to_string()?);
    Ok::<_, genco::fmt::Error>(())
}
