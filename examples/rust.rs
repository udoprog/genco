#![feature(proc_macro_hygiene)]

use genco::rust::{imported, Config};
use genco::{quote, FormatterConfig, Rust};
use rand::Rng;

use std::fmt;

fn main() -> fmt::Result {
    // Import the LittleEndian item, without referencing it through the last
    // module component it is part of.
    let little_endian = imported("byteorder", "LittleEndian").qualified();
    let big_endian = imported("byteorder", "BigEndian");

    // This is a trait, so only import it into the scope (unless we intent to
    // implement it).
    let write_bytes_ext = imported("byteorder", "WriteBytesExt").alias("_");
    let read_bytes_ext = imported("byteorder", "ReadBytesExt").alias("_");

    // Iterators can be tokenized using `tokenize_iter`, as long as they contain
    // something which can be converted into a stream of tokens.
    let numbers = (0..10).map(|_| rand::thread_rng().gen::<i16>());

    let tokens = quote! {
        // Markup used for imports without an immediate use.
        #@(write_bytes_ext)
        #@(read_bytes_ext)

        fn test() {
            let mut wtr = vec![];
            wtr.write_u16::<#little_endian>(517).unwrap();
            wtr.write_u16::<#big_endian>(768).unwrap();
            assert_eq!(wtr, vec![#numbers,*]);
        }
    };

    // Simpler printing with default indentation:
    // println!("{}", tokens.to_file_string()?);

    tokens.to_io_writer_with(
        std::io::stdout().lock(),
        Config::default(),
        FormatterConfig::from_lang::<Rust>().with_indentation(2),
    )?;

    Ok(())
}
