use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    // Import the LittleEndian item, without referencing it through the last
    // module component it is part of.
    let little_endian = rust::import("byteorder", "LittleEndian");
    let big_endian = rust::import("byteorder", "BigEndian").qualified();

    // Trait that we need to import to make use of write_u16.
    let write_bytes_ext = rust::import("byteorder", "WriteBytesExt").with_alias("_");

    // Trait that we import since we want to return it.
    let result = rust::import("anyhow", "Result");

    let tokens = quote! {
        $(register(write_bytes_ext))

        fn test() -> $result {
            let mut data = vec![];
            data.write_u16::<$little_endian>(517)?;
            data.write_u16::<$big_endian>(768)?;
            println!("{:?}", data);
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Rust>().with_indentation(fmt::Indentation::Space(4));

    let config = rust::Config::default()
        // Prettier imports and use.
        .with_default_import(rust::ImportMode::Qualified);

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}
