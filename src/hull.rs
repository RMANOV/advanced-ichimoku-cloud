use numpy::{PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Internal WMA computation — no PyO3 overhead, reusable across modules.
///
/// Weighted Moving Average with linear weights: most recent gets weight `period`,
/// oldest gets weight 1. Denominator = period * (period + 1) / 2.
/// Returns NaN for positions 0..period-1.
pub(crate) fn wma_inner(prices: &[f64], period: usize) -> Vec<f64> {
    let n = prices.len();
    if n < period || period == 0 {
        return vec![f64::NAN; n];
    }

    let mut wma = vec![f64::NAN; n];
    let weight_sum = (period * (period + 1)) as f64 / 2.0;

    for i in (period - 1)..n {
        let mut weighted_sum = 0.0;
        for j in 0..period {
            let weight = (period - j) as f64;
            weighted_sum += weight * prices[i - j];
        }
        wma[i] = weighted_sum / weight_sum;
    }

    wma
}

/// Internal Hull MA computation — 4-step lag-elimination algorithm.
///
/// HMA(n) = WMA(sqrt(n), 2 * WMA(n/2) - WMA(n))
///
/// Steps:
/// 1. Fast WMA with period n/2
/// 2. Slow WMA with period n
/// 3. Lag elimination: 2 * fast - slow
/// 4. Final smoothing: WMA(sqrt(n)) on the lag-eliminated series
pub(crate) fn hullma_inner(prices: &[f64], period: usize) -> Vec<f64> {
    let n = prices.len();
    if n < period || period == 0 {
        return vec![f64::NAN; n];
    }

    // Step 1: fast WMA(n/2)
    let half_period = (period / 2).max(2);
    let wma_half = wma_inner(prices, half_period);

    // Step 2: slow WMA(n)
    let wma_full = wma_inner(prices, period);

    // Step 3: lag elimination — 2 * WMA(n/2) - WMA(n)
    let mut raw_hma = vec![f64::NAN; n];
    for i in 0..n {
        if wma_half[i].is_nan() || wma_full[i].is_nan() {
            raw_hma[i] = f64::NAN;
        } else {
            raw_hma[i] = 2.0 * wma_half[i] - wma_full[i];
        }
    }

    // Step 4: final smoothing — WMA(sqrt(n)) on raw_hma
    let sqrt_period = ((period as f64).sqrt() as usize).max(2);
    wma_inner(&raw_hma, sqrt_period)
}

/// Weighted Moving Average with linear weights.
///
/// Weight scheme: most recent bar gets weight `period`, oldest gets weight 1.
/// Returns NaN for the first `period - 1` positions.
#[pyfunction]
pub fn wma<'py>(
    py: Python<'py>,
    prices: PyReadonlyArray1<'py, f64>,
    period: usize,
) -> Bound<'py, PyArray1<f64>> {
    let data = prices.as_slice().unwrap();
    let result = wma_inner(data, period);
    PyArray1::from_vec(py, result)
}

/// Hull Moving Average — low-lag smoothed moving average.
///
/// Formula: HMA(n) = WMA(sqrt(n), 2 * WMA(n/2) - WMA(n))
/// Reduces lag by ~50% compared to standard moving averages while
/// maintaining smoothness through the final WMA pass.
#[pyfunction]
pub fn hullma<'py>(
    py: Python<'py>,
    prices: PyReadonlyArray1<'py, f64>,
    period: usize,
) -> Bound<'py, PyArray1<f64>> {
    let data = prices.as_slice().unwrap();
    let result = hullma_inner(data, period);
    PyArray1::from_vec(py, result)
}
