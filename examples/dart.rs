use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let sqrt = dart::imported("dart:math", "sqrt");

    let tokens = quote! {
        class Position {
            int x;
            int y;

            double distanceTo(Position other) {
                var dx = other.x - x;
                var dy = other.y - y;
                return #sqrt(dx * dx + dy * dy);
            }
        }

        main() {
            var origin = new Position()
                ..x = 0
                ..y = 0;

            var p = new Position()
                ..x = -5
                ..y = 6;

            print(origin.distanceTo(p));
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Dart>();
    let config = dart::Config::default();

    tokens.format_file(&mut w.as_formatter(fmt), &config)?;
    Ok(())
}
