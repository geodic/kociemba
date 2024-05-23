use std::cmp::{max, min};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

use crate::constants::*;
use crate::coord::{self, CoordCube, EdgeMergeTables};
use crate::cubie::CubieCube;
use crate::error::Error;
use crate::facelet::FaceCube;
use crate::moves::Move;
use crate::moves::{self, MoveTables};
use crate::pruning::PrunningTables;
use crate::symmetries::SymmetriesTables;
use crate::{pruning, symmetries};

/// All data tables.
/// 
/// * `sy`: [SymmetriesTables]
/// * `mv`: [MoveTables]
/// * `pr`: [PrunningTables]
/// * `em`: [EdgeMergeTables]
pub struct SolverTables {
    sy: SymmetriesTables,
    mv: MoveTables,
    pr: PrunningTables,
    em: EdgeMergeTables,
}

impl SolverTables {
    fn new() -> Self {
        let sy = symmetries::SymmetriesTables::new();
        let mv = moves::MoveTables::new();
        let mut pr = pruning::PrunningTables::default();
        let _ = pr.create_phase1_prun_table(&sy, &mv);
        let _ = pr.create_phase2_prun_table(&sy, &mv);
        let _ = pr.create_phase2_cornsliceprun_table(&mv);
        let em = coord::EdgeMergeTables::new();
        Self {
            sy: sy,
            mv: mv,
            pr: pr,
            em: em,
        }
    }
}

/// Solution result:
/// * solution: a Move vector.
/// * solve_time: time to get solution(not include load data tables time.).
#[derive (Debug, Serialize, Deserialize, Clone)]
pub struct SoutionResult {
    pub solution: Vec<Move>,
    pub solve_time: Duration,
}

impl Default for SoutionResult {
    fn default() -> Self {
        Self {
            solution: Vec::new(),
            solve_time: Duration::from_secs(0),
        }
    }
}

/// Solve a cube defined by cubstring to a position defined by goalstring.
/// 
/// # Parameters
/// * `cubestring`: The format of the string is given in the Facelet class defined.
/// * `goalstring`: The format of the string is given in the Facelet class defined.
/// * `max_length`: The function will return if a maneuver of length <= max_length has been found
/// * `time_out`: If the function times out, the best solution found so far is returned. If there has not been found
/// any solution yet the computation continues until a first solution appears.
///
/// # Examples
/// ```rust
/// use kociemba::solver::solver;
///
/// fn main() {
///     let result = solver(
///         "RLLBUFUUUBDURRBBUBRLRRFDFDDLLLUDFLRRDDFRLFDBUBFFLBBDUF",
///         "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
///         20,
///         3.0,
///         ).unwrap();
///     println!("{:?}, ({}), ({:?})", result.solution, result.solution.len(), result.solve_time);
/// }
/// ```

