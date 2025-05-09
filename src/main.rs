use clap::Parser;
use colorgram::extract;
use std::path::{PathBuf, absolute};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'i', long = "input")]
    input_file: PathBuf,

    #[arg(short = 'c', long = "colors", default_value_t = 10)]
    colors_amount: usize,
}

fn main() {
    let args = Args::parse();

    let input_path = absolute(&args.input_file).unwrap();
    let colors_amount = args.colors_amount;

    assert!(input_path.exists(), "Input file does not exist");
    assert!(input_path.is_file(), "Input path is not a file");
    assert!(colors_amount > 0, "Colors amount must be greater than zero");

    match extract(input_path, colors_amount) {
        Ok(colors) => {
            for color in colors {
                println!(
                    "RGB: ({}, {}, {}), Proportion: {:.2}%",
                    color.rgb.r,
                    color.rgb.g,
                    color.rgb.b,
                    color.proportion * 100.0
                );
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
