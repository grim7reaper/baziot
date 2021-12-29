/// Bitmap statistics.
#[derive(Debug)]
pub struct Stats<T> {
    /// Total number of containers.
    pub nb_containers: usize,
    /// Number of array containers.
    pub nb_array_containers: usize,
    /// Number of bitmap containers.
    pub nb_bitmap_containers: usize,

    /// Total number of values stored (cardinality).
    pub nb_values: usize,
    /// Number of values in array containers.
    pub nb_values_array_containers: usize,
    /// Number of values in bitmap containers.
    pub nb_values_bitmap_containers: usize,

    /// Total number of allocated bytes (approximated).
    pub nb_bytes: usize,
    /// Number of allocated bytes (approximated) in bitmap containers.
    pub nb_bytes_array_containers: usize,
    /// Number of allocated bytes (approximated) in bitmap containers.
    pub nb_bytes_bitmap_containers: usize,

    /// The minimal value, `None` if cardinality is zero.
    pub min_value: Option<T>,
    /// The maximal value, `None` if cardinality is zero.
    pub max_value: Option<T>,
}
