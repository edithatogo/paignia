//! Internal outward-rounded arithmetic shared by numerical certificates.

#[derive(Clone, Copy, Debug, thiserror::Error)]
#[error("{0}")]
pub(crate) struct ArithmeticError(pub(crate) &'static str);

/// Finite bounds enclosing real arithmetic on the supplied binary64 inputs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub lower: f64,
    pub upper: f64,
}

// One representable neighbour outwards encloses a correctly rounded operation,
// including gradual underflow. An unbounded endpoint causes an explicit error.
fn neighbour(value: f64, upward: bool) -> Result<f64, ArithmeticError> {
    if !value.is_finite() {
        return Err(ArithmeticError("non-finite operation"));
    }
    let result = if value == 0. {
        if upward {
            f64::from_bits(1)
        } else {
            -f64::from_bits(1)
        }
    } else if (value > 0.) == upward {
        f64::from_bits(value.to_bits() + 1)
    } else {
        f64::from_bits(value.to_bits() - 1)
    };
    if result.is_finite() {
        Ok(result)
    } else {
        Err(ArithmeticError("outward rounding overflow"))
    }
}

impl Interval {
    pub(crate) fn point(value: f64) -> Self {
        Self {
            lower: value,
            upper: value,
        }
    }
    pub(crate) fn is_point(self, value: f64) -> bool {
        let equal =
            |bound: f64| bound.to_bits() == value.to_bits() || (bound == 0.0 && value == 0.0);
        equal(self.lower) && equal(self.upper)
    }
    fn neg(self) -> Self {
        Self {
            lower: -self.upper,
            upper: -self.lower,
        }
    }
    pub(crate) fn add(self, other: Self) -> Result<Self, ArithmeticError> {
        if self.is_point(0.) {
            return Ok(other);
        }
        if other.is_point(0.) {
            return Ok(self);
        }
        if self.is_point(self.lower) && other.is_point(-self.lower) {
            return Ok(Self::point(0.));
        }
        Ok(Self {
            lower: neighbour(self.lower + other.lower, false)?,
            upper: neighbour(self.upper + other.upper, true)?,
        })
    }
    pub(crate) fn sub(self, other: Self) -> Result<Self, ArithmeticError> {
        self.add(other.neg())
    }
    pub(crate) fn mul(self, other: Self) -> Result<Self, ArithmeticError> {
        if self.is_point(0.) || other.is_point(0.) {
            return Ok(Self::point(0.));
        }
        if self.is_point(1.) {
            return Ok(other);
        }
        if other.is_point(1.) {
            return Ok(self);
        }
        if self.is_point(-1.) {
            return Ok(other.neg());
        }
        if other.is_point(-1.) {
            return Ok(self.neg());
        }
        Self::enclose_corners([
            self.lower * other.lower,
            self.lower * other.upper,
            self.upper * other.lower,
            self.upper * other.upper,
        ])
    }
    pub(crate) fn div_positive(self, other: Self) -> Result<Self, ArithmeticError> {
        if other.lower <= 0. {
            return Err(ArithmeticError("uncertain recommendation mass"));
        }
        if self.is_point(0.) {
            return Ok(Self::point(0.));
        }
        if other.is_point(1.) {
            return Ok(self);
        }
        Self::enclose_corners([
            self.lower / other.lower,
            self.lower / other.upper,
            self.upper / other.lower,
            self.upper / other.upper,
        ])
    }
    fn enclose_corners(values: [f64; 4]) -> Result<Self, ArithmeticError> {
        if values.iter().any(|value| !value.is_finite()) {
            return Err(ArithmeticError("interval product or quotient"));
        }
        let lower = values.iter().copied().fold(values[0], f64::min);
        let upper = values.iter().copied().fold(values[0], f64::max);
        Ok(Self {
            lower: neighbour(lower, false)?,
            upper: neighbour(upper, true)?,
        })
    }
    pub(crate) fn abs_upper(self) -> f64 {
        self.lower.abs().max(self.upper.abs())
    }
}
