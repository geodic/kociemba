use crate::constants::*;
use crate::cubie::CubieCube;
use crate::moves::{self, Move, MoveTables};
use crate::symmetries::{self, SymmetriesTables};
use crate::{decode_table, write_table};

/// The pruning tables cut the search tree during the search.
/// 
/// The pruning values are stored modulo 3 which saves a lot of memory.
pub struct PrunningTables {
    pub flipslice_twist_depth3: Vec<u32>,
    pub corners_ud_edges_depth3: Vec<u32>,
    pub cornslice_depth: Vec<u16>,
    /// array distance computes the new distance from the old_distance i and the new_distance_mod3 j.
    /// 
    /// We need this array because the pruning tables only store the distances mod 3
    pub distance: Vec<u16>,
}

impl Default for PrunningTables {
    fn default() -> Self {
        let mut distance = vec![0; 60];
        for i in 0..20 {
            for j in 0..3 {
                distance[3 * i + j] = ((i / 3) * 3 + j) as u16;
                if i % 3 == 2 && j == 0 {
                    distance[3 * i + j] += 3;
                } else if i % 3 == 0 && j == 2 {
                    if distance[3 * i + j] >= 3 {
                        distance[3 * i + j] -= 3;
                    }
                }
            }
        }
        Self {
            flipslice_twist_depth3: vec![0xffffffff; N_FLIPSLICE_CLASS * N_TWIST / 16 + 1],
            corners_ud_edges_depth3: vec![0xffffffff; N_CORNERS_CLASS * N_UD_EDGES / 16],
            cornslice_depth: vec![65535; N_CORNERS * N_PERM_4],
            distance: distance,
        }
    }
}

impl PrunningTables {
    /// functions to extract or set values in the pruning tables
    /// 
    /// get_flipslice_twist_depth3(ix) is *exactly* the number of moves % 3 to solve phase 1 of a cube with index ix
    pub fn get_flipslice_twist_depth3(&self, ix: usize) -> u32 {
        let mut y = self.flipslice_twist_depth3[ix / 16];
        y >>= (ix % 16) * 2;
        y & 3
    }

    /// corners_ud_edges_depth3(ix) is *at least* the number of moves % 3 to solve phase 2 of a cube with index ix
    pub fn get_corners_ud_edges_depth3(&self, ix: usize) -> u32 {
        let mut y = self.corners_ud_edges_depth3[ix / 16];
        y >>= (ix % 16) * 2;
        y & 3
    }

    pub fn set_flipslice_twist_depth3(&mut self, ix: usize, value: u32) {
        let shift = (ix % 16) * 2;
        let base = ix >> 4;
        self.flipslice_twist_depth3[base] &= !(3 << shift) & 0xffffffff;
        self.flipslice_twist_depth3[base] |= value << shift;
    }

    pub fn set_corners_ud_edges_depth3(&mut self, ix: usize, value: u32) {
        let shift = (ix % 16) * 2;
        let base = ix >> 4;
        self.corners_ud_edges_depth3[base] &= !(3 << shift) & 0xffffffff;
        self.corners_ud_edges_depth3[base] |= value << shift;
    }

