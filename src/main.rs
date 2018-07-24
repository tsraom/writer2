/*
 *  writer2
 *
 *  getting the ssg to work first
 *  because writing parsers for cmark takes forever
 *
 *  ideas for options:
 *  -   replace / don't replace existing files
 */

extern crate pulldown_cmark;
extern crate nom;
extern crate getopts;

use std::fs;
use std::io::{BufReader, BufWriter, Read, Write};

use pulldown_cmark::{Parser, html};

use getopts::Options;
use std::env;
use std::path::PathBuf;

//  info, things like input / output directory
struct ProgramInfo {
    input_dir: PathBuf,
    output_dir: PathBuf,
    verbose: bool,
}

//  asset
struct Asset {
    path: PathBuf,
    asset_type: AssetType,
}

//  asset type
enum AssetType {
    Css,
    Other,
}

//  sugar for building an asset
fn make_asset(path: &str, ty: AssetType) -> Asset {
    Asset {
        path: PathBuf::from(path),
        asset_type: ty,
    }
}

//  display how to use this program
fn print_usage(program: &str, opts: &Options) {
    let brief = format!("usage: {} INPUT-DIR [options]", program);
    print!("{}", opts.usage(&brief));
}

//  copy assets over to the output directory
fn prepare_assets(info: &ProgramInfo, assets: &Vec<Asset>) {
    let output_dir = &info.output_dir;

    for asset in assets {
        let path = &asset.path;

        let parent_dir = path.parent()
            .expect(&format!("asset \"{}\" does not have a parent", path.display()));
        let parent_dir = output_dir.join(parent_dir);

        fs::create_dir_all(parent_dir.clone())
            .expect(&format!("could not create asset directory \"{}\"", parent_dir.display()));

        let asset_dir = path.file_name()
            .expect(&format!("asset \"{}\" does not have a filename", path.display()));
        let asset_dir = parent_dir.join(asset_dir);

        fs::copy(path, asset_dir.clone())
            .expect(&format!("could not copy asset \"{}\" to \"{}\"", path.display(), asset_dir.display()));
    }
}

//  write assets into a file
fn write_assets<W>(writer: &mut BufWriter<W>, assets: &Vec<Asset>, dist: usize)
    where W: Write
{
    for asset in assets {
        match asset.asset_type {
            AssetType::Css => {
                writeln!(
                    writer,
                    "<link rel=\"stylesheet\" href=\"{}{}\" type=\"text/css\">",
                    "../".repeat(dist),
                    asset.path.display()
                ).expect(&format!("failed to write asset \"{}\" into file", asset.path.display()));
            },

            AssetType::Other => {},
        }
    }
}

//  convert a markdown into html
fn convert<R, W>(reader: &mut BufReader<R>, writer: &mut BufWriter<W>, assets: &Vec<Asset>, dist: usize)
    where R: Read, W: Write
{
    let mut read_buffer = String::new();

    reader.read_to_string(&mut read_buffer).unwrap();
    let parser = Parser::new(&read_buffer.as_str());

    let mut write_buffer = String::new();

    html::push_html(&mut write_buffer, parser);
    writer.write(b"<!DOCTYPE html>\n\
<html>\n\
<head>\n\
<meta charset=\"UTF-8\">\n\
<title>Title</title>\n").unwrap();
    {
        write_assets(writer, assets, dist);
    }
    writer.write(b"</head>\n\
<body>\n").unwrap();
    writer.write(write_buffer.as_bytes()).unwrap();
    writer.write(b"</body>\n\
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

    let mut input_dir = match matches.free.is_empty() {
        true => {
            print_usage(program, &opts);
            return Err(());
        },

        false => PathBuf::from(matches.free[0].clone()),
    };

    let mut output_dir = match matches.opt_str("o") {
        Some(s) => PathBuf::from(s),
        None => input_dir.clone(),
    };

    let pwd = env::current_dir().unwrap();
    if input_dir.is_relative() {
        input_dir = pwd.join(input_dir);
    }
    if output_dir.is_relative() {
        output_dir = pwd.join(output_dir);
    }

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

    //  assets
    let assets: Vec<Asset> = vec![
        make_asset("assets/normalize.css", AssetType::Css),
        make_asset("assets/skeleton.css", AssetType::Css),
    ];

    prepare_assets(&info, &assets);
    convert_dir(&info.input_dir, &info.output_dir, info.verbose, &assets, 0);
}
