use std::{fmt, usize};

use crate::constants::*;
use crate::cubie::Edge::*;
use crate::moves;
use crate::symmetries::SymmetriesTables;
use crate::symmetries;
use crate::{cubie::CubieCube, error::Error};
use crate::{decode_table, write_table};


/// Represent a cube on the coordinate level.
/// 
/// In phase 1 a state is uniquely determined by the three coordinates flip, twist and slice = slicesorted / 24.
/// 
/// In phase 2 a state is uniquely determined by the three coordinates corners, ud_edges and slice_sorted % 24.
/// 
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CoordCube {
    pub twist: u16,        // twist of corners
    pub flip: u16,         // flip of edges
    pub slice_sorted: u16, // Position of FR, FL, BL, BR edges. Valid in phase 1 (<11880) and phase 2 (<24)
    // The phase 1 slice coordinate is given by slice_sorted / 24
    pub u_edges: u16, // Valid in phase 1 (<11880) and phase 2 (<1680). 1656 is the index of solved u_edges.
    pub d_edges: u16, // Valid in phase 1 (<11880) and phase 2 (<1680)
    pub corners: u16, // corner permutation. Valid in phase1 and phase2
    pub ud_edges: u16, // permutation of the ud-edges. Valid only in phase 2
    pub flipslice_classidx: u16, // symmetry reduced flipslice coordinate used in phase 1
    pub flipslice_sym: u8,
    pub flipslice_rep: u32,
    pub corner_classidx: u16, // symmetry reduced corner permutation coordinate used in phase 2
    pub corner_sym: u8,
    pub corner_rep: u16,
}

impl Default for CoordCube {
    fn default() -> Self {
        Self {
            twist: 0,
            flip: 0,
            slice_sorted: 0,
            u_edges: 1656,
            d_edges: 0,
            corners: 0,
            ud_edges: 0,
            flipslice_classidx: 0,
            flipslice_sym: 0,
            flipslice_rep: 0,
            corner_classidx: 0,
            corner_sym: 0,
            corner_rep: 0,
        }
    }
}

impl fmt::Display for CoordCube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "{:?}", self)
        write!(f, "(twist: {}, flip: {}, slice: {}, U-edges: {}, D-edges: {}, E-edges: {}, Corners: {}, UD-Edges: {})\n{} {} {}\n{} {} {}",
        self.twist, self.flip, self.slice_sorted / 24, self.u_edges, self.d_edges, self.slice_sorted, self.corners, self.ud_edges,
        self.flipslice_classidx, self.flipslice_sym, self.flipslice_rep, self.corner_classidx, self.corner_sym, self.corner_rep)
    }
}

impl CoordCube {

    /// Build a CoordCube from CubieCube(cc).
    /// 
    /// Because `TryFrom(fn try_from)` can only take one argument, but we reference SymmetriesTables, so create this function.
    pub fn from_cubie(cc: &CubieCube, sy: &SymmetriesTables) -> Result<Self, Error> {
        if !cc.is_solvable() {
            return Err(Error::InvalidCubieValue);
        }
        // let cornersyms = symmetries::corner_syms().unwrap();

        let twist = cc.get_twist();
        let flip = cc.get_flip();
        let slice_sorted = cc.get_slice_sorted();
        let u_edges = cc.get_u_edges();
        let d_edges = cc.get_d_edges();
        let corners = cc.get_corners();
        let ud_edges;

        let flipslice_classidx =
            sy.flipslice_classidx[N_FLIP * (slice_sorted as usize / N_PERM_4) + flip as usize];
        let flipslice_sym =
            sy.flipslice_sym[N_FLIP * (slice_sorted as usize / N_PERM_4) + flip as usize];
        let flipslice_rep = sy.flipslice_rep[flipslice_classidx as usize];
        let corner_classidx = sy.corner_classidx[corners as usize];
        let corner_sym = sy.corner_sym[corners as usize];
        let corner_rep = sy.corner_rep[corner_classidx as usize];

        if slice_sorted < N_PERM_4 as u16 {
            // phase 2 cube
            ud_edges = cc.get_ud_edges();
        } else {
            ud_edges = 65535; // invalid
        }
        Ok(Self {
            twist: twist,
            flip: flip,
            slice_sorted: slice_sorted,
            u_edges: u_edges,
            d_edges: d_edges,
            corners: corners,
            ud_edges: ud_edges,
            flipslice_classidx: flipslice_classidx,
            flipslice_sym: flipslice_sym,
            flipslice_rep: flipslice_rep,
            corner_classidx: corner_classidx,
            corner_sym: corner_sym,
            corner_rep: corner_rep,
        })
    }

