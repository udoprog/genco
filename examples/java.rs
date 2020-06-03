#![feature(proc_macro_hygiene)]

use genco::java::{imported, Config, Java};
use genco::{quote, FormatterConfig};

use anyhow::Result;

fn main() -> Result<()> {
    let car = imported("se.tedro", "Car");
    let list = imported("java.util", "List");
    let array_list = imported("java.util", "ArrayList");

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
        Config::default().with_package("se.tedro"),
        FormatterConfig::from_lang::<Java>().with_newline("\n\r"),
    )?;

    Ok(())
}
