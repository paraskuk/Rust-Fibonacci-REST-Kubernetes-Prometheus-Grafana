#[cfg(test)]
mod tests {
    //use crate::measure_time::measure_time;
    use fibonacci::measure_time::measure_time;
    use std::time::Duration;

    fn dummy_function() {
        std::thread::sleep(Duration::from_millis(100));
    }

    #[test]
    fn test_measure_time() {
        measure_time(dummy_function, "dummy_function");
    }
}
