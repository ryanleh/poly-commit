// For benchmark, run:
//     RAYON_NUM_THREADS=N cargo bench --no-default-features --features "std parallel" -- --nocapture
// where N is the number of threads you want to use (N = 1 for single-thread).

// TODO: Make a macro for benchmarking different types of poly commits
// TODO: Use TestInfo and generalize more

use ark_bls12_381::{Bls12_381, Fr as BlsFr};
use ark_ec::PairingEngine;
use ark_ff::test_rng;
use ark_poly::{univariate::DensePolynomial, UVPolynomial};
use ark_poly_commit::{marlin_pc::MarlinKZG10, LabeledPolynomial, PolynomialCommitment};
use ark_std::cmp::min;
use criterion::BenchmarkId;
use criterion::{criterion_group, criterion_main, Bencher, Criterion};

const BENCHMARK_MIN_DEGREE: usize = 1 << 15;
const BENCHMARK_MAX_DEGREE: usize = 1 << 18;
const BENCHMARK_LOG_INTERVAL_DEGREE: usize = 1;
const BENCHMARK_NUM_POLYS: usize = 4;
const BENCHMARK_HIDING_BOUND: usize = 1;

const ENABLE_COMMIT_BENCH: bool = true;

// Utility function for getting a vector of degrees to benchmark on.
// returns vec![2^{min}, 2^{min + interval}, ..., 2^{max}], where:
// interval = log_interval
// min      = ceil(log_2(min_degree))
// max      = ceil(log_2(max_degree))
pub fn size_range(log_interval: usize, min_degree: usize, max_degree: usize) -> Vec<usize> {
    let mut to_ret = vec![min_degree.next_power_of_two()];
    let interval = 1 << log_interval;
    while *to_ret.last().unwrap() < max_degree {
        let next_elem = min(max_degree, interval * to_ret.last().unwrap());
        to_ret.push(next_elem);
    }
    to_ret
}

// returns vec![2^{min}, 2^{min + interval}, ..., 2^{max}], where:
// interval = BENCHMARK_LOG_INTERVAL_DEGREE
// min      = ceil(log_2(BENCHMARK_MIN_DEGREE))
// max      = ceil(log_2(BENCHMARK_MAX_DEGREE))
fn default_size_range() -> Vec<usize> {
    size_range(
        BENCHMARK_LOG_INTERVAL_DEGREE,
        BENCHMARK_MIN_DEGREE,
        BENCHMARK_MAX_DEGREE,
    )
}

fn setup_bench<E: PairingEngine, P: UVPolynomial<E::Fr>, PC: PolynomialCommitment<E::Fr, P>>(
    c: &mut Criterion,
    name: &str,
    bench_fn: fn(&mut Bencher, &usize),
) {
    let mut group = c.benchmark_group(name);
    for degree in default_size_range().iter() {
        group.bench_with_input(BenchmarkId::from_parameter(degree), degree, bench_fn);
    }
    group.finish();
}

fn bench_poly_commit<
    E: PairingEngine,
    P: UVPolynomial<E::Fr>,
    PC: PolynomialCommitment<E::Fr, P>,
>(
    b: &mut Bencher,
    degree: &usize,
) {
    // Per benchmark setup
    // TODO: Add degree bound
    let rng = &mut test_rng();

    let degree_bound = None;
    let supported_degree = *degree;
    let supported_hiding_bound = BENCHMARK_HIDING_BOUND;
    let hiding_bound = Some(BENCHMARK_HIDING_BOUND);

    let pp = PC::setup(*degree, None, rng).unwrap();
    let (ck, _) = PC::trim(&pp, supported_degree, supported_hiding_bound, None).unwrap();

    let mut polynomials = Vec::new();
    for i in 0..BENCHMARK_NUM_POLYS {
        let label = format!("Test_{}_{}", i, degree);
        polynomials.push(LabeledPolynomial::new(
            label,
            P::rand(*degree, rng).into(),
            degree_bound,
            hiding_bound,
        ));
    }

    b.iter(|| {
        PC::commit(&ck, &polynomials, Some(rng)).unwrap();
    });
}

fn poly_commit_benches<
    E: PairingEngine,
    P: UVPolynomial<E::Fr>,
    PC: PolynomialCommitment<E::Fr, P>,
>(
    c: &mut Criterion,
    engine: &'static str,
    pc: &'static str,
) {
    if ENABLE_COMMIT_BENCH {
        let cur_name = format!("{:?}<{:?}> - commit", pc.clone(), engine.clone());
        setup_bench::<E, P, PC>(c, &cur_name, bench_poly_commit::<E, P, PC>);
    }
}

fn bench_kzg10_bls12_381(c: &mut Criterion) {
    let engine = "Bls12_381";
    let pc = "MarlinKZG10";
    poly_commit_benches::<
        Bls12_381,
        DensePolynomial<BlsFr>,
        MarlinKZG10<Bls12_381, DensePolynomial<BlsFr>>,
    >(c, engine, pc);

    //pub struct MarlinKZG10<E: PairingEngine, P: UVPolynomial<E::Fr>> {
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_kzg10_bls12_381
}
criterion_main!(benches);
