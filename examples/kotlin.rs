use genco::fmt;
use genco::lang::kotlin;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let greeter_ty = &kotlin::import("com.example.utils", "Greeter");

    let tokens: kotlin::Tokens = quote! {
        fun main() {
            val greeter = $greeter_ty("Hello Kotlin from Genco!");
            println(greeter.greet());
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt_config = fmt::Config::from_lang::<kotlin::Kotlin>()
        .with_indentation(fmt::Indentation::Space(4))
        .with_newline("\n");

    let lang_config = kotlin::Config::default().with_package("com.example");

    tokens.format_file(&mut w.as_formatter(&fmt_config), &lang_config)?;

    Ok(())
}
