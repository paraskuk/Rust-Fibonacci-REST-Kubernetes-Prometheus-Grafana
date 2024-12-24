pub mod measure_time;

use std::collections::HashMap;

pub fn fibonacci(n: u32) -> u32 {
    if n < 2 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

pub fn fibonacci_match(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

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

