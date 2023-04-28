use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    // Import `printf` from `<stdio.h>`
    let printf = &c::include("stdio.h", "printf", false);

    let day = "tuesday";
    let name = "George";

    let tokens = quote! {
        const char* greet_user() {
            return $(quoted(format!("Hello {}!", name)));
        }

        int main() {
            const char* current_day = $(quoted(day));
            $printf("%s\n", current_day);
            $printf("%s\n", greet_user());
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<C>();
    let config = c::Config::default().with_package("main");

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}
