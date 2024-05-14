use std::{fmt, str::FromStr};

use self::Move::*;
use crate::constants::*;
use crate::cubie::{self, Corner::*, CubieCube, Edge::*};
use crate::{decode_table, write_table};
use crate::{error::Error, facelet::Color};

/// Layer moves, Up, Right, Front, Down, Face, Back.
/// 
/// $ clockwise, $2 double, $3 counter-clockwise.
#[rustfmt::skip]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Move {
    U, U2, U3,
    R, R2, R3,
    F, F2, F3,
    D, D2, D3,
    L, L2, L3,
    B, B2, B3,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            U3 => write!(f, "U'"),
            D3 => write!(f, "D'"),
            R3 => write!(f, "R'"),
            L3 => write!(f, "L'"),
            F3 => write!(f, "F'"),
            B3 => write!(f, "B'"),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl FromStr for Move {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "R" => Ok(R),
            "R'" => Ok(R3),
            "R2" => Ok(R2),
            "L" => Ok(L),
            "L'" => Ok(L3),
            "L2" => Ok(L2),
            "U" => Ok(U),
            "U'" => Ok(U3),
            "U2" => Ok(U2),
            "D" => Ok(D),
            "D'" => Ok(D3),
            "D2" => Ok(D2),
            "F" => Ok(F),
            "F'" => Ok(F3),
            "F2" => Ok(F2),
            "B" => Ok(B),
            "B'" => Ok(B3),
            "B2" => Ok(B2),
            _ => Err(Error::InvalidScramble),
        }
    }
}

#[rustfmt::skip]
impl Move {
    pub fn is_inverse(&self, other: Move) -> bool {
        matches!(
            (&self, other),
            (U | U2 | U3, D | D2 | D3) 
            | (R | R2 | R3, L | L2 | L3) 
            | (F | F2 | F3, B | B2 | B3),
        )
    }

    pub fn is_same_layer(&self, other: Move) -> bool {
        matches!(
            (&self, other),
            (U | U2 | U3, U | U2 | U3)
            | (D | D2 | D3, D | D2 | D3)
            | (R | R2 | R3, R | R2 | R3)
            | (L | L2 | L3, L | L2 | L3)
            | (F | F2 | F3, F | F2 | F3)
            | (B | B2 | B3, B | B2 | B3)
        )
    }

    pub fn get_inverse(self) -> Self {
        match self {
            U => U3,
            U3 => U,
            D => D3,
            D3 => D,
            R => R3,
            R3 => R,
            L => L3,
            L3 => L,
            F => F3,
            F3 => F,
            B => B3,
            B3 => B,
            _ => self,
        }
    }
}

/// The basic six cube moves described by permutations and changes in orientation.
/// 
/// U_MOVE
pub const U_MOVE: CubieCube = CubieCube {
    cp: [UBR, URF, UFL, ULB, DFR, DLF, DBL, DRB],
    co: [0, 0, 0, 0, 0, 0, 0, 0],
    ep: [UB, UR, UF, UL, DR, DF, DL, DB, FR, FL, BL, BR],
    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

/// The basic six cube moves described by permutations and changes in orientation.
/// 
/// R_MOVE
pub const R_MOVE: CubieCube = CubieCube {
    cp: [DFR, UFL, ULB, URF, DRB, DLF, DBL, UBR], //permutation of the corners
    co: [2, 0, 0, 1, 1, 0, 0, 2],                 //changes of the orientations of the corners
    ep: [FR, UF, UL, UB, BR, DF, DL, DB, DR, FL, BL, UR], //permutation of the edges
    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],     //changes of the permutations of the edges
};

/// The basic six cube moves described by permutations and changes in orientation.
/// 
/// F_MOVE
pub const F_MOVE: CubieCube = CubieCube {
    cp: [UFL, DLF, ULB, UBR, URF, DFR, DBL, DRB],
    co: [1, 2, 0, 0, 2, 1, 0, 0],
    ep: [UR, FL, UL, UB, DR, FR, DL, DB, UF, DF, BL, BR],
    eo: [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0],
};

