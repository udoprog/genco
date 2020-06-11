use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let println = &go::imported("fmt", "Println");

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

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Go>();
    let config = go::Config::default().with_package("main");

    tokens.format_file(&mut w.as_formatter(fmt), &config)?;
    Ok(())
}
