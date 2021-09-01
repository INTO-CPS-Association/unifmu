use std::path::Path;

pub struct BenchmarkConfig {}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {}
    }
}

pub fn benchmark(rootdir: &Path, config: &BenchmarkConfig) {
    todo!();
}
