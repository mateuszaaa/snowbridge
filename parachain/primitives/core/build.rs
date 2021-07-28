use std::env;
use std::fs;
use std::path::Path;
use hex::FromHex;


use std::fmt;
use std::error::Error;

/// Wraps `&'static str` and implements the `Error` trait for it.
#[derive(Debug)]
struct StringError {
    error: &'static str
}

impl StringError {
    fn new(error: &'static str) -> Self {
        return StringError { error }
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        self.error
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.error)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

