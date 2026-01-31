use numpy::{PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

/// Exponential Moving Average.
///
/// alpha = 2 / (period + 1). First value seeded from data[0], then
/// ema[i] = alpha * data[i] + (1 - alpha) * ema[i-1].
/// No NaN output — the entire array is filled.
#[pyfunction]
pub fn ema<'py>(
    py: Python<'py>,
    data: PyReadonlyArray1<'py, f64>,
    period: usize,
) -> Bound<'py, PyArray1<f64>> {
    let d = data.as_slice().unwrap();
    let n = d.len();
    let mut result = vec![0.0; n];

    if n == 0 || period == 0 {
        return PyArray1::from_vec(py, result);
    }

    let alpha = 2.0 / (period as f64 + 1.0);
    result[0] = d[0];
    for i in 1..n {
        result[i] = alpha * d[i] + (1.0 - alpha) * result[i - 1];
    }

    PyArray1::from_vec(py, result)
}

/// Average True Range with Wilder's smoothing.
///
/// TR[0] = high[0] - low[0].
/// TR[i] = max(high-low, |high-prev_close|, |low-prev_close|).
/// ATR[period-1] = mean(TR[0..period]).
/// ATR[i] = ((period-1) * ATR[i-1] + TR[i]) / period.
/// Zeros for positions 0..period-2.
#[pyfunction]
#[pyo3(signature = (high, low, close, period=14))]
pub fn atr<'py>(
    py: Python<'py>,
    high: PyReadonlyArray1<'py, f64>,
    low: PyReadonlyArray1<'py, f64>,
    close: PyReadonlyArray1<'py, f64>,
    period: usize,
) -> Bound<'py, PyArray1<f64>> {
    let h = high.as_slice().unwrap();
    let l = low.as_slice().unwrap();
    let c = close.as_slice().unwrap();
    let n = h.len();

    let mut tr = vec![0.0; n];
    let mut result = vec![0.0; n];

    if n == 0 || period == 0 {
        return PyArray1::from_vec(py, result);
    }

    // First TR
    tr[0] = h[0] - l[0];

    // Remaining TR values
    for i in 1..n {
        let hl = h[i] - l[i];
        let hpc = (h[i] - c[i - 1]).abs();
        let lpc = (l[i] - c[i - 1]).abs();

        if hl >= hpc && hl >= lpc {
            tr[i] = hl;
        } else if hpc >= hl && hpc >= lpc {
            tr[i] = hpc;
        } else {
            tr[i] = lpc;
        }
    }

    // Initial ATR as simple average of first `period` TRs
    if period <= n {
        let mut sum_tr = 0.0;
        for i in 0..period {
            sum_tr += tr[i];
        }
        result[period - 1] = sum_tr / period as f64;

        // Wilder's smoothing
        for i in period..n {
            result[i] = ((period as f64 - 1.0) * result[i - 1] + tr[i]) / period as f64;
        }
    }

    PyArray1::from_vec(py, result)
}
