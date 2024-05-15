use std::str::FromStr;
use rand::random;

use crate::cubie::CubieCube;
use crate::facelet::FaceCube;
use crate::{error::Error, Move};

pub fn scramble_from_str(s: &str) -> Result<Vec<Move>, Error> {
    s.split_whitespace()
        .map(|word| Move::from_str(word.trim()))
        .collect()
}

pub fn scramble_to_str(s: &Vec<Move>) -> Result<String, Error> {
    let result: String = s
        .iter()
        .map(|m| Move::to_string(m))
        .fold("".to_string(), |acc, x| format!("{} {}", acc, x));
    Ok(result.trim_start().to_string())
}

pub fn gen_scramble(length: usize) -> Result<Vec<Move>, Error> {
    let mut cc = CubieCube::default();
    cc.randomize();
    let fc = FaceCube::try_from(&cc)?;
    let mut ss = "".to_string();
    let mut ps = ' ';
    for s in fc.to_string().chars() {
        if s != ps {
            if ps == ' ' {
                ps = s;
                continue;
            }
            let suffix = match random::<u16>() % 3 {
                0 => "",
                1 => "2",
                _ => "'",
            };
            ss = ss + ps.to_string().as_str() + suffix.to_string().as_str() + " ";
        }
        ps = s;
    }
    let mut scramble = scramble_from_str(&ss)?;
    scramble.truncate(length);
    Ok(scramble)
}

#[cfg(test)]
mod test {
    use crate::moves::Move::*;
    use super::*;

    #[test]
    fn test_scramble_from_str() {
        let m = vec![R, U, R3, U3, F, L3, D3, B2, R3, U3];
        assert_eq!(scramble_from_str("R U R' U' F L' D' B2 R' U'").unwrap(), m);
    }

    #[test]
    fn test_scramble_to_str() {
        let m = vec![R, U, R3, U3, F, L3, D3, B2, R3, U3];
        assert_eq!(scramble_to_str(&m).unwrap(), "R U R' U' F L' D' B2 R' U'");
    }

    #[test]
    fn test_gen_scramble() {
        let ss = gen_scramble(25).unwrap();
        assert_eq!(ss.len(), 25);
    }
}
