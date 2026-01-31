use numpy::{PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

use crate::hull::hullma_inner;

/// Internal Hull-enhanced Ichimoku line computation.
///
/// Instead of the classic (max_high + min_low) / 2, this computes
/// the median price (high + low) / 2 and applies Hull Moving Average
/// for reduced lag and smoother curves.
fn ichimoku_line_hull_inner(high: &[f64], low: &[f64], period: usize) -> Vec<f64> {
    let n = high.len();

    // Compute median price series: (high + low) / 2
    let mut hl_median = vec![0.0; n];
    for i in 0..n {
        hl_median[i] = (high[i] + low[i]) / 2.0;
    }

    // Apply Hull MA to the median series
    hullma_inner(&hl_median, period)
}

/// Hull-enhanced Ichimoku line: Hull MA applied to (high + low) / 2.
///
/// Replaces the classic (max + min) / 2 midpoint with a Hull Moving Average
/// of the bar midpoint, reducing lag by ~50% while producing smooth curves.
/// Returns NaN for initial positions where Hull MA has insufficient data.
#[pyfunction]
pub fn ichimoku_line_hull<'py>(
    py: Python<'py>,
    high: PyReadonlyArray1<'py, f64>,
    low: PyReadonlyArray1<'py, f64>,
    period: usize,
) -> Bound<'py, PyArray1<f64>> {
    let h = high.as_slice().unwrap();
    let l = low.as_slice().unwrap();
    let result = ichimoku_line_hull_inner(h, l, period);
    PyArray1::from_vec(py, result)
}

/// Compute all four Hull-enhanced Ichimoku cloud components.
///
/// Returns (tenkan, kijun, senkou_span_a, senkou_span_b) where each line
/// uses Hull MA smoothing instead of classic midpoint calculation.
/// senkou_span_a is NaN-aware: NaN if either tenkan or kijun is NaN.
#[pyfunction]
pub fn ichimoku_components_hull<'py>(
    py: Python<'py>,
    high: PyReadonlyArray1<'py, f64>,
    low: PyReadonlyArray1<'py, f64>,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_period: usize,
) -> (
    Bound<'py, PyArray1<f64>>,
    Bound<'py, PyArray1<f64>>,
    Bound<'py, PyArray1<f64>>,
    Bound<'py, PyArray1<f64>>,
) {
    let h = high.as_slice().unwrap();
    let l = low.as_slice().unwrap();
    let n = h.len();

    let tenkan = ichimoku_line_hull_inner(h, l, tenkan_period);
    let kijun = ichimoku_line_hull_inner(h, l, kijun_period);

    // NaN-aware senkou_span_a
    let mut senkou_a = vec![f64::NAN; n];
    for i in 0..n {
        if !tenkan[i].is_nan() && !kijun[i].is_nan() {
            senkou_a[i] = (tenkan[i] + kijun[i]) / 2.0;
        }
    }

    let senkou_b = ichimoku_line_hull_inner(h, l, senkou_period);

    (
        PyArray1::from_vec(py, tenkan),
        PyArray1::from_vec(py, kijun),
        PyArray1::from_vec(py, senkou_a),
        PyArray1::from_vec(py, senkou_b),
    )
}
