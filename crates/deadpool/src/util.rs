lazy_static::lazy_static! {
    /// Cache the physical CPU count to avoid calling `num_cpus::get_physical`
    /// multiple times, which is expensive.
    ///
    /// TODO: in container environment, we should use the cpus limit of the container
    /// instead of the number of physical CPUs. Thus, we should call `num_cpus::get()`
    /// instead of `num_cpus::get_physical()`, which is aware of cgroup. This may be
    /// a breaking change so we don't do it for now.
    static ref PHYSICAL_CPU_COUNT: usize = num_cpus::get_physical();
}

/// Get the default maximum size of a pool, which is `cpu_count * 4` ignoring
/// any logical CPUs (Hyper-Threading).
pub(crate) fn get_default_pool_max_size() -> usize {
    *PHYSICAL_CPU_COUNT * 4
}
