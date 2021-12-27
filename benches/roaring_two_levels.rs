use baziot::RoaringTwoLevels;
use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId,
    Criterion,
};
use rand::{seq::SliceRandom, thread_rng, Rng};

fn insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");
    for count in [1, 10, 100, 1_000, 10_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("Loop/Random", count),
            count,
            |b, &count| {
                let values = get_input_values(count, false);
                b.iter(|| {
                    let mut bitmap = RoaringTwoLevels::new();
                    for v in &values {
                        bitmap.insert(*v);
                    }
                    bitmap
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Iter/Random", count),
            count,
            |b, &count| {
                let values = get_input_values(count, false);
                b.iter(|| values.iter().copied().collect::<RoaringTwoLevels>());
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Loop/Sorted", count),
            count,
            |b, &count| {
                let values = get_input_values(count, true);
                b.iter(|| {
                    let mut bitmap = RoaringTwoLevels::new();
                    for v in &values {
                        bitmap.insert(*v);
                    }
                    bitmap
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Iter/Sorted", count),
            count,
            |b, &count| {
                let values = get_input_values(count, true);
                b.iter(|| values.iter().copied().collect::<RoaringTwoLevels>());
            },
        );
    }
    group.finish();
}

fn contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("contains");
    for count in [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("True", count),
            count,
            |b, &count| {
                let (bitmap, value) = random_bitmap(count);
                b.iter(|| bitmap.contains(black_box(value)));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("False", count),
            count,
            |b, &count| {
                let (bitmap, _) = random_bitmap(count);
                let value = bitmap.max().expect("non-empty bitmap") + 1;
                b.iter(|| bitmap.contains(black_box(value)));
            },
        );
    }
    group.finish();
}

fn cardinality(c: &mut Criterion) {
    let mut group = c.benchmark_group("cardinality");
    for count in [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                let (bitmap, _) = random_bitmap(count);
                b.iter(|| bitmap.cardinality());
            },
        );
    }
    group.finish();
}

fn is_empty(c: &mut Criterion) {
    let mut group = c.benchmark_group("is_empty");
    for count in [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                let (bitmap, _) = random_bitmap(count);
                b.iter(|| bitmap.is_empty());
            },
        );
    }
    group.finish();
}

fn remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove");
    for count in [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.iter_batched(
                    || random_bitmap(count),
                    |(mut bitmap, value)| bitmap.remove(black_box(value)),
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn get_input_values(count: i32, want_sorted: bool) -> Vec<u64> {
    let mut prng = thread_rng();
    let mut values = (0..count).map(|_| prng.gen::<u64>()).collect::<Vec<_>>();
    if want_sorted {
        values.sort_unstable()
    }
    values
}

// Also returns a random value (can be used to test `contains`, `remove`, …)
fn random_bitmap(cardinality: i32) -> (RoaringTwoLevels, u64) {
    let values = get_input_values(cardinality, true);
    let value = *values.choose(&mut thread_rng()).expect("non-empty values");

    (values.into_iter().collect::<RoaringTwoLevels>(), value)
}

criterion_group!(benches, insert, contains, cardinality, is_empty, remove);

criterion_main!(benches);
