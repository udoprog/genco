use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let date = &swift::import("Foundation", "Date");
    let logger = &swift::import_implementation_only("Logging", "Logger");

    let tokens: swift::Tokens = quote! {
        struct Greeter {
            let name: String
            let logger = $logger(label: "greeter")

            func greet() -> String {
                logger.info("Greeting at " + String(describing: $date()))
                return "Hello, " + name + "!"
            }
        }

        print(Greeter(name: $(quoted("Swift from Genco"))).greet())
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt_config = fmt::Config::from_lang::<swift::Swift>()
        .with_indentation(fmt::Indentation::Space(4))
        .with_newline("\n");

    let lang_config = swift::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt_config), &lang_config)?;

    Ok(())
}
