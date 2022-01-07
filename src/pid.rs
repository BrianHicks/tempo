static DEFAULT_PROPORTIONAL_FACTOR: f64 = 1.5;
static DEFAULT_INTEGRAL_FACTOR: f64 = 0.3;
static DEFAULT_DERIVATIVE_FACTOR: f64 = 0.1;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Pid {
    pub proportional_factor: f64,

    pub integral: f64,
    pub integral_factor: f64,

    pub last_error: f64,
    pub derivative_factor: f64,
}

impl Default for Pid {
    fn default() -> Pid {
        Pid::new(
            DEFAULT_PROPORTIONAL_FACTOR,
            DEFAULT_INTEGRAL_FACTOR,
            DEFAULT_DERIVATIVE_FACTOR,
        )
    }
}

impl Pid {
    pub fn new(proportional_factor: f64, integral_factor: f64, derivative_factor: f64) -> Self {
        Pid {
            proportional_factor,
            integral: 0.0,
            integral_factor,
            last_error: 0.0,
            derivative_factor,
        }
    }

    pub fn next(&mut self, error: f64) -> f64 {
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

    fn next_output(&self, p: f64, i: f64, d: f64) -> f64 {
        let p_f = p * self.proportional_factor;
        let i_f = i * self.integral_factor;
        let d_f = d * self.derivative_factor;
        let out = p_f + i_f - d_f;

        log::debug!("p: {}, i: {}, d: {}, out: {}", p_f, i_f, d_f, out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proportional() {
        let mut p = Pid::new(1.0, 0.0, 0.0);

        assert_eq!(1.0, p.next(1.0));
    }

    #[test]
    fn test_proportional_factor() {
        let mut p = Pid::new(2.0, 0.0, 0.0);

        assert_eq!(2.0, p.next(1.0));
    }

    #[test]
    fn test_integral() {
        let mut i = Pid::new(0.0, 1.0, 0.0);

        assert_eq!(1.0, i.next(1.0));
    }

    #[test]
    fn test_integral_grows_over_time() {
        let mut i = Pid::new(0.0, 1.0, 0.0);

        assert_eq!(1.0, i.next(1.0));
        assert_eq!(2.0, i.next(1.0));
        assert_eq!(3.0, i.next(1.0));
    }

    #[test]
    fn test_derivative() {
        let mut d = Pid::new(0.0, 0.0, 1.0);

        assert_eq!(-1.0, d.next(1.0))
    }

    #[test]
    fn test_derivative_dampens_by_rate_of_change() {
        let mut d = Pid::new(0.0, 0.0, 1.0);

        assert_eq!(-1.0, d.next(1.0));
        assert_eq!(0.0, d.next(1.0));
    }
}
