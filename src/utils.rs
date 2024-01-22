use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use crate::{MatchesGameDirectory, SAVED_DIRECTORY};

pub const PANIC_OUTPUT_PATH: &str = "PANIC_OUTPUT (READ THIS IF THE PROGRAM CRASHED).txt";
pub type ArcMutex<T> = Arc<Mutex<T>>;
pub type ArcRwLock<T> = Arc<RwLock<T>>;

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

pub fn check_directory_match(directory: &PathBuf) -> MatchesGameDirectory {
    if directory.to_str().unwrap().contains("7 Days To Die") {
        MatchesGameDirectory::Match
    } else {
        MatchesGameDirectory::NoMatch
    }
}

pub fn load_saved_directory() -> Result<String, Box<dyn Error>> {
    let mut cache_file = File::open(SAVED_DIRECTORY)?;
    let mut directory_string = String::new();

    cache_file.read_to_string(&mut directory_string)?;

    Ok(directory_string)
}

#[macro_export]
macro_rules! make_arc_mutex {
    ($variable:expr, $var_type:ty) => {
        {
            let my: ArcMutex<$var_type> = Arc::new(Mutex::new($variable));
            my
        }
    };

    ($var:expr) => {
        {
            let arc = Arc::new(Mutex::new($var));
            arc
        }
    }
}

/// Allows you to create a named thread. Converts the &str passed into a String automatically.
/// I would suggest actually just passing in a &str just so nothing gets screwed up :)
#[macro_export]
macro_rules! make_named_thread {
    ($thread_name:expr) => {
        {
            let thread_builder: thread::Builder = thread::Builder::new().name($thread_name.to_string());
            thread_builder
        }
    };
}

#[macro_export]
macro_rules! arc_rwlock {
    ($var:expr) => {
        {
            let arc_rwlock = Arc::new(RwLock::new($var));
            arc_rwlock
        }
    };
}