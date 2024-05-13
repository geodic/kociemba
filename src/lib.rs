//! A crate for manipulating and solving the 3x3 Rubik's cube with [Kociemba's two phase algorithm](http://kociemba.org/cube.htm).

/// Module containing functions for scrambling the cube.

#[macro_use]
extern crate lazy_static;
pub mod scramble;

pub mod error;

/// Module containing 3x3 cube constants.
pub mod constants;
pub mod coord;
pub mod cubie;
pub mod facelet;
pub mod moves;
pub mod pruning;
pub mod solver;
pub mod symmetries;

use std::{fs, path::Path};

use bincode::{
    config::{self, Configuration},
    decode_from_slice, encode_to_vec,
    error::DecodeError,
    Decode, Encode,
};

use crate::error::Error;
use crate::{cubie::CubieCube, moves::Move};

const CONFIG: Configuration = config::standard();

pub fn write_table<P, T: Encode>(path: P, table: &T) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let encoded = encode_to_vec(table, CONFIG)?;
    fs::write(path, encoded)?;
    Ok(())
}

pub fn decode_table<T: Decode>(bytes: &[u8]) -> Result<T, Error> {
    let (decoded, written) = decode_from_slice(bytes, CONFIG)?;
    let additional = bytes.len() - written;

    if additional != 0 {
        return Err(DecodeError::UnexpectedEnd { additional })?;
    }
    Ok(decoded)
}
