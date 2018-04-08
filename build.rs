extern crate phf_codegen;
extern crate serde_json;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    assert!(
        Command::new("yarn")
            .status()
            .expect("failed to run `yarn`")
            .success()
    );
    assert!(
        Command::new("yarn")
            .arg("run")
            .arg("webpack")
            .status()
            .expect("failed to run `yarn run webpack`")
            .success()
    );

    let manifest = serde_json::from_reader::<_, HashMap<String, String>>(
        File::open("manifest.json").expect("failed to open `manifest.json`"),
    ).expect("failed to parse the mainfest");

    let mut out = File::create(
        Path::new(&std::env::var_os("OUT_DIR").unwrap()).join("webpack_manifest.rs"),
    ).unwrap();
    write!(
        out,
        "static MANIFEST: ::phf::Map<&'static str, &'static str> = "
    ).unwrap();
    let mut map = phf_codegen::Map::new();
    for (key, value) in &manifest {
        let path = Path::new("/").join(
            Path::new(&value)
                .strip_prefix(".")
                .expect("path didn't start with a '.'"),
        );
        map.entry(&key[..], &format!("{:?}", path.to_string_lossy()));
    }
    map.build(&mut out).unwrap();
    writeln!(out, ";").unwrap();

    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=static/material.ts");
    println!("cargo:rerun-if-changed=static/search.ts");
    println!("cargo:rerun-if-changed=static/style.scss");
    println!("cargo:rerun-if-changed=webpack.config.js");
    println!("cargo:rerun-if-changed=yarn.lock");
}
