use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let println = &go::imported("fmt", "Println");

    let day = "tuesday";
    let name = "George";

    let tokens = quote! {
        func main() {
            var currentDay string
            currentDay = #(day.quoted())
            #println(currentDay)
            #println(greetUser())
        }

        func greetUser() string {
            return #(format!("Hello {}!", name).quoted())
        }
    };

    tokens.to_io_writer_with(
        std::io::stdout().lock(),
        go::Config::default().with_package("main"),
        FormatterConfig::from_lang::<Go>(),
    )?;

    Ok(())
}
