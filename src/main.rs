/*
 *  writer2
 *
 *  getting the ssg to work first
 *  because writing parsers for cmark takes forever
 *
 *  ideas for options:
 *  -   replace / don't replace existing files
 */

extern crate getopts;
extern crate libc;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate newtype_derive;

#[macro_use]
extern crate custom_derive;

#[allow(unused)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[link(name="cmark", kind="static")]
mod bind {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::fs;
use std::io::{BufReader, BufWriter, Read, Write};

mod cmark;
use cmark::{ Parser, Iter };
use cmark::Options as CmarkOptions;

mod converter;
use converter::Converter;

mod asset;
use asset::{ Asset, AssetType };

use getopts::Options;
use std::env;
use std::path::PathBuf;

//  info, things like input / output directory
struct ProgramInfo {
    input_dir: PathBuf,
    output_dir: PathBuf,
    verbose: bool,
}

//  display how to use this program
fn print_usage(program: &str, opts: &Options) {
    let brief = format!("usage: {} INPUT-DIR [options]", program);
    print!("{}", opts.usage(&brief));
}

fn copy_assets(
    input_dir: &PathBuf,
    src_dir: &PathBuf,
    dst_dir: &PathBuf
) -> Vec<Asset> {
    fs::create_dir_all(dst_dir.clone());

    let iter = fs::read_dir(src_dir).unwrap();
    let mut res: Vec<Asset> = Vec::new();

    for entry in iter {
        let mut path = entry.unwrap().path();
        let new_dst_dir = dst_dir.join(path.strip_prefix(src_dir).unwrap());

        match path.is_dir() {
            true => {
                let mut vec = copy_assets(input_dir, &path, &new_dst_dir);
                res.append(&mut vec);
            },

            false => {
                fs::copy(&path, &new_dst_dir);
                res.push(Asset::new(
                    path.strip_prefix(input_dir).unwrap().to_path_buf(),
                    AssetType::guess(&path)
                ));
            },
        }
    }

    res
}

//  copy assets over to the output directory
fn prepare_assets(
    info: &ProgramInfo,
    assets_dir: &PathBuf
) -> Vec<Asset> {
    let curr_dir = env::current_dir().unwrap();
    let src_dir = curr_dir.join(assets_dir);
    let dst_dir = info.output_dir.join(assets_dir);

    copy_assets(&curr_dir, &src_dir, &dst_dir)
}

//  write assets into a file
fn write_assets<W>(writer: &mut BufWriter<W>, assets: &Vec<Asset>, dist: usize)
    where W: Write
{
    for asset in assets {
        match asset.asset_type() {
            &AssetType::Css => {
                writeln!(
                    writer,
                    "<link rel=\"stylesheet\" href=\"{}{}\" type=\"text/css\">",
                    "../".repeat(dist),
                    asset.path().display()
                ).expect(&format!("failed to write asset \"{}\" into file", asset.path().display()));
            },

            &AssetType::Js => {
                writeln!(
                    writer,
                    "<script src=\"{}{}\" type=\"text/javascript\"></script>",
                    "../".repeat(dist),
                    asset.path().display()
                ).unwrap();
            },

            &AssetType::Other => {},
        }
    }
}

//  convert a markdown into html
fn convert<R, W>(
    reader: &mut BufReader<R>,
    writer: &mut BufWriter<W>,
    assets: &Vec<Asset>,
    dist: usize
)
    where R: Read, W: Write
{
    let mut read_buffer = String::new();
    reader.read_to_string(&mut read_buffer).unwrap();

    let iter = Iter::from_parser({
        let mut parser = Parser::new(CmarkOptions::DEFAULT);
        parser.feed(read_buffer.as_str(), read_buffer.len()).expect(
            "feeding failed"
        );
        parser
    });
    
    /*  this always works
    let write_buffer = cmark::markdown_to_html(
        read_buffer.as_str(),
        read_buffer.len(),
        CmarkOptions::DEFAULT
    ).expect("markdown-to-html conversion failed");
    */

    writer.write(b"<!DOCTYPE html>\n\
<html>\n\
<head>\n\
<meta charset=\"UTF-8\">\n\
<title>Title</title>\n").unwrap();
    {
        write_assets(writer, assets, dist);
    }
    writer.write(b"</head>\n\
<body>\n\
<div class=\"container u-full-width\">\n").unwrap();

    let mut converter = Converter::new();
    converter.convert(iter, writer);

    writer.write(b"</div>\n\
</body>\n\
<script>hljs.initHighlightingOnLoad();</script>\n\
</html>").unwrap();
}

fn parse_options() -> Result<ProgramInfo, ()> {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optopt("o", "output-dir", "set output directory", "OUTPUT-DIR");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "verbose", "be very wordy");

