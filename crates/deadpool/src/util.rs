use std::sync::LazyLock;

/// Cache the physical CPU cofunt to avoid calling `num_cpus::get()`
/// multiple times, which is expensive when creating pools in quick
/// succession.
static CPU_COUNT: LazyLock<usize> = LazyLock::new(num_cpus::get);

/// Get the default maximum size of a pool, which is `cpu_core_count * 2`
/// including logical cores (Hyper-Threading).
pub(crate) fn get_default_pool_max_size() -> usize {
    *CPU_COUNT * 2
}
