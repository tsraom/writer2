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
        //  we don't need main, otherwise we are defining main twice
        //  (the other one is the writer2 main)
        //  "main.c",
    ];

    /*
     *  we need to #define libcmark_EXPORTS while building cmark
     *  because cmark_config.h #define's the export header CMARK_EXPORT as
     *  __declspec(dllexport) properly only if libcmark_EXPORTS is defined.
     *
     *  the cmark source code we are using is originally intended to be
     *  built using cmake, which defines this symbol as per CMakeLists.txt.
     */

    cc::Build::new()
        .include("src/cmark/csrc")
        .define("libcmark_EXPORTS", None)
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
