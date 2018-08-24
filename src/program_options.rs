use clap::{ App, Arg };

use std::path::{ PathBuf };

/// Errors during options parsing.
pub enum ProgramOptionsErr {
    /// The user did not specify an input directory.
    MissingInputDirectory,

    /// The user specified a verbosity > 3.
    BadVerbosity,
}

/// Options passed to this program.
pub struct ProgramOptions {
    /// Path to the input directory. Files under this directory will be examined
    /// and copied / parsed.
    pub input_dir: PathBuf,

    /// Path to the output directory. Files will be copied / generated and
    /// written to this directory, following the same directory structure in the
    /// input directory.
    pub output_dir: PathBuf,

    /// Verbosity: 0 = error, 1 = warn, 2 = info.
    pub verbosity: usize,

    /// Persistence. If `false`, the program will stop running once it fails
    /// to convert / copy one file. If `true`, the program will attempt to
    /// run over all remaining files anyways. Defaults to `true`.
    pub persist: bool,

    /// Simple conversion: If `true`, the program will just do a simple
    /// CommonMark-to-HTML conversion, without theming and special syntax and
    /// such. If `false`, the program performs regular conversion. Defaults to
    /// `false`.
    pub simple: bool,
}

impl ProgramOptions {

    /// Parses the command-line arguments used to invoke this program.
    /// Returns `Ok(Self)` if parsing succeeds and all options are valid.
    /// Otherwise, returns `Err(())`.
    pub fn parse_options() -> Result<Self, ProgramOptionsErr> {
        let app = App::new("writer2")
            .about("A static site generator for a certain interests")
            .arg(Arg::with_name("input-dir")
                 .value_name("INPUT-DIR")
                 .help("sets the input directory")
                 .required(true))
            .arg(Arg::with_name("output-dir")
                 .short("o")
                 .long("output-dir")
                 .value_name("OUTPUT-DIR")
                 .help("sets the output directory")
                 .takes_value(true))
            .arg(Arg::with_name("help")
                 .short("h")
                 .long("help")
                 .help("display a help message"))
            .arg(Arg::with_name("verbosity")
                 .short("v")
                 .multiple(true)
                 .help("sets verbosity level, up to 3"))
            .arg(Arg::with_name("no-persist")
                 .long("no-persist")
                 .help("do not persist after error"))
            .arg(Arg::with_name("simple")
                 .long("simple")
                 .help("perform simple conversion"));

        let matches = app.get_matches();

        let input_dir = match matches.value_of("input-dir") {
            Some(s) => PathBuf::from(s),
            None => {
                return Err(ProgramOptionsErr::MissingInputDirectory);
            }
        };

        let output_dir = match matches.value_of("output-dir") {
            Some(s) => PathBuf::from(s),
            None => input_dir.clone(),
        };

        let verbosity = matches.occurrences_of("verbosity") as usize;
        if verbosity > 3 {
            return Err(ProgramOptionsErr::BadVerbosity);
        }

        Ok(Self {
            input_dir: input_dir,
            output_dir: output_dir,
            verbosity: verbosity,
            persist: !matches.is_present("no-persist"),
            simple: matches.is_present("simple"),
        })
    }

}
