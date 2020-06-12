use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    // Import the LittleEndian item, without referencing it through the last
    // module component it is part of.
    let little_endian = rust::imported("byteorder", "LittleEndian");
    let big_endian = rust::imported("byteorder", "BigEndian").prefixed();

    // This is a trait, so only import it into the scope (unless we intent to
    // implement it).
    let write_bytes_ext = rust::imported("byteorder", "WriteBytesExt").alias("_");
    let read_bytes_ext = rust::imported("byteorder", "ReadBytesExt").alias("_");
    let error = rust::imported("std::error", "Error");

    let tokens = quote! {
        #((write_bytes_ext, read_bytes_ext).register())

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
