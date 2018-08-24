/*
 *  writer2
 *
 *  getting the ssg to work first
 *  because writing parsers for cmark takes forever
 *
 *  ideas for options:
 *  -   replace / don't replace existing files
 */

extern crate clap;

extern crate libc;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate newtype_derive;

#[macro_use]
extern crate custom_derive;

#[macro_use]
extern crate log;

extern crate simplelog;
use simplelog::*;

#[macro_use]
extern crate lazy_static;

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

mod converters;
use converters::Converter;
use converters::basic::{ BasicConverter, BasicData };
use converters::simple::SimpleConverter;

mod asset;
use asset::{ Asset, AssetType };

mod program_options;
use program_options::{ ProgramOptions, ProgramOptionsErr };

use std::env;
use std::path::PathBuf;

fn copy_assets(
    info: &ProgramOptions,
    input_dir: &PathBuf,
    src_dir: &PathBuf,
    dst_dir: &PathBuf
) -> Result<Vec<Asset>, ()> {
    match fs::create_dir_all(&dst_dir) {
        Ok(_) => {
            info!("Creating directory {}...", dst_dir.display());
        },

        Err(_) => {
            error!("Cannot create directory {}. Skipping this directory...", dst_dir.display());
            return Err(());
        },
    };

    let iter = match fs::read_dir(&src_dir) {
        Ok(res) => {
            info!("Iterating over files in directory {}...", src_dir.display());
            res
        },

        Err(_) => {
            error!("Cannot iterate over files in directory {}. Skipping this directory...", src_dir.display());
            return Err(());
        },
    };

    let mut res: Vec<Asset> = Vec::new();

    for entry in iter {
        let mut path = match entry {
            Ok(res) => res,
            Err(_) => {
                error!("Cannot iterate over file. Skipping this file...");
                continue;
            },
        }.path();

        let new_dst_path = dst_dir.join(match path.strip_prefix(&src_dir) {
            Ok(res) => res,
            Err(_) => {
                error!("Cannot strip prefix {} from {}. Skipping this file...", src_dir.display(), path.display());
                continue;
            },
        });

        match path.is_dir() {
            true => {
                info!("{} is a directory, going in...", path.display());
                
                match copy_assets(info, input_dir, &path, &new_dst_path) {
                    Ok(mut vec) => {
                        res.append(&mut vec);
                    },

                    Err(_) => {
                        if !info.persist {
                            error!("Failed to copy assets in inner directory {}. Falling back...", path.display());
                            return Err(());
                        }
                    },
                };
            },

            false => {
                match fs::copy(&path, &new_dst_path) {
                    Ok(_) => {
                        info!("Copying {} over to {}", path.display(), new_dst_path.display());
                    },

                    Err(_) => {
                        error!("Cannot copy {} over to {}. Skipping this file...", path.display(), new_dst_path.display());
                        continue;
                    },
                };

                let asset_type = AssetType::guess(&path);
                res.push(Asset::new(
                    match path.strip_prefix(&input_dir) {
                        Ok(res) => res,
                        Err(_) => {
                            error!("Cannot strip prefix {} from {}. Skipping this file...", input_dir.display(), path.display());
                            continue;
                        },
                    }.to_path_buf(),
                    asset_type
                ));

                match asset_type {
                    AssetType::Css => {
                        info!("Recognizing {} as a JavaScript asset", path.display());
                    },

                    AssetType::Js => {
                        info!("Recognizing {} as an CSS asset", path.display());
                    },

                    AssetType::Other => {
                        warn!("Not sure what kind of asset {} is. This asset is copied into the output directory, but will not be included in the <head> elements of the generated HTML files", path.display());
                    },
                };
            },
        }
    }

    Ok(res)
}

//  copy assets over to the output directory
fn prepare_assets(
    info: &ProgramOptions,
    assets_dir: &PathBuf
) -> Result<Vec<Asset>, ()> {
    let curr_dir = match env::current_dir() {
        Ok(res) => res,
        Err(_) => {
            error!("Cannot obtain current working directory");
            return Err(());
        },
    };

    let src_dir = curr_dir.join(assets_dir);
    let dst_dir = info.output_dir.join(assets_dir);

    copy_assets(&info, &curr_dir, &src_dir, &dst_dir)
}

