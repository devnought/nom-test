use nom::{
    bytes::complete::{tag, take},
    IResult,
};
use std::{fs, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opts {
    pub input: PathBuf,
}

fn main() {
    let opt = Opts::from_args();
    let data = fs::read_to_string(&opt.input).unwrap_or_else(|_| {
        println!("Could not open file '{}'", &opt.input.display());
        std::process::exit(1);
    });

    println!("{:#?}", abcd_parser(&data));
}

fn abcd_parser(i: &str) -> IResult<&str, &str> {
    tag("abcd")(i) // will consume bytes if the input begins with "abcd"
}