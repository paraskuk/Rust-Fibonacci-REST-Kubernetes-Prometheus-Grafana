use std::time::Instant;

pub fn measure_time<F, R>(func: F, func_name: &str) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = func();
    let duration = start.elapsed();
    println!("Time taken by {}: {:?}", func_name, duration);
    result
}