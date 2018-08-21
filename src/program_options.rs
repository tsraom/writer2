use getopts::Options;
use std::env;
use std::path::PathBuf;

/// program options.
pub struct ProgramOptions {
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub verbose: bool,
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("usage: {} INPUT-DIR [options]", program);
    print!("{}", opts.usage(&brief));
}

impl ProgramOptions {
    pub fn parse_options() -> Result<Self, ()> {
        let args: Vec<String> = env::args().collect();
        let program = &args[0];

        let mut opts = Options::new();
        opts.optopt("o", "output-dir", "set output directory", "OUTPUT-DIR");
        opts.optflag("h", "help", "print this help menu");
        opts.optflag("v", "verbose", "be very wordy");

        let matches = opts.parse(&args[1..])
            .unwrap_or_else(|f| panic!(f.to_string()));

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

        Ok(Self {
            input_dir: input_dir,
            output_dir: output_dir,
            verbose: matches.opt_present("v"),
        })
    }
}
