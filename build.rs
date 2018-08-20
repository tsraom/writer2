extern crate cc;
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from("src/cmark/csrc");
    let files = vec![
        "cmark.c",
        "node.c",
        "iterator.c",
        "blocks.c",
        "inlines.c",
        "scanners.c",
        "utf8.c",
        "buffer.c",
        "references.c",
        "render.c",
        "man.c",
        "xml.c",
        "html.c",
        "commonmark.c",
        "latex.c",
        "houdini_href_e.c",
        "houdini_html_e.c",
        "houdini_html_u.c",
        "cmark_ctype.c",

        //  there is a main.c but we don't need it, otherwise we are defining
        //  main twice (the other one is the writer2 main, our own main)
    ];

    cc::Build::new()
        .include("src/cmark/csrc")
        .files(files.into_iter().map(|file| {
            dir.join(PathBuf::from(file))
        }))
        .compile("cmark");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search=native={}", out_dir.display());

    let bindings = bindgen::Builder::default()
        .header("src/cmark/csrc/wrapper.h")
        .generate()
        .expect("could not generate bindings for cmark");

    bindings.write_to_file(out_dir.join("bindings.rs"))
        .expect("could not write bindings");
}

