#![feature(proc_macro_hygiene)]

use genco::rust::imported;
use genco::{quote, Rust, Tokens};

fn main() {
    // Import the LittleEndian item, without referencing it through the last
    // module component it is part of.
    let little_endian = imported("byteorder", "LittleEndian").qualified();
    let big_endian = imported("byteorder", "BigEndian");

    // This is a trait, so only import it into the scope (unless we intent to
    // implement it).
    let write_bytes_ext = imported("byteorder", "WriteBytesExt").alias("_");

    let tokens: Tokens<Rust> = quote! {
        @write_bytes_ext
        let mut wtr = vec![];
        wtr.write_u16::<#little_endian>(517).unwrap();
        wtr.write_u16::<#big_endian>(768).unwrap();
        assert_eq!(wtr, vec![5, 2, 3, 0]);
    };

    println!("{}", tokens.to_file_string().unwrap());
}
