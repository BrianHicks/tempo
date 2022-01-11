static PROPORTIONAL_FACTOR: f64 = 1.5;
static INTEGRAL_FACTOR: f64 = 0.3;
static INTEGRAL_DECAY: f64 = 0.5;
static DERIVATIVE_FACTOR: f64 = 0.1;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Pid {
    pub integral: f64,
    pub last_error: f64,
}

impl Default for Pid {
    fn default() -> Pid {
        Pid {
            integral: 0.0,
            last_error: 0.0,
        }
    }
}

impl Pid {
    pub fn next(&mut self, error: f64) -> f64 {
        let p = error;
        let i = self.next_integral(error);
        let d = self.next_derivative(error);

        self.integral = i;
        self.last_error = error;

        let p_f = p * PROPORTIONAL_FACTOR;
        let i_f = i * INTEGRAL_FACTOR;
        let d_f = d * DERIVATIVE_FACTOR;
        let out = p_f + i_f - d_f;

        log::debug!("p: {}, i: {}, d: {}, out: {}", p_f, i_f, d_f, out);
        out
    }

    fn next_integral(&self, error: f64) -> f64 {
        let decay = if error.abs() < f64::EPSILON {
            INTEGRAL_DECAY
        } else {
            1.0
        };

        (self.integral + error) * decay
    }

    fn next_derivative(&self, error: f64) -> f64 {
        error - self.last_error
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn response_is_in_proportion_to_error() {
        let mut less = Pid::default();
        let mut more = Pid::default();

        // the values here are less important than the fact that the bigger
        // output correlates with the bigger input. However, assert! does not
        // give a nice error message when this fails if I do a direct comparison.
        assert_eq!(1.7, less.next(1.0));
        assert_eq!(3.4, more.next(2.0));
    }

    #[test]
    fn response_grows_over_time() {
        let mut pid = Pid::default();

        assert_eq!(1.7, pid.next(1.0));
        assert_eq!(2.1, pid.next(1.0));
        assert_eq!(2.4, pid.next(1.0));
        assert_eq!(2.7, pid.next(1.0));
        assert_eq!(3.0, pid.next(1.0));
    }

    #[test]
    fn sudden_large_error_is_dampened() {
        let mut pid = Pid::default();

        assert_eq!(1.7, pid.next(1.0));
        assert_eq!(2.1, pid.next(1.0));
        assert_eq!(9.2, pid.next(5.0));
    }
}
