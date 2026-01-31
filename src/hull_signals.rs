use numpy::PyReadonlyArray1;
use pyo3::prelude::*;

/// Determine trend direction from Hull MA crossover.
///
/// Compares the last value of a short-period Hull MA against the last value
/// of a long-period Hull MA.
///
/// Returns: 1 (uptrend), -1 (downtrend), 0 (neutral or insufficient data).
#[pyfunction]
pub fn hullma_trend(
    hullma_short: PyReadonlyArray1<'_, f64>,
    hullma_long: PyReadonlyArray1<'_, f64>,
) -> i32 {
    let short = hullma_short.as_slice().unwrap();
    let long = hullma_long.as_slice().unwrap();

    if short.len() < 2 || long.len() < 2 {
        return 0;
    }

    let current_short = short[short.len() - 1];
    let current_long = long[long.len() - 1];

    if current_short.is_nan() || current_long.is_nan() {
        return 0;
    }

    if current_short > current_long {
        1
    } else if current_short < current_long {
        -1
    } else {
        0
    }
}

/// Detect pullback opportunities relative to Hull MA.
///
/// A pullback occurs when the current price is within `threshold` distance
/// (as a ratio) of the long-period Hull MA.
///
/// Returns: (is_pullback, distance_ratio)
#[pyfunction]
#[pyo3(signature = (prices, hullma_long, threshold=0.03))]
pub fn hullma_pullback(
    prices: PyReadonlyArray1<'_, f64>,
    hullma_long: PyReadonlyArray1<'_, f64>,
    threshold: f64,
) -> (bool, f64) {
    let p = prices.as_slice().unwrap();
    let h = hullma_long.as_slice().unwrap();

    if p.is_empty() || h.is_empty() {
        return (false, 0.0);
    }

    let current_price = p[p.len() - 1];
    let current_hullma = h[h.len() - 1];

    if current_price.is_nan() || current_hullma.is_nan() || current_hullma <= 0.0 {
        return (false, 0.0);
    }

    let distance_ratio = (current_price - current_hullma).abs() / current_hullma;
    let is_pullback = distance_ratio <= threshold;

    (is_pullback, distance_ratio)
}

/// Detect recent price bounce (reversal momentum).
///
/// Compares the last two bars to detect upward or downward bounces
/// exceeding `threshold`.
///
/// Returns: (is_bounce_up, is_bounce_down, bounce_strength)
#[pyfunction]
#[pyo3(signature = (prices, threshold=0.002))]
pub fn hullma_bounce(
    prices: PyReadonlyArray1<'_, f64>,
    threshold: f64,
) -> (bool, bool, f64) {
    let p = prices.as_slice().unwrap();

    if p.len() < 2 {
        return (false, false, 0.0);
    }

    let recent_return = p[p.len() - 1] / p[p.len() - 2] - 1.0;
    let bounce_strength = recent_return.abs();

    let is_bounce_up = recent_return > threshold;
    let is_bounce_down = recent_return < -threshold;

    (is_bounce_up, is_bounce_down, bounce_strength)
}
