use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const PANIC_OUTPUT_PATH: &str = "PANIC_OUTPUT (READ THIS IF THE PROGRAM CRASHED).txt";

pub fn write_to_panic_output(message: &str) -> Result<(), Box<dyn Error>> {
    let mut output = File::create(PANIC_OUTPUT_PATH)?;
    output.write(message.as_bytes())?;

    Ok(())
}

pub fn zips_temp_exists<F>(closure: F)
where
    F: Fn(&Path)
{
    let temp_path = Path::new(".zips_temp");
    if temp_path.exists() {
        closure(temp_path);
    }
}