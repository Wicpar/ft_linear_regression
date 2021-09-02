use std::fs::{OpenOptions};
use std::path::Path;
use std::io::{Read, Write};
use std::convert::TryInto;
use crate::args::{FileParser};
use std::error::Error;

pub struct ThetaFileArg;

impl FileParser<'_> for ThetaFileArg {
    const NAMES: &'static [&'static str] = &["-f", "--file"];
    const DESCRIPTION: &'static str = "The Theta variable file path";
}

fn read_theta(path: Option<&Path>) -> Result<(f64, f64), ()> {
    fn read(path: &Path) -> Result<(f64, f64), ()> {
        match OpenOptions::new().read(true).open(path) {
            Ok(mut file) => {
                let mut bytes: Vec<u8> = vec![];
                let _ = file.read_to_end(&mut bytes).map_err(|err| println!("Error: theta file {} could not be read: {}", path.display(), err))?;
                drop(file);
                if bytes.len() != 16 {
                    Err(println!("Error: theta file {} must be 16 bytes long: two big endian f64 values", path.display()))
                } else {
                    Ok((f64::from_be_bytes(bytes[0..8].try_into().unwrap()), f64::from_be_bytes(bytes[8..16].try_into().unwrap())))
                }
            }
            Err(err) => {
                Err(println!("Error: theta file {} could not be opened with read permission: {}", path.display(), err))
            }
        }
    }
    if let Some(path) = path {
        read(path)
    } else {
        let file_path = Path::new("./theta");
        if file_path.exists() {
            read(file_path)
        } else {
            Err(println!("Warning: default Theta file {} not found", file_path.display()))
        }
    }
}

pub fn get_theta(path: Option<&Path>) -> (f64, f64) {
    read_theta(path).unwrap_or_else(|_| {
        println!("Setting Theta to (0, 0) due to error");
        (0.0, 0.0)
    })
}

pub fn save_theta(path: Option<&Path>, theta: (f64, f64)) -> Result<(), ()> {
    fn write(path: &Path, theta: (f64, f64)) -> Result<(), ()> {
        let mut file =  OpenOptions::new().write(true).create(true).open(path)
            .map_err(|err| println!("Error: Theta file {} could not be opened or created for write: {}", path.display(), err))?;
        #[inline]
        fn chk(err: Result<(), impl Error>) -> Result<(), ()> {
            if let Err(err) = err {
                println!("Error: could not write theta to file: {}", err);
                Err(())
            } else {
                Ok(())
            }
        }
        chk(file.write_all(&theta.0.to_be_bytes()))?;
        chk(file.write_all(&theta.1.to_be_bytes()))
    }
    if let Some(file) = path {
        write(file, theta)
    } else {
        let file_path = Path::new("./theta");
        write(file_path, theta)
    }
}