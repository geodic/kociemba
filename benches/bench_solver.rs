use criterion::{criterion_group, criterion_main, Criterion};

use kociemba::cubie::CubieCube;
use kociemba::moves::Move::*;
use kociemba::solver::solve;

fn cc_apply_moves() {
    let cc = CubieCube::default();
    let _ = cc.apply_moves(&vec![R, U, R3, U3]);
}

fn cc_multi_moves() {
    let mut cc = CubieCube::default();
    cc.multiply_moves(&vec![R, U, R3, U3]);
}

fn bench_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("CubieCube Moves");
    group.bench_function("multiply_moves", |b| b.iter(|| cc_multi_moves()));
    group.bench_function("apply_moves", |b| b.iter(|| cc_apply_moves()));
    group.finish();
}

fn bench_solver(c: &mut Criterion) {
    c.bench_function("Solver", |b| {
        b.iter(|| {
            solve(
                "RLLBUFUUUBDURRBBUBRLRRFDFDDLLLUDFLRRDDFRLFDBUBFFLBBDUF",
                20,
                3.0,
            )
            .unwrap()
        })
    });
}

criterion_group!(benches, bench_solver, bench_moves);
criterion_main!(benches);
