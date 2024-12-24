
#[cfg(test)]
mod tests {
    //use crate::measure_time::measure_time;
use std::time::Duration;
    use fibonacci::measure_time::measure_time;

    fn dummy_function() {
        std::thread::sleep(Duration::from_millis(100));
    }

    #[test]
    fn test_measure_time() {
        let result = measure_time(|| dummy_function(), "dummy_function");
        assert_eq!(result, ());
    }
}