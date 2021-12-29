struct PID {
    proportional_factor: f64,

    integral: f64,
    integral_factor: f64,

    last_error: f64,
    derivative_factor: f64,
}

impl PID {
    fn new(proportional_factor: f64, integral_factor: f64, derivative_factor: f64) -> Self {
        PID {
            proportional_factor,
            integral: 0.0,
            integral_factor,
            last_error: 0.0,
            derivative_factor,
        }
    }

    fn next(&mut self, error: f64) -> f64 {
        let p = error;
        let i = self.next_integral(error);
        let d = self.next_derivative(error);

        self.integral = i;
        self.last_error = error;

        self.next_output(p, i, d)
    }

    fn next_integral(&self, error: f64) -> f64 {
        self.integral + error
    }

    fn next_derivative(&self, error: f64) -> f64 {
        error - self.last_error
    }

    fn next_output(self: &Self, p: f64, i: f64, d: f64) -> f64 {
        p * self.proportional_factor + i * self.integral_factor - d * self.derivative_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proportional() {
        let mut p = PID::new(1.0, 0.0, 0.0);

        assert_eq!(1.0, p.next(1.0));
    }

    #[test]
    fn test_proportional_factor() {
        let mut p = PID::new(2.0, 0.0, 0.0);

        assert_eq!(2.0, p.next(1.0));
    }

    #[test]
    fn test_integral() {
        let mut i = PID::new(0.0, 1.0, 0.0);

        assert_eq!(1.0, i.next(1.0));
    }

    #[test]
    fn test_integral_grows_over_time() {
        let mut i = PID::new(0.0, 1.0, 0.0);

        assert_eq!(1.0, i.next(1.0));
        assert_eq!(2.0, i.next(1.0));
        assert_eq!(3.0, i.next(1.0));
    }

    #[test]
    fn test_derivative() {
        let mut d = PID::new(0.0, 0.0, 1.0);

        assert_eq!(-1.0, d.next(1.0))
    }

    #[test]
    fn test_derivative_dampens_by_rate_of_change() {
        let mut d = PID::new(0.0, 0.0, 1.0);

        assert_eq!(-1.0, d.next(1.0));
        assert_eq!(0.0, d.next(1.0));
    }
}