pub fn solver(
    cubestring: &str,
    goalstring: &str,
    max_length: usize,
    time_out: f32,
) -> Result<SoutionResult, Error> {
    lazy_static! {
        static ref SOLVERTABLES: SolverTables = SolverTables::new();
    }
    for i in 0..2 {
        let facestr;
        let maxlength;
        let timeout;
        let mut cc;
        if i == 0 {
            facestr = "RLLBUFUUUBDURRBBUBRLRRFDFDDLLLUDFLRRDDFRLFDBUBFFLBBDUF";
            maxlength = 25;
            timeout = 3.;
            let fc = FaceCube::try_from(facestr)?;
            cc = CubieCube::try_from(&fc)?;
        } else {
            facestr = &cubestring;
            maxlength = max_length;
            timeout = time_out;
            let fc0 = FaceCube::try_from(facestr)?;
            let fcg = FaceCube::try_from(goalstring)?;
            let cc0 = CubieCube::try_from(&fc0)?;
            let s = cc0.verify()?;
            if s != true {
                return Err(Error::InvalidFaceletString); // no valid facelet cube, gives invalid cubie cube
            }
            let ccg = CubieCube::try_from(&fcg)?;
            let s = ccg.verify()?;
            if s != true {
                return Err(Error::InvalidFaceletString); // no valid facelet cube, gives invalid cubie cube
            }
            // cc0 * S = ccg  <=> (ccg^-1 * cc0) * S = Id
            cc = ccg.inverse_cubie_cube();
            cc.multiply(cc0);
        }
    
        let start_time = Instant::now();
        let syms = cc.symmetries();
        let v: HashSet<usize> = HashSet::from([16, 20, 24, 28]);
        let symsset: HashSet<usize> = HashSet::from_iter(syms.into_iter());
        let ins: Vec<&usize> = v.intersection(&symsset).collect();
        let mut tr = match ins.len() > 0 {
            // we have some rotational symmetry along a long diagonal
            true => vec![0, 3], // so we search only one direction and the inverse
            false => (0..6).collect(), // This means search in 3 directions + inverse cube
        };
        let vv: HashSet<usize> = HashSet::from_iter(48..96);
        let ins: Vec<&usize> = vv.intersection(&symsset).collect();
        if ins.len() > 0 {
            // we have some antisymmetry so we do not search the inverses
            tr = tr.into_iter().filter(|x| *x < 3).collect()
        }
        let mut solverthreads = vec![];

        // these mutable variables are modidified by all six threads
        let solutions = Arc::new(Mutex::new(vec![Vec::<Move>::new()]));
        let terminated = Arc::new(Mutex::new(false));

        for i in tr {
            let solutions = Arc::clone(&solutions);
            let terminated = Arc::clone(&terminated);

            let mut sth = SolverThread::new(
                cc,
                i % 3,
                i / 3,
                maxlength,
                timeout,
                start_time,
                solutions,
                terminated,
                vec![999],
                &SOLVERTABLES,
            );

            let handle = thread::spawn(move || {
                sth.start();
            });
            solverthreads.push(handle);
        }
        for handle in solverthreads {
            handle.join().unwrap();
        }

        if i == 1 {
            let solutions = solutions.lock().unwrap();
            if (*solutions).len() > 1 {
                let end_time = Instant::now();
                let ls = (*solutions).last().unwrap();
                // println!("{:?} ({}), {:?}", *ls, (*ls).len(), end_time - start_time)
                return Ok(SoutionResult {
                    solution: ls.to_vec(),
                    solve_time: end_time - start_time,
                });
            }
        }
    }
    Ok(SoutionResult::default())
}

/// Solve a cube defined by its cube definition string.
/// # Parameters
/// * `cubestring`: The format of the string is given in the Facelet class defined.
/// * `max_length`: The function will return if a maneuver of length <= max_length has been found
/// * `time_out`: If the function times out, the best solution found so far is returned. If there has not been found
///     any solution yet the computation continues until a first solution appears.
/// # Examples
/// ```rust
/// use kociemba::solver::solve;
///
/// fn main() {
///     let result = solve(
///         "RLLBUFUUUBDURRBBUBRLRRFDFDDLLLUDFLRRDDFRLFDBUBFFLBBDUF",
///         20,
///         3.0,).unwrap();
///     println!("{:?}, ({}), ({:?})", result.solution, result.solution.len(), result.solve_time);
/// }
/// ```

pub fn solve(cubestring: &str, max_length: usize, timeout: f32) -> Result<SoutionResult, Error> {
    let goalstring = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    solver(cubestring, goalstring, max_length, timeout)
}


/** The SolverThread class solves implements the two phase algorithm.

* `cb_cube`: The cube to be solved in CubieCube representation
* `rot`: Rotates the  cube 120° * rot along the long diagonal before applying the two-phase-algorithm
* `inv`: 0: Do not invert the cube . 1: Invert the cube before applying the two-phase-algorithm
* `ret_length`: If a solution with length <= ret_length is found the search stops.
 The most efficient way to solve a cube is to start six threads in parallel with rot = 0, 1 and 2 and
 inv = 0, 1. The first thread which finds a solutions sets the terminated flag which signals all other threads
 to teminate. On average this solves a cube about 12 times faster than solving one cube with a single thread.
* `timeout`: Essentially the maximal search time in seconds. Essentially because the search does not return
 before at least one solution has been found.
* `start_time`: The time the search started.
* `solutions`: An array with the found solutions found by the six parallel threads
* `terminated`: An event shared by the six threads to signal a termination request
* `shortest_length`: The length of the shortest solutions in the solution array
* `solvertables`: The reference to [SolverTables].
*/
pub struct SolverThread<'a> {
    cb_cube: CubieCube,
    co_cube: CoordCube, // CoordCube initialized in function run
    rot: u8,
    inv: u8,
    sofar_phase1: Vec<Move>,
    sofar_phase2: Vec<Move>,
    phase2_done: bool,
    ret_length: usize,
    timeout: f32,
    start_time: Instant,
    cornersave: u16,
    // these variables are shared by the six threads, initialized in function solve
    solutions: Arc<Mutex<Vec<Vec<Move>>>>,
    terminated: Arc<Mutex<bool>>,
    shortest_length: Vec<usize>,
    solvertables: &'a SolverTables,
}

