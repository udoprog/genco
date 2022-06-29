use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let flask = &python::import("flask", "Flask");

    let tokens = quote! {
        app = $flask(__name__)

        @app.route('/')
        def hello():
            return "Hello World!"

        if __name__ == "__main__":
            app.run()
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Python>();
    let config = python::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}
