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
use std::io::{ BufReader, BufWriter };

mod cmark;

mod converter;
use converter::Converter;

mod asset;
use asset::{ Asset, AssetType };

mod program_options;
use program_options::ProgramOptions;

use getopts::Options;
use std::env;
use std::path::PathBuf;

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
    output_dir: &PathBuf,
    assets_dir: &PathBuf
) -> Vec<Asset> {
    let curr_dir = env::current_dir().unwrap();
    let src_dir = curr_dir.join(assets_dir);
    let dst_dir = output_dir.join(assets_dir);

    copy_assets(&curr_dir, &src_dir, &dst_dir)
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

                        let mut converter = Converter::new();
                        converter.convert(&mut reader, &mut writer, assets, dist);
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
    let info = ProgramOptions::parse_options()
        .expect("bad program options");

    if info.verbose {
        println!("the current working directory is \"{}\"", env::current_dir().unwrap().display());
    }

    let assets = prepare_assets(&info.output_dir, &PathBuf::from("assets"));
    convert_dir(&info.input_dir, &info.output_dir, info.verbose, &assets, 0);
}
