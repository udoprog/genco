use std::env;
use std::process::Command;
use std::str;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let version = rustc_version().unwrap_or(RustcVersion {
        minor: u32::MAX,
        nightly: false,
    });

    if version.nightly {
        println!("cargo:rustc-cfg=proc_macro_span");
    }
}

struct RustcVersion {
    #[allow(unused)]
    minor: u32,
    nightly: bool,
}

fn rustc_version() -> Option<RustcVersion> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = str::from_utf8(&output.stdout).ok()?;
    let nightly = version.contains("nightly") || version.contains("dev");
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    let minor = pieces.next()?.parse().ok()?;
    Some(RustcVersion { minor, nightly })
}
