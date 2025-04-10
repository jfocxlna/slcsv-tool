use std::{
    error::Error, fmt, fs::File, io::{BufRead, BufReader, BufWriter, Write}, path::Path
};

use regex::Regex;

#[derive(Debug, Clone)]
struct NotAMultipleError;

impl fmt::Display for NotAMultipleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Original sampling is not a multiple of final sampling")
    }
}

impl Error for NotAMultipleError {}

#[derive(Debug, Clone)]
struct FinalGreaterOrEqualThanOriginalError;

impl fmt::Display for FinalGreaterOrEqualThanOriginalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "New sampling is equal or greater than the original sampling")
    }
}

impl Error for FinalGreaterOrEqualThanOriginalError {}

#[derive(Debug, Clone)]
struct FormatError;

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wrong format inserted")
    }
}

impl Error for FormatError {}

/// generates a single line csv with ramps.
/// # Formatting
/** 
 * $n$ means hold last value for n samples. Start value is always zero. If n is zero, the command will be skipped
 * /value,n/ means reach maxValue in n samples. It detects automatically if it nees to descend or not. The ramp includes maxValue. If n is zero, the command will be skipped
 * + as delimiter between commands
 */
/// # Format examples
/**
 * $10$+/100,100/+/50,10/+$30$ means hold 0 for 10 samples, then reach 100 in 100 samples from 0, then reach 50 in 10 samples from the last value, then hold 50 for 30 samples
 */
/// # Doubles, u64 and loss of precision
/// With really big values or really small doubles becomes imprecise and conversions fail to work. Expect issues with extreme values.
/// In some cases, due to precision issues, it might happen that the top value of the ramp can be ever so slighty higher (or lower) than the expected value.
/// The next written value will be the desired one, though.
pub fn single_line_gencsv_with_ramps<P: AsRef<Path>>(filepath: P, commands: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(filepath)?;
    let mut writer = BufWriter::new(file);
    let regex_fixed = Regex::new("^\\$[0-9]+\\$$")?;
    let regex_ramp = Regex::new("^\\/[+-]?([0-9]+[.])?[0-9]+,[0-9]+\\/$")?;
    let mut iterator = commands.split("+");
    let mut last_value: f64 = 0.0;
    while let Some(parsed) = iterator.next() {
        if regex_fixed.captures(parsed).is_some() {
            let num = parsed.replace("$", "");
            let samples = num.parse::<u64>()?;
            for _ in 0..samples {
                writer.write(last_value.to_string().as_bytes())?;
                writer.write(&[b'\n'])?;
            }
        } else if regex_ramp.captures(parsed).is_some() {
            let nums = parsed.replace("/", "");
            let mut nums_it: std::str::Split<'_, &str> = nums.split(",");
            let target = nums_it.next().unwrap().parse::<f64>()?;
            let samples = nums_it.next().unwrap().parse::<u64>()?;
            
            let increment = (target - last_value) / samples as f64;

            for _ in 0..samples {
                last_value += increment;
                writer.write(last_value.to_string().as_bytes())?;
                writer.write(&[b'\n'])?;
            }
            last_value = target;

        } else {
            return Err(Box::new(FormatError));
        }
    }

    Ok(())
}

/// Sampling conversion function.
/// Note that the final sampling must be a multiple of original sampling.
/// sampling intervals MUST be in ms.
pub fn sampling_conversion<P: AsRef<Path>>(original_sampling: u64, final_sampling: u64, original: P, new: P) -> Result<(), Box<dyn Error>> {
    let file = File::open(original)?;
    let newfile = File::create(new)?;
    let reader = BufReader::new(file);
    let mut writer = BufWriter::new(newfile);

    if final_sampling % original_sampling != 0 {
        return Err(Box::new(NotAMultipleError));
    }

    if final_sampling <= original_sampling {
        return Err(Box::new(FinalGreaterOrEqualThanOriginalError));
    }

    let increment = (final_sampling / original_sampling) as u64;

    let mut i: u64 = 0;

    for line in reader.lines() {
        let mut line = line?;
        line.push('\n');
        if i % increment == 0 {
            writer.write(line.as_bytes())?;
        }
        i += 1;
    }

    Ok(())
}

/// conversion from csv with . as decimal and , as delimiter to , as decimal and \t as delimiter.
pub fn csv_to_tsv<P: AsRef<Path>>(original: P, new: P) -> Result<(), Box<dyn Error>> {
    let file = File::open(original)?;
    let newfile = File::create(new)?;
    let reader = BufReader::new(file);
    let mut writer = BufWriter::new(newfile);

    for line in reader.lines() {
        let mut line = line?;
            line.push('\n');
            for ch in line.chars() {
                if ch == '.' {
                    writer.write(&[b','])?;
                } else  if ch == ',' {
                    writer.write(&[b'\t'])?;
                } else {
                    writer.write(&[ch as u8])?;
                }
            }
    }

    Ok(())
}

/// copies an arbitrary number of lines from a file to a new one, starting from the first line.
pub fn csv_truncate<P: AsRef<Path>>(original: P, new: P, keep: u64) -> Result<(), Box<dyn Error>> {
    let file = File::open(original)?;
    let newfile = File::create(new)?;
    let reader = BufReader::new(file);
    let mut writer = BufWriter::new(newfile);
    let mut c = 0;

    for line in reader.lines() {
        let mut line = line?;
        if c < keep {
            line.push_str("\n");
            writer.write(line.as_bytes())?;
            c += 1;
        } else {
            break;
        }
    }

    Ok(())
}