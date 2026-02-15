pub mod measure_time;

use std::collections::HashMap;

///
///
/// # Arguments
///
/// * `n`:  u32, the nth number in the fibonacci sequence
///
/// returns: u32,the result of the nth number in the fibonacci sequence
///
/// # Examples
///
/// ```
///  use fibonacci::fibonacci;
/// let result = fibonacci(1);
///  assert_eq!(result, 1);
/// ```
pub fn fibonacci(n: u32) -> u32 {
    if n < 2 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

///
///
/// # Arguments
///
/// * `n`:  u32, the nth number in the fibonacci sequence
///
/// returns: u32, result of the nth number in the fibonacci sequence
///
/// # Examples
///
/// ```
/// use fibonacci::fibonacci_match;
///  let result = fibonacci_match(3);
///  assert_eq!(result, 2);
/// ```
pub fn fibonacci_match(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

///
///
/// # Arguments
///
/// * `n`:  u32, the nth number in the fibonacci sequence
///
/// returns: u32  the result of the nth number in the fibonacci sequence
///
/// # Examples
///
/// ```
/// use fibonacci::fibonacci_dp;
/// let result = fibonacci_dp(4);
/// assert_eq!(result, 3);
//
/// ```
pub fn fibonacci_dp(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let mut fib = vec![0; (n + 1) as usize];
    fib[1] = 1;
    for i in 2..=n as usize {
        fib[i] = fib[i - 1] + fib[i - 2];
    }
    fib[n as usize]
}

///
///
/// # Arguments
///
/// * `n`:  u32, the nth number in the fibonacci sequence
/// * `memo`:  &mut HashMap<u32, u32>, a mutable reference to a hashmap
///
/// returns: u32  the result of the nth number in the fibonacci sequence
///
/// # Examples
///
/// ```
/// use fibonacci::fibonacci_memo;
/// use std::collections::HashMap;
/// let mut memo = HashMap::new();
/// let result = fibonacci_memo(4, &mut memo);
/// assert_eq!(result, 3);
/// ```
pub fn fibonacci_memo(n: u32, memo: &mut HashMap<u32, u32>) -> u32 {
    if let Some(&result) = memo.get(&n) {
        return result;
    }
    let result = if n < 2 {
        n
    } else {
        fibonacci_memo(n - 1, memo) + fibonacci_memo(n - 2, memo)
    };
    memo.insert(n, result);
    result
}

///
///
/// # Arguments
///
/// * `n`:  u32, the nth number in the fibonacci sequence
///
/// returns: u32  the result of the nth number in the fibonacci sequence
///
/// # Examples
///
/// ```
/// use fibonacci::fibonacci_iterative;
/// let result = fibonacci_iterative(4);
/// assert_eq!(result, 3);
///
/// ```
pub fn fibonacci_iterative(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let mut a = 0;
    let mut b = 1;
    for _ in 1..n {
        let temp = b;
        b = a + b;
        a = temp;
    }
    b
}
