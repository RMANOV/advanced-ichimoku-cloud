use numpy::{PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Internal classic Ichimoku line computation.
///
/// Rolling (max_high + min_low) / 2 over `period` bars.
/// Initial values (0..period-1) are backfilled with the first valid result.
/// Returns zeros if input is shorter than period.
pub(crate) fn ichimoku_line_inner(high: &[f64], low: &[f64], period: usize) -> Vec<f64> {
    let n = high.len();
    let mut result = vec![0.0; n];

    if n < period || period == 0 {
        return result;
    }

    for i in (period - 1)..n {
        let mut max_high = high[i - period + 1];
        let mut min_low = low[i - period + 1];

        for j in (i - period + 2)..=i {
            if high[j] > max_high {
                max_high = high[j];
            }
            if low[j] < min_low {
                min_low = low[j];
            }
        }

        result[i] = (max_high + min_low) / 2.0;
    }

    // Backfill initial values with first valid result
    if period - 1 < n && period > 1 {
        let initial_value = result[period - 1];
        for i in 0..(period - 1) {
            result[i] = initial_value;
        }
    }

    result
}

/// Classic Ichimoku line: (max_high + min_low) / 2 over a rolling window.
///
/// This is the traditional midpoint calculation used in Hosoda's original
/// Ichimoku Kinko Hyo system. Initial positions are backfilled with the
/// first valid value.
#[pyfunction]
pub fn ichimoku_line<'py>(
    py: Python<'py>,
    high: PyReadonlyArray1<'py, f64>,
    low: PyReadonlyArray1<'py, f64>,
    period: usize,
) -> Bound<'py, PyArray1<f64>> {
    let h = high.as_slice().unwrap();
    let l = low.as_slice().unwrap();
    let result = ichimoku_line_inner(h, l, period);
    PyArray1::from_vec(py, result)
}

/// Compute all four classic Ichimoku cloud components.
///
/// Returns (tenkan, kijun, senkou_span_a, senkou_span_b) where:
/// - tenkan = ichimoku_line(tenkan_period)  — conversion line
/// - kijun  = ichimoku_line(kijun_period)   — base line
/// - senkou_span_a = (tenkan + kijun) / 2   — leading span A
/// - senkou_span_b = ichimoku_line(senkou_period) — leading span B
#[pyfunction]
pub fn ichimoku_components<'py>(
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

    let tenkan = ichimoku_line_inner(h, l, tenkan_period);
    let kijun = ichimoku_line_inner(h, l, kijun_period);

    let mut senkou_a = vec![0.0; n];
    for i in 0..n {
        senkou_a[i] = (tenkan[i] + kijun[i]) / 2.0;
    }

    let senkou_b = ichimoku_line_inner(h, l, senkou_period);

    (
        PyArray1::from_vec(py, tenkan),
        PyArray1::from_vec(py, kijun),
        PyArray1::from_vec(py, senkou_a),
        PyArray1::from_vec(py, senkou_b),
    )
}