/// The basic six cube moves described by permutations and changes in orientation.
/// 
/// D_MOVE
pub const D_MOVE: CubieCube = CubieCube {
    cp: [URF, UFL, ULB, UBR, DLF, DBL, DRB, DFR],
    co: [0, 0, 0, 0, 0, 0, 0, 0],
    ep: [UR, UF, UL, UB, DF, DL, DB, DR, FR, FL, BL, BR],
    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

/// The basic six cube moves described by permutations and changes in orientation.
/// 
/// L_MOVE
pub const L_MOVE: CubieCube = CubieCube {
    cp: [URF, ULB, DBL, UBR, DFR, UFL, DLF, DRB],
    co: [0, 1, 2, 0, 0, 2, 1, 0],
    ep: [UR, UF, BL, UB, DR, DF, FL, DB, FR, UL, DL, BR],
    eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

/// The basic six cube moves described by permutations and changes in orientation.
/// 
/// B_MOVE
pub const B_MOVE: CubieCube = CubieCube {
    cp: [URF, UFL, UBR, DRB, DFR, DLF, ULB, DBL],
    co: [0, 0, 1, 2, 0, 0, 2, 1],
    ep: [UR, UF, UL, BR, DR, DF, DL, BL, FR, FL, UB, DB],
    eo: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
};

pub struct MoveTables {
    pub twist_move: Vec<u16>,
    pub flip_move: Vec<u16>,
    pub u_edges_move: Vec<u16>,
    pub d_edges_move: Vec<u16>,
    pub ud_edges_move: Vec<u16>,
    pub corners_move: Vec<u16>,
    pub slice_sorted_move: Vec<u16>,
}

impl MoveTables {
    pub fn new() -> Self {
        Self {
            twist_move: move_twist().unwrap(),
            flip_move: move_flip().unwrap(),
            u_edges_move: move_u_edges().unwrap(),
            d_edges_move: move_d_edges().unwrap(),
            ud_edges_move: move_ud_edges().unwrap(),
            corners_move: move_corners().unwrap(),
            slice_sorted_move: move_slice_sorted().unwrap(),
        }
    }
}

/// Move table for the twists of the corners.
/// 
/// The twist coordinate describes the 3^7 = 2187 possible orientations of the 8 corners
/// 
/// 0 <= twist < 2187 in phase 1, twist = 0 in phase 2
pub fn move_twist() -> Result<Vec<u16>, Error> {
    let mut a = CubieCube::default();
    let bmc = cubie::basic_move_cubes();
    let fname = "tables/move_twist";
    let move_twist_table = std::fs::read(&fname).unwrap_or("".into());
    let mut twist_move = vec![0; N_TWIST * N_MOVE];
    if move_twist_table.is_empty() {
        println!("Creating {} table...", fname);
        for i in 0..N_TWIST {
            a.set_twist(i as u16);
            for j in ALL_COLORS {
                // six faces U, R, F, D, L, B
                for k in 0..3 {
                    // three moves for each face, for example U, U2, U3 = U'
                    a.corner_multiply(bmc[j as usize]);
                    twist_move[N_MOVE * i + 3 * j as usize + k] = a.get_twist();
                }
                a.corner_multiply(bmc[j as usize]); // 4. move restores face
            }
        }
        write_table(fname, &twist_move)?;
    } else {
        // println!("Loading {} table...", fname);
        twist_move = decode_table(&move_twist_table)?;
    }
    Ok(twist_move)
}

/// Move table for the flip of the edges.
/// 
/// The flip coordinate describes the 2^11 = 2048 possible orientations of the 12 edges
/// 
/// 0 <= flip < 2048 in phase 1, flip = 0 in phase 2
pub fn move_flip() -> Result<Vec<u16>, Error> {
    let mut a = CubieCube::default();
    let bmc = cubie::basic_move_cubes();
    let fname = "tables/move_flip";
    let flip_move_table = std::fs::read(&fname).unwrap_or("".into());
    let mut flip_move = vec![0; N_FLIP * N_MOVE];
    if flip_move_table.is_empty() {
        println!("Creating {} table...", fname);
        for i in 0..N_FLIP {
            a.set_flip(i as u16);
            for j in ALL_COLORS {
                for k in 0..3 {
                    a.edge_multiply(bmc[j as usize]);
                    flip_move[N_MOVE * i + 3 * j as usize + k] = a.get_flip() as u16;
                }
                a.edge_multiply(bmc[j as usize]);
            }
        }
        write_table(fname, &flip_move)?;
    } else {
        // println!("Loading {} table...", fname);
        flip_move = decode_table(&flip_move_table)?;
    }
    Ok(flip_move)
}

/// Move table for the four UD-slice edges FR, FL, Bl and BR.
/// 
/// The slice_sorted coordinate describes the 12!/8! = 11880 possible positions of the FR, FL, BL and BR edges.
/// 
/// Though for phase 1 only the "unsorted" slice coordinate with Binomial(12,4) = 495 positions is relevant, using the
/// slice_sorted coordinate gives us the permutation of the FR, FL, BL and BR edges at the beginning of phase 2 for free.
/// 
/// 0 <= slice_sorted < 11880 in phase 1, 0 <= slice_sorted < 24 in phase 2, slice_sorted = 0 for solved cube
pub fn move_slice_sorted() -> Result<Vec<u16>, Error> {
    let mut a = CubieCube::default();
    let bmc = cubie::basic_move_cubes();
    let fname = "tables/move_slice_sorted";
    let slice_move_table = std::fs::read(&fname).unwrap_or("".into());
    let mut slice_move = vec![0; N_SLICE_SORTED * N_MOVE];
    if slice_move_table.is_empty() {
        println!("Creating {} table...", fname);
        for i in 0..N_SLICE_SORTED {
            if i % 200 == 0 {
                print!("");
            }
            a.set_slice_sorted(i as u16);
            for j in ALL_COLORS {
                for k in 0..3 {
                    a.edge_multiply(bmc[j as usize]);
                    slice_move[N_MOVE * i + 3 * j as usize + k] = a.get_slice_sorted() as u16;
                }
                a.edge_multiply(bmc[j as usize]);
            }
        }
        write_table(fname, &slice_move)?;
    } else {
        // println!("Loading {} table...", fname);
        slice_move = decode_table(&slice_move_table)?;
    }
    Ok(slice_move)
}

/// Move table for the u_edges coordinate for transition phase 1 -> phase 2
/// 
/// The u_edges coordinate describes the 12!/8! = 11880 possible positions of the UR, UF, UL and UB edges. It is needed at
/// the end of phase 1 to set up the coordinates of phase 2
/// 
/// 0 <= u_edges < 11880 in phase 1, 0 <= u_edges < 1680 in phase 2, u_edges = 1656 for solved cube.
pub fn move_u_edges() -> Result<Vec<u16>, Error> {
    let mut a = CubieCube::default();
    let bmc = cubie::basic_move_cubes();
    let fname = "tables/move_u_edges";
    let move_u_edges_table = std::fs::read(&fname).unwrap_or("".into());
    let mut u_edges_move = vec![0; N_SLICE_SORTED * N_MOVE];
    if move_u_edges_table.is_empty() {
        println!("Creating {} table...", fname);
        for i in 0..N_SLICE_SORTED {
            if i % 200 == 0 {
                print!(".");
            }
            a.set_u_edges(i as u16);
            for j in ALL_COLORS {
                for k in 0..3 {
                    a.edge_multiply(bmc[j as usize]);
                    u_edges_move[N_MOVE * i + 3 * j as usize + k] = a.get_u_edges() as u16;
                }
                a.edge_multiply(bmc[j as usize]);
            }
        }
        write_table(fname, &u_edges_move)?;
    } else {
        // println!("Loading {} table...", fname);
        u_edges_move = decode_table(&move_u_edges_table)?;
    }
    Ok(u_edges_move)
}

/// Move table for the d_edges coordinate for transition phase 1 -> phase 2
/// 
/// The d_edges coordinate describes the 12!/8! = 11880 possible positions of the DR, DF, DL and DB edges. It is needed at
/// the end of phase 1 to set up the coordinates of phase 2
/// 
/// 0 <= d_edges < 11880 in phase 1, 0 <= d_edges < 1680 in phase 2, d_edges = 0 for solved cube.
pub fn move_d_edges() -> Result<Vec<u16>, Error> {
    let mut a = CubieCube::default();
    let bmc = cubie::basic_move_cubes();
    let fname = "tables/move_d_edges";
    let move_d_edges_table = std::fs::read(&fname).unwrap_or("".into());
    let mut d_edges_move = vec![0; N_SLICE_SORTED * N_MOVE];
    if move_d_edges_table.is_empty() {
        println!("Creating {} table...", fname);
        for i in 0..N_SLICE_SORTED {
            if i % 200 == 0 {
                print!(".");
            }
            a.set_d_edges(i as u16);
            for j in ALL_COLORS {
                for k in 0..3 {
                    a.edge_multiply(bmc[j as usize]);
                    d_edges_move[N_MOVE * i + 3 * j as usize + k] = a.get_d_edges() as u16;
                }
                a.edge_multiply(bmc[j as usize]);
            }
        }
        write_table(fname, &d_edges_move)?;
    } else {
        // println!("Loading {} table...", fname);
        d_edges_move = decode_table(&move_d_edges_table)?;
    }
    Ok(d_edges_move)
}

/// Move table for the edges in the U-face and D-face.
/// 
/// The ud_edges coordinate describes the 40320 permutations of the edges UR, UF, UL, UB, DR, DF, DL and DB in phase 2
/// 
/// ud_edges undefined in phase 1, 0 <= ud_edges < 40320 in phase 2, ud_edges = 0 for solved cube.
pub fn move_ud_edges() -> Result<Vec<u16>, Error> {
    let mut a = CubieCube::default();
    let bmc = cubie::basic_move_cubes();
    let fname = "tables/move_ud_edges";
    let move_ud_edges_table = std::fs::read(&fname).unwrap_or("".into());
    let mut ud_edges_move = vec![0; N_UD_EDGES * N_MOVE];
    if move_ud_edges_table.is_empty() {
        println!("Creating {} table...", fname);
        for i in 0..N_UD_EDGES {
            if i % 600 == 0 {
                print!(".");
            }
            if (i + 1) % 48000 == 0 {
                println!();
            }
            a.set_ud_edges(i);
            for j in ALL_COLORS {
                for k in 0..3 {
                    a.edge_multiply(bmc[j as usize]);
                    // only R2, F2, L2 and B2 in phase 2
                    if ((j == Color::R) || (j == Color::F) || (j == Color::L) || (j == Color::B))
                        && k != 1
                    {
                        continue;
                    }
                    ud_edges_move[N_MOVE * i + 3 * j as usize + k] = a.get_ud_edges() as u16;
                }
                a.edge_multiply(bmc[j as usize]);
            }
        }
        write_table(fname, &ud_edges_move)?;
    } else {
        // println!("Loading {} table...", fname);
        ud_edges_move = decode_table(&move_ud_edges_table)?;
    }
    Ok(ud_edges_move)
}

/// Move table for the corners coordinate in phase 2
/// 
/// The corners coordinate describes the 8! = 40320 permutations of the corners.
/// 
/// 0 <= corners < 40320 defined but unused in phase 1, 0 <= corners < 40320 in phase 2, corners = 0 for solved cube
pub fn move_corners() -> Result<Vec<u16>, Error> {
    let mut a = CubieCube::default();
    let bmc = cubie::basic_move_cubes();
    let fname = "tables/move_corners";
    let move_corners_table = std::fs::read(&fname).unwrap_or("".into());
    let mut corners_move = vec![0; N_CORNERS * N_MOVE];
    if move_corners_table.is_empty() {
        println!("Creating {} table...", fname);
        for i in 0..N_CORNERS {
            if i % 200 == 0 {
                print!(".");
            }
            if (i + 1) % 16000 == 0 {
                println!();
            }
            a.set_corners(i as u16);
            for j in ALL_COLORS {
                for k in 0..3 {
                    a.corner_multiply(bmc[j as usize]);
                    corners_move[N_MOVE * i + 3 * j as usize + k] = a.get_corners() as u16;
                }
                a.corner_multiply(bmc[j as usize]);
            }
        }
        write_table(fname, &corners_move)?;
    } else {
        // println!("Loading {} table...", fname);
        corners_move = decode_table(&move_corners_table)?;
    }
    Ok(corners_move)
}

#[cfg(test)]
mod test {
    use crate::moves::*;

    #[test]
    fn test_move_twist() {
        let move_twist = move_twist().unwrap();
        assert_eq!(move_twist.len(), 39366);
        assert_eq!(move_twist[39365], 1995);
        assert_eq!(move_twist[3936], 142);
        assert_eq!(move_twist[393], 158);
        assert_eq!(move_twist[39], 1505);
        assert_eq!(move_twist[3], 1494);
    }

    #[test]
    fn test_move_flip() {
        let move_flip = move_flip().unwrap();
        assert_eq!(move_flip.len(), 36864);
        assert_eq!(move_flip[36863], 1910);
        assert_eq!(move_flip[3686], 204);
        assert_eq!(move_flip[368], 54);
        assert_eq!(move_flip[36], 2);
        assert_eq!(move_flip[3], 0);
    }

    #[test]
    fn test_move_slice_sorted() {
        let move_slice = move_slice_sorted().unwrap();
        assert_eq!(move_slice.len(), 213840);
        assert_eq!(move_slice[213839], 11687);
        assert_eq!(move_slice[21383], 2849);
        assert_eq!(move_slice[2138], 3490);
        assert_eq!(move_slice[213], 1914);
        assert_eq!(move_slice[2], 0);
    }

    #[test]
    fn test_move_u_edges() {
        let move_u_edges: Vec<u16> = move_u_edges().unwrap();
        assert_eq!(move_u_edges.len(), 213840);
        assert_eq!(move_u_edges[213839], 10967);
        assert_eq!(move_u_edges[21383], 1187);
        assert_eq!(move_u_edges[2138], 5260);
        assert_eq!(move_u_edges[213], 1769);
        assert_eq!(move_u_edges[21], 7921);
    }

    #[test]
    fn test_move_d_edges() {
        let move_d_edges = move_d_edges().unwrap();
        assert_eq!(move_d_edges.len(), 213840);
        assert_eq!(move_d_edges[213839], 10967);
        assert_eq!(move_d_edges[21383], 1187);
        assert_eq!(move_d_edges[2138], 5260);
        assert_eq!(move_d_edges[213], 1769);
        assert_eq!(move_d_edges[21], 7921);
    }

    #[test]
    fn test_move_ud_edges() {
        let move_ud_edges = move_ud_edges().unwrap();
        assert_eq!(move_ud_edges.len(), 725760);
        assert_eq!(move_ud_edges[725759], 0);
        assert_eq!(move_ud_edges[7275], 0);
        assert_eq!(move_ud_edges[725], 0);
        assert_eq!(move_ud_edges[72], 10);
        assert_eq!(move_ud_edges[7], 313);
    }

    #[test]
    fn test_move_corners() {
        let move_corners = move_corners().unwrap();
        assert_eq!(move_corners.len(), 725760);
        assert_eq!(move_corners[725759], 16668);
        assert_eq!(move_corners[7275], 27211);
        assert_eq!(move_corners[725], 22323);
        assert_eq!(move_corners[72], 10);
        assert_eq!(move_corners[7], 157);
    }
}
