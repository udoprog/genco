use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let tokens: html::Tokens = quote! {
        <html>
            <head>
                <title>Example</title>
            </head>
            <body>
                <p>This contains stuff that needs to be escaped like <></p>
            </body>
        </html>
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Html>().with_indentation(fmt::Indentation::Space(4));

    let config = html::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}
