use std::str;

use csvgen::{csv_to_tsv, csv_truncate, sampling_conversion, single_line_gencsv_with_ramps};
use inquire::{Select, Text};

mod csvgen;

const HELP_STR: &str = "Available options:
\u{20DD} ramp -> generate single-line csv with linear ramps
\u{20DD} rsmp -> reduce amount of samples in a csv file and save the result in another one
\u{20DD} cnvl -> copy a csv and replace . and , with , and \t respectively
\u{20DD} trnc -> truncate a csv by specifying the amount of remaining samples from the start of the file
You can exit with Ctrl + D or Ctrl + C or Esc

Description of the format for the ramp command: 
\u{20DD} $n$ means hold last value for n samples. Start value is always zero. If n is zero, the command will be skipped
\u{20DD} /value,n/ means reach maxValue in n samples. It detects automatically if it nees to descend or not. The ramp includes maxValue. If n is zero, the command will be skipped
\u{20DD} + as delimiter between commands
Example: $10$+/100,100/+/50,10/+$30$ means hold 0 for 10 samples, then reach 100 in 100 samples from 0, then reach 50 in 10 samples from the last value, then hold 50 for 30 samples
";
const INPUT_CMD_STR: &str = ">>> ";

fn main() {

    let options = vec!["ramp", "rsmp", "cnvl", "trnc", "help"];

    while let Ok(command) = Select::new(INPUT_CMD_STR, options.clone()).prompt() {
        match command {
            "ramp" => ramp(),
            "rsmp" => rsmp(),
            "cnvl" => cnvl(),
            "trnc" => trnc(),
            "help" => println!("{HELP_STR}"),
            &_ => println!("Unrecognized command"),
        }
    }
}

fn ramp() {
    if let Ok(filename) = Text::new("Insert the path of the new file: ").prompt() {
        if let Ok(command) = Text::new("Insert the command as the expected format (check help in case from menu): ").prompt() {
            match single_line_gencsv_with_ramps(&filename, &command) {
                Ok(_) => println!("Done!"),
                Err(err) => eprintln!("{}", err),
            }
        }
    }
}

fn rsmp() {
    if let Ok(filename) = Text::new("Insert the path of the original file: ").prompt() {
        if let Ok(new_filename) = Text::new("Insert the path of the new file: ").prompt() {
            let mut got_sample = false;
            let mut og_sampling = 0;
            while !got_sample {
                if let Ok(ogsampling_str ) = Text::new("Insert the original sampling rate (positive integer with no decimals, in ms): ").prompt() {
                    if let Ok(sample_parse) = ogsampling_str.parse::<u64>() {
                        got_sample = true;
                        og_sampling = sample_parse;
                    }
                } else {
                    return;
                }
            }
            got_sample = false;
            let mut new_sampling = 0;
            while !got_sample {
                if let Ok(newsampling_str ) = Text::new("Insert the new sampling rate (positive integer with no decimals, in ms): ").prompt() {
                    if let Ok(sample_parse) = newsampling_str.parse::<u64>() {
                        got_sample = true;
                        new_sampling = sample_parse;
                    }
                } else {
                    return;
                }
            }
            match sampling_conversion(og_sampling, new_sampling, &filename, &new_filename) {
                Ok(_) => println!("Done!"),
                Err(err) => eprintln!("{}", err),
            }
        }
    }
}

fn cnvl() {
    if let Ok(filename) = Text::new("Insert the path of the original file: ").prompt() {
        if let Ok(new_filename) = Text::new("Insert the path of the new file: ").prompt() {
            match csv_to_tsv(&filename, &new_filename) {
                Ok(_) => println!("Done!"),
                Err(err) => eprintln!("{}", err),
            }
        }
    }
}

fn trnc() {
    if let Ok(filename) = Text::new("Insert the path of the original file: ").prompt() {
        if let Ok(new_filename) = Text::new("Insert the path of the new file: ").prompt() {
            let mut got_keep = false;
            let mut keep = 0;
            while !got_keep {
                if let Ok(keep_str ) = Text::new("Insert how many samples you want to keep from the start of the file (positive integer with no decimals, in ms): ").prompt() {
                    if let Ok(keep_parse) = keep_str.parse::<u64>() {
                        got_keep = true;
                        keep = keep_parse;
                    }
                } else {
                    return;
                }
            }
            
            match csv_truncate(&filename, &new_filename, keep) {
                Ok(_) => println!("Done!"),
                Err(err) => eprintln!("{}", err),
            }
        }
    }
}