//  convert all markdowns in a directory and copy whatever else
//  dist is the number of levels from the original output directory to output_dir
//  precondition: input_dir.is_dir() must be true
fn convert_dir(
    info: &ProgramOptions,
    src_dir: &PathBuf,
    dst_dir: &PathBuf,
    assets: &Vec<Asset>,
    dist: usize
) -> Result<(), ()> {
    match fs::create_dir_all(&dst_dir) {
        Ok(_) => {
            info!("Creating directory {}...", dst_dir.display());
        },

        Err(_) => {
            error!("Cannot create directory {}. Skipping this directory...", dst_dir.display());
            return Err(());
        },
    };

    let iter = match fs::read_dir(&src_dir) {
        Ok(res) => {
            info!("Iterating over files in directory {}...", src_dir.display());
            res
        },

        Err(_) => {
            error!("Cannot iterate over files in directory {}. Skipping this directory...", src_dir.display());
            return Err(());
        },
    };

    for entry in iter {
        let mut path = match entry {
            Ok(res) => res,
            Err(_) => {
                error!("Cannot iterate over file. Skipping this file..");
                continue;
            },
        }.path();

        let mut new_dst_path = dst_dir.join(match path.strip_prefix(&src_dir) {
            Ok(res) => res,
            Err(_) => {
                error!("Cannot strip prefix {} from {}. Skipping this file...", src_dir.display(), path.display());
                continue;
            },
        });

        match path.is_dir() {
            true => {
                info!("{} is a directory, going in...", path.display());

                match convert_dir(info, &path, &new_dst_path, assets, dist + 1) {
                    Ok(_) => (),
                    Err(_) => {
                        if !info.persist {
                            error!("Failed to convert files in inner directory {}. Falling back...", path.display());
                            return Err(());
                        }
                    },
                };
            },

            false => {
                let ext = match path.extension() {
                    Some(res) => res,
                    None => {
                        error!("Cannot extract extension from path {}. Skipping this file...", path.display());
                        continue;
                    },
                };

                match ext == "md" {
                    true => {
                        info!("{} is a markdown, converting to post", path.display());

                        let input = match fs::File::open(&path) {
                            Ok(res) => res,
                            Err(_) => {
                                error!("Cannot open file {}. Skipping this file...", path.display());
                                continue;
                            },
                        };
                        let mut reader = BufReader::new(input);

                        let mut html_path = new_dst_path.clone();
                        html_path.set_extension("html");

                        let output = match fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .create(true)
                            .open(html_path.clone()) {
                            Ok(res) => res,
                            Err(_) => {
                                error!("Cannot open file {} for writing. Skipping this file...", new_dst_path.display());
                                continue;
                            },
                        };
                        let mut writer = BufWriter::new(output);

                        match match info.simple {
                            true => {
                                let mut cvt = SimpleConverter::new();
                                cvt.convert(&mut reader, &mut writer, ())
                            },

                            false => {
                                let mut cvt = BasicConverter::new();
                                cvt.convert(&mut reader, &mut writer, BasicData::new(assets, dist))
                            },
                        } {
                            Ok(_) => {
                                info!("Markdown conversion successful.");
                            },

                            Err(_) => {
                                error!("Markdown conversion failed.");

                                if !info.persist {
                                    return Err(());
                                }
                            },
                        }
                    },

                    false => {
                        info!("{} is a non-markdown file, copying over", path.display());

                        match fs::copy(&path, &new_dst_path) {
                            Ok(_) => (),
                            Err(_) => {
                                error!("Cannot copy {}. Skipping this file...", path.display());
                                continue;
                            },
                        };
                    },
                }
            },
        }
    };

    Ok(())
}

fn convert(info: &ProgramOptions, assets: &Vec<Asset>) -> Result<(), ()> {
    convert_dir(&info, &info.input_dir, &info.output_dir, assets, 0)
}

fn main() {
    let info = match ProgramOptions::parse_options() {
        Ok(res) => res,

        Err(e) => {
            match SimpleLogger::init(LevelFilter::Error, Config::default()) {
                Ok(_) => (),
                Err(_) => {
                    println!("Pre-logging error: cannot initialize log. Terminating...");
                    return;
                },
            };

            match e {
                ProgramOptionsErr::MissingInputDirectory =>
                    error!("Input directory was not specified. Terminating..."),

                ProgramOptionsErr::BadVerbosity =>
                    error!("Verbosity must be <= 3. Terminating..."),
            };

            return;
        },
    };

    match SimpleLogger::init(
        match info.verbosity {
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            _ => LevelFilter::Error,
        },
        Config::default()
    ) {
        Ok(_) => (),
        Err(_) => {
            println!("Pre-logging error: cannot initialize log. Terminating...");
            return;
        },
    };

    let assets = match info.simple {
        true => {
            info!("Simple conversion, skipping assets...");
            Vec::new()
        },

        false => {
            match prepare_assets(&info, &PathBuf::from("assets")) {
                Ok(res) => {
                    info!("Assets copied successfully.");
                    res
                },

                Err(_) => {
                    error!("Failed to copy assets from input directory to output directory.");

                    if !info.persist {
                        error!("The program is non-persisting. Terminating...");
                        return;
                    }

                    Vec::new()
                },
            }
        },
    };

    match convert(&info, &assets) {
        Ok(_) => {
            info!("Files in input directory converted successfully.");
        },

        Err(_) => {
            if !info.persist {
                error!("The program is non-persisting. Terminating...");
                return;
            }
        },
    };
}
