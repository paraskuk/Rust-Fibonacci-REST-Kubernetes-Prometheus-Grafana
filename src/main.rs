// use fibonacci::measure_time::measure_time;
// use std::io;
// use std::collections::HashMap;
// use fibonacci::{fibonacci_match, fibonacci_memo, fibonacci_iterative};
//
// fn main() {
//     println!("Please enter a number:");
//
//     let mut input = String::new();
//     io::stdin().read_line(&mut input).expect("Failed to read line");
//
//     let n: u32 = input.trim().parse().expect("Please type a number!");
//     let mut memo = HashMap::new();
//
//     measure_time(|| fibonacci_match(n), "fibonacci_match");
//     measure_time(|| fibonacci_memo(n, &mut memo), "fibonacci_memo");
//     measure_time(|| fibonacci_iterative(n), "fibonacci_iterative");
// }
use fibonacci::measure_time::measure_time;
use std::env;
use std::collections::HashMap;
use std::process::exit;
use fibonacci::{fibonacci_match, fibonacci_memo, fibonacci_iterative};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <N>", args[0]);
        exit(1);
    }

    let n: u32 = args[1].parse().expect("Please provide a valid integer!");
    let mut memo = HashMap::new();

    println!("Calculating fibonacci(iterative method) for number {n} with result: {}", fibonacci_iterative(n));
    measure_time(|| fibonacci_match(n), "fibonacci_match");
    measure_time(|| fibonacci_memo(n, &mut memo), "fibonacci_memo");
    measure_time(|| fibonacci_iterative(n), "fibonacci_iterative");
}