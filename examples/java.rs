#![feature(proc_macro_hygiene)]

use genco::java::{imported, Config};
use genco::{quote, Java, Tokens};

use anyhow::Result;

fn main() -> Result<()> {
    let car = imported("se.tedro", "Car");
    let list = imported("java.util", "List");
    let array_list = imported("java.util", "ArrayList");

    let tokens: Tokens<Java> = quote! {
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

    println!(
        "{}",
        tokens.to_file_with(
            Config::default()
                .with_package("se.tedro")
                .with_indentation(8)
        )?
    );

    Ok(())
}