    /// Create/load the flipslice_twist_depth3 pruning table for phase 1.
    pub fn create_phase1_prun_table(&mut self, sy: &SymmetriesTables, mv: &MoveTables) {
        let total: usize = N_FLIPSLICE_CLASS * N_TWIST;
        let fname = "tables/phase1_prun";
        let phase1_prun_table = std::fs::read(&fname).unwrap_or("".into());

        let flipslice_classidx = &sy.flipslice_classidx;
        let flipslice_sym = &sy.flipslice_sym;
        let flipslice_rep = &sy.flipslice_rep;

        let sc = &sy.sc;
        let inv_idx = &sy.inv_idx;
        let twist_conj = &sy.twist_conj;
        let twist_move = &mv.twist_move;
        let flip_move = &mv.flip_move;
        let slice_sorted_move = &mv.slice_sorted_move;

        if phase1_prun_table.is_empty() {
            println!("Creating {} table...", fname);
            println!("This may take half a few minutes or longer, depending on the hardware.");
            // create table with the symmetries of the flipslice classes
            let mut cc = CubieCube::default();
            let mut fs_sym = vec![0; N_FLIPSLICE_CLASS];

            for i in 0..N_FLIPSLICE_CLASS {
                if (i + 1) % 1000 == 0 {
                    print!(".");
                }
                let rep = flipslice_rep[i];
                cc.set_slice((rep as usize / N_FLIP) as u16);
                cc.set_flip(((rep as usize) % N_FLIP) as u16);

                for s in 0..N_SYM_D4H {
                    let mut ss = CubieCube {
                        cp: sc[s].cp,
                        co: sc[s].co,
                        ep: sc[s].ep,
                        eo: sc[s].eo,
                    }; // copy cube
                    ss.edge_multiply(cc); // s*cc
                    ss.edge_multiply(sc[inv_idx[s] as usize]); // s*cc*s^-1
                    if ss.get_slice() == (rep as usize / N_FLIP) as u16
                        && ss.get_flip() == (rep as usize % N_FLIP) as u16
                    {
                        fs_sym[i] |= 1 << s;
                    }
                }
            }
            println!();
            let fs_classidx = 0; // value for solved phase 1
            let mut twist = 0;
            self.set_flipslice_twist_depth3(N_TWIST * fs_classidx + twist, 0);
            let mut done = 1;
            let mut depth = 0;
            let mut backsearch = false;
            println!("Depth: {} done: {}/{}", depth, done, total);
            while done != total {
                let depth3 = depth % 3;
                if depth == 9 {
                    // backwards search is faster for depth >= 9
                    println!("flipping to backwards search...");
                    backsearch = true;
                }
                let mut mult = 1;
                if depth < 8 {
                    mult = 5; // controls the output a few lines below
                }
                let mut idx = 0;
                for fs_classidx in 0..N_FLIPSLICE_CLASS {
                    if (fs_classidx + 1) % (200 * mult) == 0 {
                        print!(".");
                    }
                    if (fs_classidx + 1) % (16000 * mult) == 0 {
                        println!();
                    }

                    twist = 0;
                    while twist < N_TWIST {
                        // if table entries are not populated, this is very fast:
                        if !backsearch
                            && idx % 16 == 0
                            && self.flipslice_twist_depth3[idx / 16] == 0xffffffff
                            && twist < N_TWIST - 16
                        {
                            twist += 16;
                            idx += 16;
                            continue;
                        }

                        let mat = match backsearch {
                            true => self.get_flipslice_twist_depth3(idx) == 3,
                            false => self.get_flipslice_twist_depth3(idx) == depth3,
                        };

                        if mat {
                            let flipslice = flipslice_rep[fs_classidx];
                            let flip = flipslice % 2048; // N_FLIP = 2048
                            let slice_ = flipslice >> 11; // N_FLIP

                            for m in ALL_MOVES {
                                let twist1 = twist_move[18 * twist + m as usize]; // N_MOVE = 18
                                let flip1 = flip_move[18 * flip as usize + m as usize];
                                let slice1 =
                                    slice_sorted_move[432 * slice_ as usize + m as usize] / 24; // N_PERM_4 = 24, 18*24 = 432
                                let flipslice1 = ((slice1 as usize) << 11) + flip1 as usize;
                                let fs1_classidx = flipslice_classidx[flipslice1];
                                let fs1_sym = flipslice_sym[flipslice1];
                                let twist1 =
                                    twist_conj[((twist1 as usize) << 4) + fs1_sym as usize];
                                let idx1 = 2187 * fs1_classidx as usize + twist1 as usize; // N_TWIST = 2187
                                if !backsearch {
                                    if self.get_flipslice_twist_depth3(idx1) == 3 {
                                        // entry not yet filled
                                        if idx1 == 136 {
                                            println!(
                                                "idx1 136, value: {}, depth, {}",
                                                (depth + 1) % 3,
                                                depth
                                            );
                                        }
                                        self.set_flipslice_twist_depth3(idx1, (depth + 1) % 3);
                                        done += 1;
                                        // symmetric position has eventually more than one representation
                                        let mut sym = fs_sym[fs1_classidx as usize];
                                        if sym != 1 {
                                            for k in 1..16 {
                                                sym >>= 1;
                                                if sym % 2 == 1 {
                                                    // let twist_conj = symmetries::conj_twist().unwrap();
                                                    let twist2 = twist_conj
                                                        [((twist1 as usize) << 4) + k as usize];
                                                    // fs2_classidx = fs1_classidx due to symmetry
                                                    let idx2 = 2187 * fs1_classidx as usize
                                                        + twist2 as usize;
                                                    if self
                                                        .get_flipslice_twist_depth3(idx2 as usize)
                                                        == 3
                                                    {
                                                        self.set_flipslice_twist_depth3(
                                                            idx2 as usize,
                                                            (depth + 1) % 3,
                                                        );
                                                        done += 1;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // backwards search
                                    if self.get_flipslice_twist_depth3(idx1) == depth3 {
                                        self.set_flipslice_twist_depth3(idx, (depth + 1) % 3);
                                        done += 1;
                                        break;
                                    }
                                }
                            }
                        }
                        twist += 1;
                        idx += 1; // idx = N_TWIST * fs_class + twist
                    }
                }
                depth += 1;
                println!("Depth: {} done: {}/{}", depth, done, total);
            }
            write_table(fname, &self.flipslice_twist_depth3).unwrap();
        } else {
            println!("Loading {} table...", fname);
            self.flipslice_twist_depth3 = decode_table(&phase1_prun_table).unwrap();
        }
    }

    /// Create/load the corners_ud_edges_depth3 pruning table for phase 2.
    pub fn create_phase2_prun_table(&mut self, sy: &SymmetriesTables, mv: &MoveTables) {
        let total = N_CORNERS_CLASS * N_UD_EDGES;
        let fname = "tables/phase2_prun";
        let phase2_prun_table = std::fs::read(&fname).unwrap_or("".into());
        let mut fs_sym = vec![0; N_FLIPSLICE_CLASS];
        let corner_classidx = &sy.corner_classidx;
        let corner_sym = &sy.corner_sym;
        let corner_rep = &sy.corner_rep;
        let sc = &sy.sc;
        let inv_idx = sy.inv_idx;
        let ud_edges_conj = &sy.ud_edges_conj;
        let ud_edges_move = &mv.ud_edges_move;
        let corners_move = &mv.corners_move;
        if phase2_prun_table.is_empty() {
            println!("Creating {} table...", fname);
            // create table with the symmetries of the corners classes
            let mut cc = CubieCube::default();
            let mut c_sym = [0; N_CORNERS_CLASS];
            for i in 0..N_CORNERS_CLASS {
                if (i + 1) % 1000 == 0 {
                    print!(".");
                }
                let rep = corner_rep[i];
                cc.set_corners(rep);
                for s in 0..N_SYM_D4H {
                    let mut ss = CubieCube {
                        cp: sc[s].cp,
                        co: sc[s].co,
                        ep: sc[s].ep,
                        eo: sc[s].eo,
                    }; //  copy cube
                    ss.corner_multiply(cc); // s*cc
                    ss.corner_multiply(sc[inv_idx[s] as usize]); // s*cc*s^-1
                    if ss.get_corners() == rep {
                        c_sym[i] |= 1 << s;
                    }
                }
            }
            println!();

            let mut c_classidx = 0; // value for solved phase 2
            let mut ud_edge = 0;
            self.set_corners_ud_edges_depth3(N_UD_EDGES * c_classidx + ud_edge, 0);
            let mut done = 1;
            let mut depth = 0;
            println!("Depth: {} done: {}/{}", depth, done, total);
            while depth < 10 {
                //  we fill the table only do depth 9 + 1
                let depth3 = depth % 3;
                let mut idx = 0;
                let mut mult = 2;
                if depth > 9 {
                    mult = 1;
                }
                for c_classidx in 0..N_CORNERS_CLASS {
                    if (c_classidx + 1) % (20 * mult) == 0 {
                        print!("");
                    }
                    if (c_classidx + 1) % (1600 * mult) == 0 {
                        println!();
                    }

                    let mut ud_edge = 0;
                    while ud_edge < N_UD_EDGES {
                        // if table entries are not populated, this is very fast
                        if idx % 16 == 0
                            && self.corners_ud_edges_depth3[idx / 16] == 0xffffffff
                            && ud_edge < N_UD_EDGES - 16
                        {
                            ud_edge += 16;
                            idx += 16;
                            continue;
                        }

                        if self.get_corners_ud_edges_depth3(idx) == depth3 {
                            let corner = corner_rep[c_classidx];
                            // only iterate phase 2 moves
                            for m in [
                                Move::U,
                                Move::U2,
                                Move::U3,
                                Move::R2,
                                Move::F2,
                                Move::D,
                                Move::D2,
                                Move::D3,
                                Move::L2,
                                Move::B2,
                            ] {
                                let ud_edge1 = ud_edges_move[18 * ud_edge + m as usize];
                                let corner1 = corners_move[18 * corner as usize + m as usize];
                                let c1_classidx = corner_classidx[corner1 as usize];
                                let c1_sym = corner_sym[corner1 as usize];
                                let ud_edge1 =
                                    ud_edges_conj[((ud_edge1 as usize) << 4) + c1_sym as usize];
                                let idx1 = 40320 * c1_classidx as usize + ud_edge1 as usize; // N_UD_EDGES = 40320
                                if self.get_corners_ud_edges_depth3(idx1) == 3 {
                                    // entry not yet filled
                                    self.set_corners_ud_edges_depth3(idx1, (depth + 1) % 3); // depth + 1 <= 10
                                    done += 1;
                                    // symmetric position has eventually more than one representation
                                    let mut sym = c_sym[c1_classidx as usize];
                                    if sym != 1 {
                                        for k in 1..16 {
                                            sym >>= 1;
                                            if sym % 2 == 1 {
                                                let ud_edge2 =
                                                    ud_edges_conj[((ud_edge1 as usize) << 4) + k];
                                                // c1_classidx does not change
                                                let idx2 = 40320 * c1_classidx as usize
                                                    + ud_edge2 as usize;
                                                if self.get_corners_ud_edges_depth3(idx2) == 3 {
                                                    self.set_corners_ud_edges_depth3(
                                                        idx2,
                                                        (depth + 1) % 3,
                                                    );
                                                    done += 1;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        ud_edge += 1;
                        idx += 1; // idx = N_UD_EDGEPERM * corner_classidx + ud_edge
                    }
                }
                depth += 1;
                println!();
                println!("Depth: {} done: {}/{}", depth, done, total);
            }
            println!("remaining unfilled entries have depth >=11");
            write_table(fname, &self.corners_ud_edges_depth3).unwrap();
        } else {
            println!("Loading {} table...", fname);
            self.corners_ud_edges_depth3 = decode_table(&phase2_prun_table).unwrap();
        }
    }

    /// Create/load the cornslice_depth pruning table for phase 2. 
    /// 
    /// With this table we do a fast precheck at the beginning of phase 2.
    pub fn create_phase2_cornsliceprun_table(&mut self, mv: &MoveTables) {
        let fname = "tables/phase2_cornsliceprun";
        let phase2_cornsliceprun_table = std::fs::read(&fname).unwrap_or("".into());
        let corners_move = &mv.corners_move;
        let slice_sorted_move = &mv.slice_sorted_move;

        if phase2_cornsliceprun_table.is_empty() {
            println!("Creating {} table...", fname);
            let corners = 0; // values for solved phase 2
            let slice_ = 0;
            self.cornslice_depth[N_PERM_4 * corners + slice_] = 0;
            let mut done = 1;
            let mut depth = 0;
            while done != N_CORNERS * N_PERM_4 {
                for corners in 0..N_CORNERS {
                    for slice_ in 0..N_PERM_4 {
                        if self.cornslice_depth[N_PERM_4 * corners + slice_] == depth {
                            for m in [
                                Move::U,
                                Move::U2,
                                Move::U3,
                                Move::R2,
                                Move::F2,
                                Move::D,
                                Move::D2,
                                Move::D3,
                                Move::L2,
                                Move::B2,
                            ] {
                                let corners1 = corners_move[18 * corners + m as usize];
                                let slice_1 = slice_sorted_move[18 * slice_ + m as usize];
                                let idx1 = N_PERM_4 * corners1 as usize + slice_1 as usize;
                                if self.cornslice_depth[idx1] == 65535 {
                                    // entry not yet filled
                                    self.cornslice_depth[idx1] = depth + 1;
                                    done += 1;
                                    if done % 20000 == 0 {
                                        print!(".");
                                    }
                                }
                            }
                        }
                    }
                }
                depth += 1;
            }
            println!();
            write_table(fname, &self.cornslice_depth).unwrap();
        } else {
            println!("Loading {} table...", fname);
            self.cornslice_depth = decode_table(&phase2_cornsliceprun_table).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::pruning::*;

    // #[test]
    // fn test_flipslice_twist_depth3() {
    //     let mut pruningtable = PrunningTables::default();
    //     pruningtable.create_phase1_prun_table();

    //     let flipslice_twist_depth3 = pruningtable.flipslice_twist_depth3;
    //     assert_eq!(flipslice_twist_depth3.len(), 8806776);
    //     assert_eq!(flipslice_twist_depth3[0], 1704289684);
    //     assert_eq!(flipslice_twist_depth3[88], 136478754);
    //     assert_eq!(flipslice_twist_depth3[136], 2824101892);
    //     assert_eq!(flipslice_twist_depth3[271], 291852549);
    //     assert_eq!(flipslice_twist_depth3[8806], 341067092);
    //     assert_eq!(flipslice_twist_depth3[880677], 136971537);
    //     assert_eq!(flipslice_twist_depth3[8806775], 4294517857);
    // }

    // #[test]
    // fn test_corners_ud_edges_depth3() {
    //     let mut pruningtable = PrunningTables::default();
    //     pruningtable.create_phase2_prun_table();

    //     let corners_ud_edges_depth3 = pruningtable.corners_ud_edges_depth3;
    //     assert_eq!(corners_ud_edges_depth3.len(), 6975360);
    //     assert_eq!(corners_ud_edges_depth3[0], 1040187196);
    //     assert_eq!(corners_ud_edges_depth3[88], 3480960252);
    //     assert_eq!(corners_ud_edges_depth3[135], 4286013407);
    //     assert_eq!(corners_ud_edges_depth3[136], 4294834141);
    //     assert_eq!(corners_ud_edges_depth3[8806], 4294967295);
    //     assert_eq!(corners_ud_edges_depth3[880677], 4292870143);
    //     assert_eq!(corners_ud_edges_depth3[6975359], 2147483647);
    //     // println!(
    //     //     "{} {} {} {}",
    //     //     flipslice_twist_depth3[13600],
    //     //     flipslice_twist_depth3[13601],
    //     //     flipslice_twist_depth3[13602],
    //     //     flipslice_twist_depth3[136]
    //     // );
    // }

    // #[test]
    // fn test_cornslice_depth() {
    //     let mut pruningtable = PrunningTables::default();
    //     pruningtable.create_phase2_cornsliceprun_table();

    //     let cornslice_depth = pruningtable.cornslice_depth;
    //     assert_eq!(cornslice_depth.len(), 967680);
    //     assert_eq!(cornslice_depth[0], 0);
    //     assert_eq!(cornslice_depth[8], 8);
    //     assert_eq!(cornslice_depth[88], 12);
    //     assert_eq!(cornslice_depth[136], 11);
    //     assert_eq!(cornslice_depth[9676], 12);
    //     assert_eq!(cornslice_depth[96767], 12);
    //     assert_eq!(cornslice_depth[967679], 7);
    //     // println!(
    //     //     "{} {} {} {}",
    //     //     flipslice_twist_depth3[13600],
    //     //     flipslice_twist_depth3[13601],
    //     //     flipslice_twist_depth3[13602],
    //     //     flipslice_twist_depth3[136]
    //     // );
    // }
}
