use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let hash_map = &dart::import("dart:collection", "HashMap");

    let tokens = quote! {
        print_greeting(String name) {
            print($[str](Hello $(name)));
        }

        $hash_map<int, String> map() {
            return new $hash_map<int, String>();
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Dart>();
    let config = dart::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}
