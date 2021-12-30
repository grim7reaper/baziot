use baziot::{Roaring, RoaringLazy, RoaringTreeMap, RoaringTwoLevels};
use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BatchSize,
    BenchmarkId, Criterion, PlotConfiguration,
};
use rand::{
    distributions::Standard, prelude::*, seq::SliceRandom, thread_rng, Rng,
};

macro_rules! new_benchmark_group {
    // Initialize a new benchmark group with logarithmic axis scale.
    ($c:ident, $name:literal) => {{
        let plot_config =
            PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
        let mut group = $c.benchmark_group($name);
        group.plot_config(plot_config);
        group
    }};
}

macro_rules! bench_insert_loop {
    // Benchmark insertion using a loop, with either sorted or unsorted input.
    ($group:ident, $count:ident, $sorted:literal, $roaring: ty, $int:ty) => {
        let benchmark_id = BenchmarkId::new(stringify!($roaring), $count);
        $group.bench_with_input(benchmark_id, $count, |b, &$count| {
            let values: Vec<$int> = get_input_values($count, $sorted);
            b.iter(|| {
                let mut bitmap = <$roaring>::new();
                for v in &values {
                    bitmap.insert(*v);
                }
                bitmap
            });
        });
    };
}

fn insert_sorted_loop(c: &mut Criterion) {
    let mut group = new_benchmark_group!(c, "Inser/Sorted/Loop");
    for count in [1, 10, 100, 1_000, 10_000].iter() {
        bench_insert_loop!(group, count, true, Roaring, u32);
        bench_insert_loop!(group, count, true, RoaringTwoLevels, u64);
        bench_insert_loop!(group, count, true, RoaringTreeMap, u64);
        bench_insert_loop!(group, count, true, RoaringLazy, u64);
    }
    group.finish();
}

fn insert_random_loop(c: &mut Criterion) {
    let mut group = new_benchmark_group!(c, "Insert/Random/Loop");
    for count in [1, 10, 100, 1_000, 10_000].iter() {
        bench_insert_loop!(group, count, false, Roaring, u32);
        bench_insert_loop!(group, count, false, RoaringTwoLevels, u64);
        bench_insert_loop!(group, count, false, RoaringTreeMap, u64);
        bench_insert_loop!(group, count, false, RoaringLazy, u64);
    }
    group.finish();
}

macro_rules! bench_insert_iter {
    // Benchmark insertion using an iterator, with either sorted or unsorted
    // input.
    ($group:ident, $count:ident, $sorted: literal, $roaring: ty, $int:ty) => {
        let benchmark_id = BenchmarkId::new(stringify!($roaring), $count);
        $group.bench_with_input(benchmark_id, $count, |b, &$count| {
            let values: Vec<$int> = get_input_values($count, $sorted);
            b.iter(|| values.iter().copied().collect::<$roaring>());
        });
    };
}

fn insert_sorted_iter(c: &mut Criterion) {
    let mut group = new_benchmark_group!(c, "Insert/Sorted/Iter");
    for count in [1, 10, 100, 1_000, 10_000].iter() {
        bench_insert_iter!(group, count, true, Roaring, u32);
        bench_insert_iter!(group, count, true, RoaringTwoLevels, u64);
        bench_insert_iter!(group, count, true, RoaringTreeMap, u64);
        bench_insert_iter!(group, count, true, RoaringLazy, u64);
    }
    group.finish();
}

fn insert_random_iter(c: &mut Criterion) {
    let mut group = new_benchmark_group!(c, "Insert/Random/Iter");
    for count in [1, 10, 100, 1_000, 10_000].iter() {
        bench_insert_iter!(group, count, false, Roaring, u32);
        bench_insert_iter!(group, count, false, RoaringTwoLevels, u64);
        bench_insert_iter!(group, count, false, RoaringTreeMap, u64);
        bench_insert_iter!(group, count, false, RoaringLazy, u64);
    }
    group.finish();
}

macro_rules! bench_contains {
    // Benchmark membership test, with a value present or absent.
    ($group:ident, $count:ident, $present: literal, $roaring: ty, $int:ty) => {
        let benchmark_id = BenchmarkId::new(
            format!("{}/{}", stringify!($roaring), stringify!($present)),
            $count,
        );
        $group.bench_with_input(benchmark_id, $count, |b, &$count| {
            let (bitmap, mut value) = random_bitmap::<$roaring, $int>($count);
            if !$present {
                value = bitmap.max().expect("non-empty bitmap") + 1;
            }
            b.iter(|| bitmap.contains(black_box(value)));
        });
    };
}

