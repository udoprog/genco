#![feature(proc_macro_hygiene)]

use genco::go::{imported, Config, Go};
use genco::{quote, FormatterConfig, Quoted as _};

use anyhow::Result;

fn main() -> Result<()> {
    let println = imported("fmt", "Println");

    let day = "tuesday";
    let name = "George";

    let tokens = quote! {
        func main() {
            var currentDay string
            currentDay = #(day.quoted())
            #println(currentDay)
            #println(greetUser())
        }

        func greetUser() string {
            return #(format!("Hello {}!", name).quoted())
        }
    };

    tokens.to_io_writer_with(
        std::io::stdout().lock(),
        Config::default().with_package("main"),
        FormatterConfig::from_lang::<Go>(),
    )?;

    Ok(())
}