    /// Update phase 1 coordinates when move is apply.
    /// 
    /// :param m: The move
    pub fn phase1_move(&mut self, m: moves::Move) {
        let twist_move = moves::move_twist().unwrap();
        let flip_move = moves::move_flip().unwrap();
        let slice_sorted_move = moves::move_slice_sorted().unwrap();
        let u_edges_move = moves::move_u_edges().unwrap();
        let d_edges_move = moves::move_d_edges().unwrap();
        let corners_move = moves::move_corners().unwrap();
        let flipslicesyms = symmetries::flipslice_syms().unwrap();
        let flipslice_classidx = flipslicesyms.classidx;
        let flipslice_sym = flipslicesyms.sym;
        let flipslice_rep = flipslicesyms.rep;
        let cornersyms = symmetries::corner_syms().unwrap();
        let corner_classidx = cornersyms.classidx;
        let corner_sym = cornersyms.sym;
        let corner_rep = cornersyms.rep;

        self.twist = twist_move[N_MOVE * self.twist as usize + m as usize];
        self.flip = flip_move[N_MOVE * self.flip as usize + m as usize];
        self.slice_sorted = slice_sorted_move[N_MOVE * self.slice_sorted as usize + m as usize];
        // optional:
        self.u_edges = u_edges_move[N_MOVE * self.u_edges as usize + m as usize]; // u_edges and d_edges retrieve ud_edges easily
        self.d_edges = d_edges_move[N_MOVE * self.d_edges as usize + m as usize]; // if phase 1 is finished and phase 2 starts
        self.corners = corners_move[N_MOVE * self.corners as usize + m as usize]; // Is needed only in phase 2
        self.flipslice_classidx = flipslice_classidx
            [N_FLIP * (self.slice_sorted as usize / N_PERM_4) + self.flip as usize];
        self.flipslice_sym =
            flipslice_sym[N_FLIP * (self.slice_sorted as usize / N_PERM_4) + self.flip as usize];
        self.flipslice_rep = flipslice_rep[self.flipslice_classidx as usize];
        self.corner_classidx = corner_classidx[self.corners as usize];
        self.corner_sym = corner_sym[self.corners as usize];
        self.corner_rep = corner_rep[self.corner_classidx as usize];
    }

    /// Update phase 2 coordinates when move is apply.
    /// 
    /// :param m: The move
    pub fn phase2_move(&mut self, m: moves::Move) {
        let slice_sorted_move = moves::move_slice_sorted().unwrap();
        let corners_move = moves::move_corners().unwrap();
        let ud_edges_move = moves::move_ud_edges().unwrap();

        self.slice_sorted = slice_sorted_move[N_MOVE * self.slice_sorted as usize + m as usize];
        self.corners = corners_move[N_MOVE * self.corners as usize + m as usize];

        self.ud_edges = match self.ud_edges {
            65535 => ud_edges_move[N_UD_EDGES * N_MOVE + m as usize - N_MOVE],
            _ => ud_edges_move[N_MOVE * self.ud_edges as usize + m as usize],
        };
    }

}

/// EdgeMergeTables stores the initial phase 2 ud_edges coordinate from the u_edges and d_edges coordinates.
/// 
pub struct EdgeMergeTables {
    pub upd_ud_edges: Vec<u16>,
}

impl EdgeMergeTables {
    pub fn new() -> Self{
        Self {
            upd_ud_edges: create_phase2_edgemerge_table().unwrap(),
        }
    }
}

/// phase2_edgemerge retrieves the initial phase 2 ud_edges coordinate from the u_edges and d_edges coordinates.
fn create_phase2_edgemerge_table() -> Result<Vec<u16>, Error> {
    let fname = "tables/phase2_edgemerge";
    let mut u_edges_plus_d_edges_to_ud_edges: Vec<u16> = vec![0; N_U_EDGES_PHASE2 * N_PERM_4];
    let mut c_u = CubieCube::default();
    let mut c_d = CubieCube::default();
    let mut c_ud = CubieCube::default();
    let edge_u = [UR, UF, UL, UB];
    let edge_d = [DR, DF, DL, DB];
    let edge_ud = [UR, UF, UL, UB, DR, DF, DL, DB];

    let phase2_edgemerge_table = std::fs::read(&fname).unwrap_or("".into());
    if phase2_edgemerge_table.is_empty() {
        println!("Creating {} table...", fname);
        let mut cnt = 0;
        for i in 0..N_U_EDGES_PHASE2 {
            c_u.set_u_edges(i as u16);
            for j in 0..N_CHOOSE_8_4 {
                c_d.set_d_edges((j * N_PERM_4) as u16);
                let mut invalid = false;
                let mut c_ud_ep = [-1; 12];
                for ei in edge_ud {
                    let e = ei as usize;
                    c_ud_ep[e] = -1; // invalidate edges
                    if edge_u.contains(&c_u.ep[e]) {
                        c_ud.ep[e] = c_u.ep[e];
                        c_ud_ep[e] = c_u.ep[e] as i32;
                    }
                    if edge_d.contains(&c_d.ep[e]) {
                        c_ud.ep[e] = c_d.ep[e];
                        c_ud_ep[e] = c_d.ep[e] as i32;
                    }
                    if c_ud_ep[e] == -1 {
                        invalid = true; // edge collision
                        break;
                    }
                }
                if !invalid {
                    for k in 0..N_PERM_4 {
                        c_d.set_d_edges((j * N_PERM_4 + k) as u16);
                        for ei in edge_ud {
                            let e = ei as usize;
                            if edge_u.contains(&c_u.ep[e]) {
                                c_ud.ep[e] = c_u.ep[e];
                            }
                            if edge_d.contains(&c_d.ep[e]) {
                                c_ud.ep[e] = c_d.ep[e];
                            }
                        }
                        u_edges_plus_d_edges_to_ud_edges[N_PERM_4 * i + k] = c_ud.get_ud_edges();
                        cnt += 1;
                        if cnt % 2000 == 0 {
                            print!(".");
                        }
                    }
                }
            }
        }
        println!();
        write_table(fname, &u_edges_plus_d_edges_to_ud_edges)?;
        println!();
    } else {
        // println!("Loading {} table...", fname);
        u_edges_plus_d_edges_to_ud_edges = decode_table(&phase2_edgemerge_table)?;
    }
    Ok(u_edges_plus_d_edges_to_ud_edges)
}