impl<'a> SolverThread<'a> {
    pub fn new(
        cb_cube: CubieCube,
        rot: u8,
        inv: u8,
        ret_length: usize,
        timeout: f32,
        start_time: Instant,
        solutions: Arc<Mutex<Vec<Vec<Move>>>>,
        terminated: Arc<Mutex<bool>>,
        shortest_length: Vec<usize>,
        solvertables: &'a SolverTables,
    ) -> Self {
        let co_cube = CoordCube::default();
        Self {
            cb_cube: cb_cube,
            co_cube: co_cube,
            rot: rot,
            inv: inv,
            sofar_phase1: Vec::new(),
            sofar_phase2: Vec::new(),
            phase2_done: false,
            ret_length: ret_length,
            timeout: timeout,
            start_time: start_time,
            cornersave: 0,
            solutions: solutions,
            terminated: terminated,
            shortest_length: shortest_length,
            solvertables: solvertables,
        }
    }

    /// Compute the distance to the cube subgroup H where flip=slice=twist=0
    ///
    /// return: The distance to H
    fn get_depth_phase1(&self) -> u32 {
        let mut slice_ = self.co_cube.slice_sorted / N_PERM_4 as u16;
        let mut flip = self.co_cube.flip;
        let mut twist = self.co_cube.twist;
        let flipslice = (N_FLIP * slice_ as usize) + flip as usize;
        let classidx = self.solvertables.sy.flipslice_classidx[flipslice];
        let sym = self.solvertables.sy.flipslice_sym[flipslice];
        let mut depth_mod3 = self.solvertables.pr.get_flipslice_twist_depth3(
            N_TWIST * classidx as usize
                + self.solvertables.sy.twist_conj[((twist as usize) << 4) + sym as usize] as usize,
        );

        let mut depth = 0;
        while flip != SOLVED || slice_ != SOLVED || twist != SOLVED {
            if depth_mod3 == 0 {
                depth_mod3 = 3;
            }
            for m in ALL_MOVES {
                let twist1 = self.solvertables.mv.twist_move[N_MOVE * twist as usize + m as usize];
                let flip1 = self.solvertables.mv.flip_move[N_MOVE * flip as usize + m as usize];
                let slice1 = self.solvertables.mv.slice_sorted_move
                    [N_MOVE * slice_ as usize * N_PERM_4 + m as usize]
                    / N_PERM_4 as u16;
                let flipslice1 = N_FLIP * slice1 as usize + flip1 as usize;
                let classidx1 = self.solvertables.sy.flipslice_classidx[flipslice1];
                let sym = self.solvertables.sy.flipslice_sym[flipslice1];
                if self.solvertables.pr.get_flipslice_twist_depth3(
                    N_TWIST * classidx1 as usize
                        + self.solvertables.sy.twist_conj[((twist1 as usize) << 4) + sym as usize]
                            as usize,
                ) == depth_mod3 - 1
                {
                    depth += 1;
                    twist = twist1;
                    flip = flip1;
                    slice_ = slice1;
                    depth_mod3 -= 1;
                    break;
                }
            }
        }
        depth
    }

