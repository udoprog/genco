use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let car = &java::imported("se.tedro", "Car");
    let list = &java::imported("java.util", "List");
    let array_list = &java::imported("java.util", "ArrayList");

    let tokens = quote! {
        public class HelloWorld {
            public static void main(String[] args) {
                #list<#car> cars = new #array_list<#car>();

                cars.add(new #car("Volvo"));
                cars.add(new #car("Audi"));

                for (#car car : cars) {
                    System.out.println(car);
                }
            }
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());
    let mut formatter = w.as_formatter(fmt::Config::from_lang::<Java>().with_newline("\n\r"));
    let config = java::Config::default().with_package("se.tedro");

    tokens.format_file(&mut formatter, &config)?;
    Ok(())
}
