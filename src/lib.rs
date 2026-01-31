use pyo3::prelude::*;

mod hull;
mod hull_signals;
mod ichimoku;
mod ichimoku_hull;
mod indicators;

/// Rust-accelerated Ichimoku Cloud with Hull MA smoothing.
/// Enhanced technical analysis: classic + Hull-based Ichimoku components.
#[pymodule]
fn advanced_ichimoku_cloud(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hull::wma, m)?)?;
    m.add_function(wrap_pyfunction!(hull::hullma, m)?)?;
    m.add_function(wrap_pyfunction!(hull_signals::hullma_trend, m)?)?;
    m.add_function(wrap_pyfunction!(hull_signals::hullma_pullback, m)?)?;
    m.add_function(wrap_pyfunction!(hull_signals::hullma_bounce, m)?)?;
    m.add_function(wrap_pyfunction!(ichimoku::ichimoku_line, m)?)?;
    m.add_function(wrap_pyfunction!(ichimoku::ichimoku_components, m)?)?;
    m.add_function(wrap_pyfunction!(ichimoku_hull::ichimoku_line_hull, m)?)?;
    m.add_function(wrap_pyfunction!(ichimoku_hull::ichimoku_components_hull, m)?)?;
    m.add_function(wrap_pyfunction!(indicators::ema, m)?)?;
    m.add_function(wrap_pyfunction!(indicators::atr, m)?)?;
    Ok(())
}
