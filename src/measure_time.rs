use std::time::Instant;

///
///
/// # Arguments
///
/// * `func`:  F a function that takes no arguments and returns R
/// * `func_name`:  &str, the name of the function
///
/// returns: R  the result of the function
///
/// # Examples
///
/// ```
/// use fibonacci::measure_time::measure_time;
/// use std::collections::HashMap;
/// use fibonacci::fibonacci_match;
/// let mut memo: HashMap<u32, u32> = HashMap::new();
/// let result = measure_time(|| fibonacci_match(3), "fibonacci_match");
/// assert_eq!(result, 2);
/// ```
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