#[cfg(test)]
mod test {
    use crate::coord::*;
    use crate::facelet::FaceCube;
    use crate::moves::Move;
    
    #[test]
    fn test_coordcube() {
        let sy = SymmetriesTables::new();
        let fc =
            FaceCube::try_from("RLLBUFUUUBDURRBBUBRLRRFDFDDLLLUDFLRRDDFRLFDBUBFFLBBDUF").unwrap();
        let cc = CubieCube::try_from(&fc).unwrap();
        let mut cdc = CoordCube::from_cubie(&cc, &sy).unwrap();
        assert_eq!(cdc.twist, 149);
        assert_eq!(cdc.flip, 1514);
        assert_eq!(cdc.slice_sorted, 1701);
        assert_eq!(cdc.u_edges, 407);
        assert_eq!(cdc.d_edges, 9068);
        assert_eq!(cdc.ud_edges, 65535);
        assert_eq!(cdc.corners, 3935);
        assert_eq!(cdc.flipslice_classidx, 1940);
        assert_eq!(cdc.flipslice_sym, 9);
        assert_eq!(cdc.flipslice_rep, 3802);
        assert_eq!(cdc.corner_classidx, 716);
        assert_eq!(cdc.corner_sym, 7);
        assert_eq!(cdc.corner_rep, 1260);
        cdc.phase1_move(Move::U2);
        assert_eq!(cdc.twist, 1229);
        assert_eq!(cdc.flip, 1898);
        assert_eq!(cdc.slice_sorted, 5061);
        assert_eq!(cdc.u_edges, 71);
        assert_eq!(cdc.d_edges, 9064);
        assert_eq!(cdc.ud_edges, 65535);
        assert_eq!(cdc.corners, 3876);
        assert_eq!(cdc.flipslice_classidx, 3220);
        assert_eq!(cdc.flipslice_sym, 13);
        assert_eq!(cdc.flipslice_rep, 7130);
        assert_eq!(cdc.corner_classidx, 1321);
        assert_eq!(cdc.corner_sym, 7);
        assert_eq!(cdc.corner_rep, 2459);
        cdc.phase2_move(Move::R2);
        assert_eq!(cdc.twist, 1229);
        assert_eq!(cdc.flip, 1898);
        assert_eq!(cdc.slice_sorted, 5116);
        assert_eq!(cdc.u_edges, 71);
        assert_eq!(cdc.d_edges, 9064);
        assert_eq!(cdc.ud_edges, 37019);
        assert_eq!(cdc.corners, 7596);
        assert_eq!(cdc.flipslice_classidx, 3220);
        assert_eq!(cdc.flipslice_sym, 13);
        assert_eq!(cdc.flipslice_rep, 7130);
        assert_eq!(cdc.corner_classidx, 1321);
        assert_eq!(cdc.corner_sym, 7);
        assert_eq!(cdc.corner_rep, 2459);
    }
    
    #[test]
    fn test_create_phase2_edgemerge_table() {
        let ud_edges = create_phase2_edgemerge_table().unwrap();
        assert_eq!(ud_edges.len(), 40320);
        assert_eq!(ud_edges[4], 24504);
        assert_eq!(ud_edges[40], 11521);
        assert_eq!(ud_edges[403], 15256);
        assert_eq!(ud_edges[4031], 23963);
        assert_eq!(ud_edges[40319], 39767);
    }
}