    /**
    Get distance to subgroup where only the UD-slice edges may be permuted in their slice (only 24/2 = 12 possible
    ways due to overall even parity). This is a lower bound for the number of moves to solve phase 2.

    # Parameters
    
    `corners`: Corners coordinate

    `ud_edges`: Coordinate of the 8 edges of U and D face.

    return:
    */
    fn get_depth_phase2(&self, corners: u16, ud_edges: u16) -> u16 {
        let mut corners = corners;
        let mut ud_edges = ud_edges;
        let classidx = self.solvertables.sy.corner_classidx[corners as usize];
        let sym = self.solvertables.sy.corner_sym[corners as usize];

        let mut depth_mod3 = self.solvertables.pr.get_corners_ud_edges_depth3(
            N_UD_EDGES * classidx as usize
                + self.solvertables.sy.ud_edges_conj[((ud_edges as usize) << 4) + sym as usize]
                    as usize,
        );
        if depth_mod3 == 3 {
            // unfilled entry, depth >= 11
            return 11;
        }
        let mut depth = 0;
        while corners != SOLVED || ud_edges != SOLVED {
            if depth_mod3 == 0 {
                depth_mod3 = 3;
            }
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
                let corners1 =
                    self.solvertables.mv.corners_move[N_MOVE * corners as usize + m as usize];
                let ud_edges1 =
                    self.solvertables.mv.ud_edges_move[N_MOVE * ud_edges as usize + m as usize];
                let classidx1 = self.solvertables.sy.corner_classidx[corners1 as usize];
                let sym = self.solvertables.sy.corner_sym[corners1 as usize];
                if self.solvertables.pr.get_corners_ud_edges_depth3(
                    N_UD_EDGES * classidx1 as usize
                        + self.solvertables.sy.ud_edges_conj
                            [((ud_edges1 as usize) << 4) + sym as usize]
                            as usize,
                ) == depth_mod3 - 1
                {
                    depth += 1;
                    corners = corners1;
                    ud_edges = ud_edges1;
                    depth_mod3 -= 1;
                    break;
                }
            }
        }
        depth
    }

    /// search_phase2
    fn search_phase2(
        &mut self,
        corners: u16,
        ud_edges: u16,
        slice_sorted: u16,
        dist: u16,
        togo_phase2: u16,
    ) -> bool {
        {
            let terminated = self.terminated.lock().unwrap();
            if *terminated || self.phase2_done {
                // println!("Search2 terminated {}", *terminated);
                return true;
            }
        }
        if togo_phase2 == 0 && slice_sorted == 0 {
            // phase 2 solved, store solution
            let mut man = self.sofar_phase1.clone();
            let mut other = self.sofar_phase2.clone();
            man.append(&mut other);
            let mut solutions = self.solutions.lock().unwrap();
            let lslen = (*solutions).last().unwrap().len();
            if (*solutions).len() == 1 || lslen > man.len() {
                if self.inv == 1 {
                    // we solved the inverse cube
                    man.reverse();
                    let mut newman = Vec::new();
                    for m in man {
                        newman.push(ALL_MOVES[(m as usize / 3) * 3 + (2 - (m as usize) % 3)]);
                        // R1->R3, R2->R2, R3->R1 etc.
                    }
                    man = newman;
                }
                let mut newman = Vec::new();
                for m in man {
                    newman.push(
                        ALL_MOVES[self.solvertables.sy.conj_move
                            [N_MOVE * 16 * self.rot as usize + m as usize]],
                    );
                }
                man = newman;
                self.shortest_length[0] = man.len();
                (*solutions).push(man);
            }

            if self.shortest_length[0] <= self.ret_length {
                // we have reached the target length
                let mut terminated = self.terminated.lock().unwrap();
                *terminated = true;
            }
            self.phase2_done = true;
        } else {
            for m in ALL_MOVES {
                if [
                    Move::R,
                    Move::R3,
                    Move::F,
                    Move::F3,
                    Move::L,
                    Move::L3,
                    Move::B,
                    Move::B3,
                ]
                .contains(&m)
                {
                    continue;
                }

                if self.sofar_phase2.len() > 0 {
                    let diff = *self.sofar_phase2.last().unwrap() as i8 / 3 - m as i8 / 3;
                    if [0, 3].contains(&diff) {
                        // successive moves: on same face or on same axis with wrong order
                        continue;
                    }
                } else {
                    if self.sofar_phase1.len() > 0 {
                        let diff = *self.sofar_phase1.last().unwrap() as i8 / 3 - m as i8 / 3;
                        if [0, 3].contains(&diff) {
                            // successive moves: on same face or on same axis with wrong order
                            continue;
                        }
                    }
                }

                let corners_new =
                    self.solvertables.mv.corners_move[18 * corners as usize + m as usize];
                let ud_edges_new =
                    self.solvertables.mv.ud_edges_move[18 * ud_edges as usize + m as usize];
                let slice_sorted_new =
                    self.solvertables.mv.slice_sorted_move[18 * slice_sorted as usize + m as usize];
                let classidx = self.solvertables.sy.corner_classidx[corners_new as usize];
                let sym = self.solvertables.sy.corner_sym[corners_new as usize];
                let dist_new_mod3 = self.solvertables.pr.get_corners_ud_edges_depth3(
                    40320 * classidx as usize
                        + self.solvertables.sy.ud_edges_conj
                            [((ud_edges_new as usize) << 4) + sym as usize]
                            as usize,
                );
                let dist_new =
                    self.solvertables.pr.distance[3 * dist as usize + dist_new_mod3 as usize];
                if max(
                    dist_new,
                    self.solvertables.pr.cornslice_depth
                        [24 * corners_new as usize + slice_sorted_new as usize],
                ) as u16
                    >= togo_phase2
                {
                    continue; // impossible to reach solved cube in togo_phase2 - 1 moves
                }
                self.sofar_phase2.push(m);
                self.search_phase2(
                    corners_new,
                    ud_edges_new,
                    slice_sorted_new,
                    dist_new as u16,
                    togo_phase2 - 1,
                );
                self.sofar_phase2.pop();
            }
        }
        true
    }

    fn search(
        &mut self,
        flip: u16,
        twist: u16,
        slice_sorted: u16,
        dist: u16,
        togo_phase1: u16,
    ) -> bool {
        {
            let terminated = self.terminated.lock().unwrap();
            if *terminated {
                return true;
            }
        }

        if togo_phase1 == 0 {
            // phase 1 solved
            {
                let solutions = self.solutions.lock().unwrap();
                if self.start_time.elapsed() > Duration::from_secs_f32(self.timeout)
                    && (*solutions).len() > 1
                {
                    let mut terminated = self.terminated.lock().unwrap();
                    *terminated = true;
                }
            }
            // compute initial phase 2 coordinates
            // check if list is not empty
            let m = match self.sofar_phase1.len() > 0 {
                true => *self.sofar_phase1.last().unwrap(),
                false => Move::U, //alue is irrelevant here, no phase 1 moves
            };

            let mut corners;

            if [Move::R3, Move::F3, Move::L3, Move::B3].contains(&m) {
                // phase 1 solution come in pairs
                corners = self.solvertables.mv.corners_move
                    [18 * self.cornersave as usize + m as usize - 1];
            // apply R2, F2, L2 ord B2 on last ph1 solution
            } else {
                corners = self.co_cube.corners;
                for m in &self.sofar_phase1 {
                    // get current corner configuration
                    corners =
                        self.solvertables.mv.corners_move[18 * corners as usize + *m as usize];
                }
                self.cornersave = corners;
            }

            // new solution must be shorter and we do not use phase 2 maneuvers with length > 11 - 1 = 10
            let togo2_limit = min(self.shortest_length[0] - self.sofar_phase1.len(), 11) as u16;
            if self.solvertables.pr.cornslice_depth[24 * corners as usize + slice_sorted as usize]
                >= togo2_limit
            {
                // precheck speeds up the computation
                return false;
            }

            let mut u_edges = self.co_cube.u_edges;
            let mut d_edges = self.co_cube.d_edges;
            for m in &self.sofar_phase1 {
                u_edges = self.solvertables.mv.u_edges_move[18 * u_edges as usize + *m as usize];
                d_edges = self.solvertables.mv.d_edges_move[18 * d_edges as usize + *m as usize];
            }

            let ud_edges =
                self.solvertables.em.upd_ud_edges[24 * u_edges as usize + d_edges as usize % 24];

            let dist2 = self.get_depth_phase2(corners, ud_edges);
            for togo2 in dist2..togo2_limit {
                // do not use more than togo2_limit - 1 moves in phase 2
                self.sofar_phase2 = Vec::new();
                self.phase2_done = false;
                self.search_phase2(corners, ud_edges, slice_sorted, dist2, togo2);
                if self.phase2_done {
                    // solution already found
                    break;
                }
            }
        } else {
            for m in ALL_MOVES {
                // dist = 0 means that we are already are in the subgroup H. If there are less than 5 moves left
                // this forces all remaining moves to be phase 2 moves. So we can forbid these at the end of phase 1
                // and generate these moves in phase 2.
                if dist == 0
                    && togo_phase1 < 5
                    && [
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
                    ]
                    .contains(&m)
                {
                    continue;
                }

                if self.sofar_phase1.len() > 0 {
                    let diff = *self.sofar_phase1.last().unwrap() as i8 / 3 - m as i8 / 3;
                    if [0, 3].contains(&diff) {
                        // successive moves: on same face or on same axis with wrong order
                        continue;
                    }
                }

                let flip_new = self.solvertables.mv.flip_move[18 * flip as usize + m as usize]; // N_MOVE = 18;
                let twist_new = self.solvertables.mv.twist_move[18 * twist as usize + m as usize];
                let slice_sorted_new =
                    self.solvertables.mv.slice_sorted_move[18 * slice_sorted as usize + m as usize];

                let flipslice = 2048 * (slice_sorted_new as usize / 24) + flip_new as usize; // N_FLIP * (slice_sorted / N_PERM_4) + flip;
                let classidx = self.solvertables.sy.flipslice_classidx[flipslice];
                let sym = self.solvertables.sy.flipslice_sym[flipslice];
                let dist_new_mod3 = self.solvertables.pr.get_flipslice_twist_depth3(
                    2187 * classidx as usize
                        + self.solvertables.sy.twist_conj
                            [((twist_new as usize) << 4) + sym as usize]
                            as usize,
                );
                let dist_new =
                    self.solvertables.pr.distance[3 * dist as usize + dist_new_mod3 as usize];
                if dist_new >= togo_phase1 {
                    // impossible to reach subgroup H in togo_phase1 - 1 moves
                    continue;
                }

                self.sofar_phase1.push(m);
                self.search(
                    flip_new,
                    twist_new,
                    slice_sorted_new,
                    dist_new,
                    togo_phase1 - 1,
                );
                self.sofar_phase1.pop();
            }
        }
        true
    }

    /// Start solverthread to find solution.
    pub fn start(&mut self) {
        let mut cb = CubieCube::default();
        let sc = &self.solvertables.sy.sc;
        if self.rot == 0 {
            // no rotation
            cb = CubieCube {
                cp: self.cb_cube.cp,
                co: self.cb_cube.co,
                ep: self.cb_cube.ep,
                eo: self.cb_cube.eo,
            };
        } else if self.rot == 1 {
            // conjugation by 120° rotation
            cb = CubieCube {
                cp: sc[32].cp,
                co: sc[32].co,
                ep: sc[32].ep,
                eo: sc[32].eo,
            };
            cb.multiply(self.cb_cube);
            cb.multiply(sc[16]);
        } else if self.rot == 2 {
            // conjugation by 240° rotation
            cb = CubieCube {
                cp: sc[16].cp,
                co: sc[16].co,
                ep: sc[16].ep,
                eo: sc[16].eo,
            };
            cb.multiply(self.cb_cube);
            cb.multiply(sc[32]);
        }
        if self.inv == 1 {
            // invert cube
            cb = cb.inverse_cubie_cube();
        }
        // self.co_cube = CoordCube::try_from(&cb)?; // the rotated/inverted cube in coordinate representation
        self.co_cube = CoordCube::from_cubie(&cb, &self.solvertables.sy).unwrap(); // the rotated/inverted cube in coordinate representation
        let dist = self.get_depth_phase1() as u16;
        for togo1 in dist..20 {
            // iterative deepening, solution has at least dist moves
            self.sofar_phase1 = Vec::new();
            let _ret = self
                .search(
                    self.co_cube.flip,
                    self.co_cube.twist,
                    self.co_cube.slice_sorted,
                    dist,
                    togo1,
                );
        }
    }
}

#[cfg(test)]
mod test {
    use crate::moves::Move::*;
    use crate::solver::*;

    #[test]
    fn test_solve() {
        let result = solve(
            "RLLBUFUUUBDURRBBUBRLRRFDFDDLLLUDFLRRDDFRLFDBUBFFLBBDUF",
            20,
            3.0,
        )
        .unwrap();
        assert_eq!(result.solution.len(), 17);
        assert_eq!(
            result.solution,
            vec![R, D2, B2, R2, L2, B3, U, F3, D2, R, B2, R2, F2, B2, R2, D2, B]
        );
        // println!("{:?}, ({})", solution, solution.len());
    }

    #[test]
    fn test_solver() {
        let result = solver(
            "RLLBUFUUUBDURRBBUBRLRRFDFDDLLLUDFLRRDDFRLFDBUBFFLBBDUF",
            "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
            20,
            3.0,
        ).unwrap();
        println!("{:?}, ({}), ({:?})", result.solution, result.solution.len(), result.solve_time);
    }
}