fn contains_present(c: &mut Criterion) {
    let mut group = new_benchmark_group!(c, "Contains/Found");
    for count in [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        bench_contains!(group, count, true, Roaring, u32);
        bench_contains!(group, count, true, RoaringTwoLevels, u64);
        bench_contains!(group, count, true, RoaringTreeMap, u64);
        bench_contains!(group, count, true, RoaringLazy, u64);
    }
    group.finish();
}

fn contains_absent(c: &mut Criterion) {
    let mut group = new_benchmark_group!(c, "Contains/NotFound");
    for count in [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        bench_contains!(group, count, false, Roaring, u32);
        bench_contains!(group, count, false, RoaringTwoLevels, u64);
        bench_contains!(group, count, false, RoaringTreeMap, u64);
        bench_contains!(group, count, false, RoaringLazy, u64);
    }
    group.finish();
}

macro_rules! bench_cardinality {
    // Benchmark cardinality computation.
    ($group:ident, $count:ident, $roaring: ty, $int:ty) => {
        $group.bench_with_input(
            BenchmarkId::new(stringify!($roaring), $count),
            $count,
            |b, &$count| {
                let (bitmap, _) = random_bitmap::<$roaring, $int>($count);
                b.iter(|| bitmap.cardinality());
            },
        );
    };
}

fn cardinality(c: &mut Criterion) {
    let mut group = new_benchmark_group!(c, "Cardinality");
    for count in [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        bench_cardinality!(group, count, Roaring, u32);
        bench_cardinality!(group, count, RoaringTwoLevels, u64);
        bench_cardinality!(group, count, RoaringTreeMap, u64);
        bench_cardinality!(group, count, RoaringLazy, u64);
    }
    group.finish();
}

macro_rules! bench_is_empty {
    // Benchmark the emptyness test.
    ($group:ident, $count:ident, $roaring: ty, $int:ty) => {
        $group.bench_with_input(
            BenchmarkId::new(stringify!($roaring), $count),
            $count,
            |b, &$count| {
                let (bitmap, _) = random_bitmap::<$roaring, $int>($count);
                b.iter(|| bitmap.is_empty());
            },
        );
    };
}

fn is_empty(c: &mut Criterion) {
    let mut group = new_benchmark_group!(c, "IsEmpty");
    for count in [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        bench_is_empty!(group, count, Roaring, u32);
        bench_is_empty!(group, count, RoaringTwoLevels, u64);
        bench_is_empty!(group, count, RoaringTreeMap, u64);
        bench_is_empty!(group, count, RoaringLazy, u64);
    }
    group.finish();
}

macro_rules! bench_remove {
    // Benchmark the removal of a single value.
    ($group:ident, $count:ident, $roaring: ty, $int:ty) => {
        $group.bench_with_input(
            BenchmarkId::new(stringify!($roaring), $count),
            $count,
            |b, &$count| {
                b.iter_batched(
                    || random_bitmap::<$roaring, $int>($count),
                    |(mut bitmap, value)| bitmap.remove(black_box(value)),
                    BatchSize::SmallInput,
                );
            },
        );
    };
}

fn remove(c: &mut Criterion) {
    let mut group = new_benchmark_group!(c, "Remove");
    for count in [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        bench_remove!(group, count, Roaring, u32);
        bench_remove!(group, count, RoaringTwoLevels, u64);
        bench_remove!(group, count, RoaringTreeMap, u64);
        bench_remove!(group, count, RoaringLazy, u64);
    }
    group.finish();
}

/// Returns a list of random value, uniformly distributed and optionally sorted.
fn get_input_values<I>(count: i32, want_sorted: bool) -> Vec<I>
where
    I: Ord,
    Standard: Distribution<I>,
{
    let mut prng = thread_rng();
    let mut values = (0..count).map(|_| prng.gen::<I>()).collect::<Vec<_>>();
    if want_sorted {
        values.sort_unstable()
    }
    values
}

/// Builds a bitmap randomly populated.
///
/// Also returns a random value (can be used to test `contains`, `remove`, …).
fn random_bitmap<R, I>(cardinality: i32) -> (R, I)
where
    R: FromIterator<I>,
    I: Ord + Copy,
    Standard: rand::distributions::Distribution<I>,
{
    let values = get_input_values(cardinality, true);
    let value = *values.choose(&mut thread_rng()).expect("non-empty values");

    (values.into_iter().collect::<R>(), value)
}

criterion_group!(
    benches,
    insert_sorted_loop,
    insert_random_loop,
    insert_sorted_iter,
    insert_random_iter,
    contains_present,
    contains_absent,
    cardinality,
    is_empty,
    remove
);

criterion_main!(benches);
