use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let nixpkgs = &nix::inherit("inputs", "nixpkgs");
    let pkgs = &nix::variable(
        "pkgs",
        quote! {
            import $nixpkgs {
                inherit ($nixpkgs) system;
                config.allowUnfree = true;
                overlays = [];
            }
        },
    );
    let mk_default = &nix::with("lib", "mkDefault");

    let tokens = quote! {
        {
            imports = [];
            environment.systemPackages = with $pkgs; [];
            networking.useDHCP = $mk_default true;
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Nix>();
    let config = nix::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}
