use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let printf = &c::include_system("stdio.h", "printf");

    let day = "tuesday";
    let name = "George";

    let tokens = quote! {
        const char* greet_user() {
            return $(quoted(format_args!("Hello {name}!")));
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
    let config = c::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}
