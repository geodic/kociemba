use std::str::FromStr;

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
    Ok(result)
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
        assert_eq!(scramble_to_str(&m).unwrap().trim(), "R U R' U' F L' D' B2 R' U'");
    }
}
