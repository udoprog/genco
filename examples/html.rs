use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let url = "https://example.com/?a=1&b=2";
    let message = "Text like <b>this</b> & that is escaped";

    let tokens: html::Tokens = quote! {
        <html>
            <head>
                <title>Example</title>
            </head>
            <body>
                <a href=$(quoted(url))>Link</a>
                <p>$(html::text(message))</p>
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
