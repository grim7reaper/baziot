use baziot::{Roaring, RoaringLazy, RoaringTreeMap, RoaringTwoLevels};
use humansize::{file_size_opts as options, FileSize};
use rand::{distributions::Standard, prelude::*, thread_rng, Rng};

fn main() {
    for count in [0, 1, 10, 100, 1_000, 10_000, 100_000, 1_000_000].into_iter()
    {
        let roaring = random_bitmap::<Roaring, u32>(count);
        let two_levels = random_bitmap::<RoaringTwoLevels, u64>(count);
        let tree_map = random_bitmap::<RoaringTreeMap, u64>(count);
        let lazy = random_bitmap::<RoaringLazy, u64>(count);

        println!(
            "count: {:7} Roaring:{:9} TwoLevels:{:9} TreeMap:{:9} Lazy:{:9}",
            count,
            roaring.mem_size().file_size(options::DECIMAL).unwrap(),
            two_levels.mem_size().file_size(options::DECIMAL).unwrap(),
            tree_map.mem_size().file_size(options::DECIMAL).unwrap(),
            lazy.mem_size().file_size(options::DECIMAL).unwrap()
        );
    }
}

/// Builds a bitmap randomly populated.
fn random_bitmap<R, I>(count: i32) -> R
where
    R: FromIterator<I>,
    I: Ord,
    Standard: Distribution<I>,
{
    let mut prng = thread_rng();
    let mut values = (0..count).map(|_| prng.gen::<I>()).collect::<Vec<_>>();
    values.sort_unstable();
    values.into_iter().collect::<R>()
}
