/// Performs a saturating negation.
pub trait SaturatingNeg {
    type Output;

    /// Like [`Neg`](core::ops::Neg), but returns the maximum value on negation of
    /// a minimum value instead of overflowing.
    fn saturating_neg(self) -> Self::Output;
}

impl SaturatingNeg for i8 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl SaturatingNeg for i16 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}
impl SaturatingNeg for i32 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl SaturatingNeg for i64 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl SaturatingNeg for i128 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl SaturatingNeg for f32 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        -self
    }
}

impl SaturatingNeg for f64 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        -self
    }
}
