use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let println = &go::import("fmt", "Println");

    let day = "tuesday";
    let name = "George";

    let tokens = quote! {
        func main() {
            var currentDay string
            currentDay = $(quoted(day))
            $println(currentDay)
            $println(greetUser())
        }

        func greetUser() string {
            return $(quoted(format_args!("Hello {name}!")))
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Go>();
    let config = go::Config::default().with_package("main");

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}
