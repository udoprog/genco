use anyhow::Result;
use genco::prelude::*;

fn main() -> Result<()> {
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

    tokens.to_io_writer_with(
        std::io::stdout().lock(),
        java::Config::default().with_package("se.tedro"),
        FormatterConfig::from_lang::<Java>().with_newline("\n\r"),
    )?;

    Ok(())
}
