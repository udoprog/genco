use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let react = &js::import_default("react", "React");
    let display = &js::import_default("./Display", "Display");
    let button_panel = &js::import_default("./ButtonPanel", "ButtonPanel");
    let calculate = &js::import_default("../logic/calculate", "calculate");

    let tokens = quote! {
        export default class App extends #react.Component {
            state = {
                total: null,
                next: null,
                operation: null,
            };

            handleClick = buttonName => {
                this.setState(#calculate(this.state, buttonName));
            };

            render() {
                return (
                    <div className="component-app">
                        <#display value={this.state.next || this.state.total || "0"} />
                        <#button_panel clickHandler={this.handleClick} />
                    </div>
                );
            }
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<JavaScript>();
    let config = js::Config::default();

    tokens.format_file(&mut w.as_formatter(fmt), &config)?;
    Ok(())
}
