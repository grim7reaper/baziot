use baziot::Roaring;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};

fn main() {
    let mut prng = thread_rng();
    let range = Uniform::from(0..500_000);
    let mut values = (0..32_000)
        .map(|_| range.sample(&mut prng))
        .collect::<Vec<_>>();
    values.sort_unstable();

    let bitmap = values.into_iter().collect::<Roaring>();

    println!("{:#?}", bitmap.stats());
}
