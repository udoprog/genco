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

    let tokens = quote! {
        // Markup used for imports without an immediate use.
        #@(write_bytes_ext)
        #@(read_bytes_ext)

        fn test() {
            let mut wtr = vec![];
            wtr.write_u16::<#little_endian>(517).unwrap();
            wtr.write_u16::<#big_endian>(768).unwrap();
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());
    let fmt_config = fmt::Config::from_lang::<Rust>().with_indentation(2);
    let mut formatter = w.as_formatter(fmt_config);
    let config = rust::Config::default();

    tokens.format_file(&mut formatter, &config)?;
    Ok(())
}
