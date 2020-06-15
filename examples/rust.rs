use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    // Import the LittleEndian item, without referencing it through the last
    // module component it is part of.
    let little_endian = rust::import("byteorder", "LittleEndian");
    let big_endian = rust::import("byteorder", "BigEndian").qualified();

    // This is a trait, so only import it into the scope (unless we intent to
    // implement it).
    let write_bytes_ext = rust::import("byteorder", "WriteBytesExt").with_alias("_");
    let read_bytes_ext = rust::import("byteorder", "ReadBytesExt").with_alias("_");
    let error = rust::import("std::error", "Error");

    let tokens = quote! {
        #(register((write_bytes_ext, read_bytes_ext)))

        fn test() -> Result<(), Box<dyn #error>> {
            let mut wtr = vec![];
            wtr.write_u16::<#little_endian>(517)?;
            wtr.write_u16::<#big_endian>(768)?;
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Rust>().with_indentation(fmt::Indentation::Space(2));
    let config = rust::Config::default();

    tokens.format_file(&mut w.as_formatter(fmt), &config)?;
    Ok(())
}
