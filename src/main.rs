use fibonacci::measure_time::measure_time;
use std::io;
use std::collections::HashMap;
use fibonacci::{fibonacci_match};
use fibonacci::{fibonacci_memo, fibonacci_iterative};

fn main() {
    println!("Please enter a number:");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    let n: u32 = input.trim().parse().expect("Please type a number!");
    let mut memo = HashMap::new();

    println!("fibonacci_memo({}) = {}", n, fibonacci_memo(n, &mut memo));
    println!("fibonacci_iterative({}) = {}", n, fibonacci_iterative(n));


    measure_time(|| fibonacci_match(n), "fibonacci_match");
    measure_time(|| fibonacci_memo(n, &mut memo), "fibonacci_memo");
    measure_time(|| fibonacci_iterative(n), "fibonacci_iterative");
}