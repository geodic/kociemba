//! # kociemba
//! `kociemba`: crate for manipulating and solving the 3x3 Rubik's cube with [Kociemba's two phase algorithm](http://kociemba.org/cube.htm).

#[macro_use]
extern crate lazy_static;

/// Module containing functions for scrambling the cube.
pub mod scramble;

/// Error define.
pub mod error;

/// Module containing 3x3 cube constants.
pub mod constants;

/// Module for represent a cube on the coordinate level.
pub mod coord;

/// Module for represent a cube on the cubie level.
pub mod cubie;

/// Module for represent a cube on the facelet level.
pub mod facelet;

/// Module for create/load symmetries tables.
pub mod symmetries;

/// Module for represent move and create/load move tables.
pub mod moves;

/// Module for create/load pruning tables. The pruning tables cut the search tree during the search.
pub mod pruning;

/// Module for Solver.
pub mod solver;

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

fn write_table<P, T: Encode>(path: P, table: &T) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let encoded = encode_to_vec(table, CONFIG)?;
    fs::write(path, encoded)?;
    Ok(())
}

fn decode_table<T: Decode>(bytes: &[u8]) -> Result<T, Error> {
    let (decoded, written) = decode_from_slice(bytes, CONFIG)?;
    let additional = bytes.len() - written;

    if additional != 0 {
        return Err(DecodeError::UnexpectedEnd { additional })?;
    }
    Ok(decoded)
}