    let matches = opts.parse(&args[1..]).unwrap_or_else(|f| panic!(f.to_string()));

    if matches.opt_present("h") {
        print_usage(program, &opts);
    }

    let input_dir = match matches.free.is_empty() {
        true => {
            print_usage(program, &opts);
            return Err(());
        },

        false => PathBuf::from(matches.free[0].clone()),
    };

    let output_dir = match matches.opt_str("o") {
        Some(s) => PathBuf::from(s),
        None => input_dir.clone(),
    };

    Ok(ProgramInfo {
        input_dir: input_dir,
        output_dir: output_dir,
        verbose: matches.opt_present("v"),
    })
}

//  convert all markdowns in a directory and copy whatever else
//  dist is the number of levels from the original output directory to output_dir
//  precondition: input_dir.is_dir() must be true
fn convert_dir(input_dir: &PathBuf, output_dir: &PathBuf, verbose: bool, assets: &Vec<Asset>, dist: usize) {
    let iter = fs::read_dir(input_dir)
        .expect(&format!("could not open directory \"{}\"", input_dir.display()));

    for entry in iter {
        let mut path = entry.expect("I/O error during iteration").path();
        match path.is_dir() {
            true => {
                if verbose {
                    println!("directory \"{}\" is a path, going in...", path.display());
                }

                convert_dir(
                    &path,
                    &output_dir.join(path.strip_prefix(input_dir).expect("error during prefix-stripping")),
                    verbose,
                    assets,
                    dist + 1
                );
            },

            false => {
                let parent_dir = path.parent()
                    .expect(&format!("directory \"{}\" does not have a parent", path.display()));
                fs::create_dir_all(output_dir.join(parent_dir.strip_prefix(input_dir).expect("error during prefix-stripping")))
                    .expect(&format!("could not create directory \"{}\"", path.display()));

                match path.extension() == Some(std::ffi::OsStr::new("md")) {
                    true => {
                        if verbose {
                            println!("directory \"{}\" is a markdown, converting to post", path.display());
                        }

                        let input = fs::File::open(path.clone())
                            .expect(&format!("could not open file \"{}\"", path.display()));
                        let mut reader = BufReader::new(input);

                        let mut html_path = output_dir.join(path.strip_prefix(input_dir).expect("error during prefix-stripping"));
                        html_path.set_extension("html");
                        let output = fs::OpenOptions::new().write(true).truncate(true).create(true).open(html_path.clone())
                            .expect(&format!("could not open file \"{}\"", html_path.display()));
                        let mut writer = BufWriter::new(output);

                        convert(&mut reader, &mut writer, assets, dist);
                    },

                    false => {
                        if verbose {
                            println!("directory \"{}\" is a non-markdown file, copying over", path.display());
                        }

                        fs::copy(&path, &output_dir.join(path.strip_prefix(input_dir).expect("error during prefix-stripping")))
                            .expect(&format!("failed to copy \"{}\"", path.display()));
                    },
                }
            },
        }
    }
}

fn main() {
    let info = parse_options()
        .expect("bad program options");

    if info.verbose {
        println!("the current working directory is \"{}\"", env::current_dir().unwrap().display());
    }

    let assets = prepare_assets(&info, &PathBuf::from("assets"));
    convert_dir(&info.input_dir, &info.output_dir, info.verbose, &assets, 0);
}
