use ansi_term::{Color::RGB, Style};
use clap::Parser;
use colorgram::extract;
use std::path::{PathBuf, absolute};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'i', long = "input", help = "Path to the image")]
    input_file: PathBuf,

    #[arg(
        short = 'c',
        long = "colors",
        default_value_t = 10,
        help = "Amount of colors to extract"
    )]
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
                let style = Style::new()
                    .bold()
                    .fg(RGB(255 - color.rgb.r, 255 - color.rgb.g, 255 - color.rgb.b))
                    .on(RGB(color.rgb.r, color.rgb.g, color.rgb.b));
                let proportion_string = format!("{:.2}%", color.proportion * 100.0);
                let final_string = format!("{:6} | {}", proportion_string, color.rgb);
                let output = style.paint(format!("{:1}{:28}", "", final_string));
                println!("{}", output);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
