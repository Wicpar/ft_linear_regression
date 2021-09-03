
#[inline]
pub const fn estimate_price(kilometers: f64, theta: (f64, f64)) -> f64 {
    theta.0 + (theta.1 * kilometers)
}