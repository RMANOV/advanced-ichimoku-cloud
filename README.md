# advanced-ichimoku-cloud

**Rust + PyO3 Enhanced Ichimoku Cloud with Hull MA smoothing** — a compiled, zero-copy, GIL-free technical analysis library combining classic Ichimoku Kinko Hyo with Hull Moving Average lag reduction.

All 11 numerical functions implemented in safe Rust. Bit-exact parity with Python reference implementations (verified to `1e-10` tolerance). One-line import swap to activate.

---

## A Brief History of Ichimoku Kinko Hyo

The Ichimoku Cloud has one of the most remarkable origin stories in technical analysis — a single journalist spent three decades perfecting it before releasing it to the public.

**1930s — Goichi Hosoda begins development**
Goichi Hosoda (pen name **Ichimoku Sanjin**, literally "a glance from a man on a mountain") was a Japanese newspaper reporter who began developing a comprehensive equilibrium chart system. His goal: a single indicator that could show support/resistance, trend direction, momentum, and future projections — all at a glance. He enlisted teams of university students to manually compute and backtest the system across decades of Japanese stock and commodity data, long before computers were available.

**1968 — Publication after 30 years of research**
Hosoda published his system in Japanese as *Ichimoku Kinko Hyo* (一目均衡表), which translates to "one-glance equilibrium chart." The original parameters (9, 26, 52) were calibrated to the Japanese trading calendar: 9 trading days (1.5 weeks), 26 trading days (1 month), and 52 trading days (2 months). The system was designed for daily bars on Japanese equity and commodity markets.

**1990s–2000s — Western adoption**
Ichimoku remained almost exclusively Japanese until the internet era. Western traders discovered it through translated materials and recognized its power as a complete trading system — not just an indicator, but a full framework integrating time, price, and equilibrium theory. The cloud (kumo) component proved particularly valuable for identifying support/resistance zones that other indicators missed.

**Present — Enhanced variants**
Modern implementations adapt the original parameters for different timeframes (intraday, weekly) and markets (crypto, forex). This library implements both the classic system and an enhanced variant that replaces the midpoint calculation with Hull Moving Average smoothing for reduced lag.

---

## The Five Classical Components

Hosoda's system produces five lines, each serving a distinct analytical purpose:

| Component | Japanese Name | Formula | Interpretation |
|---|---|---|---|
| **Conversion Line** | Tenkan-sen (転換線) | (highest high + lowest low) / 2 over short period | Short-term equilibrium. Acts as minor support/resistance. |
| **Base Line** | Kijun-sen (基準線) | (highest high + lowest low) / 2 over medium period | Medium-term equilibrium. Primary signal line. |
| **Leading Span A** | Senkou Span A (先行スパンA) | (Tenkan + Kijun) / 2 | Front edge of the cloud. Faster-moving boundary. |
| **Leading Span B** | Senkou Span B (先行スパンB) | (highest high + lowest low) / 2 over long period | Back edge of the cloud. Slower-moving boundary. |
| **Lagging Span** | Chikou Span (遅行スパン) | Close price shifted back N periods | Confirms trend by comparing current price to historical. |

The area between Senkou Span A and Senkou Span B forms the **cloud (kumo)**. When price is above the cloud, the trend is bullish. When below, bearish. When inside, the market is in transition. Cloud thickness indicates support/resistance strength.

---

## Classic vs Enhanced (Hull MA) Ichimoku

This library implements both approaches. The enhanced variant replaces the midpoint `(max + min) / 2` with Hull Moving Average smoothing:

| Aspect | Classic Ichimoku | Enhanced (Hull MA) |
|---|---|---|
| **Line calculation** | `(max_high + min_low) / 2` | Hull MA of `(high + low) / 2` median |
| **Lag** | Full period lag | ~50% reduced via HMA formula |
| **Weighting** | Equal weight to all bars in window | WMA gives higher weight to recent bars |
| **Default periods** | 9 / 26 / 52 | 10 / 30 / 60 (configurable) |
| **Chikou shift** | 26 periods | 30 periods |
| **Smoothness** | Stepped (changes only at new max/min) | Smooth continuous curve |
| **Sensitivity** | Reacts only to extreme values | Reacts to price distribution changes |
| **NaN behavior** | Zeros, backfilled with first valid | NaN from Hull MA chain |

**When to use classic:** Identifying hard support/resistance levels, traditional Ichimoku signal interpretation.

**When to use enhanced:** Trend-following strategies requiring fast reaction, reducing whipsaw in noisy data, smoother cloud boundaries.

---

## Hull Moving Average — The Key Innovation

The Hull Moving Average (HMA), developed by Alan Hull in 2005, solves the fundamental lag-vs-smoothness tradeoff in moving averages. Standard moving averages must choose: short period (fast but noisy) or long period (smooth but laggy). HMA achieves both through a clever algebraic trick.

### The 4-Step Formula

```
HMA(n) = WMA(√n,  2 × WMA(n/2) - WMA(n))
```

**Step 1 — Fast WMA:** Calculate `WMA(n/2)` — a fast weighted moving average using half the period.

**Step 2 — Slow WMA:** Calculate `WMA(n)` — a slow weighted moving average using the full period.

**Step 3 — Lag elimination:** Compute `2 × WMA(n/2) - WMA(n)`. This extrapolates from the difference between fast and slow lines, overshooting the actual price to compensate for lag. Think of it as: "if the fast line leads the slow line by X, the true value is probably X ahead of the fast line."

**Step 4 — Final smoothing:** Apply `WMA(√n)` to the lag-eliminated series. This removes the noise introduced by the extrapolation step, using a very short period (√n) to preserve the lag reduction.

### WMA Weight Scheme

The Weighted Moving Average assigns linearly decreasing weights:
```
WMA(n) = (n × P₁ + (n-1) × P₂ + ... + 1 × Pₙ) / (n × (n+1) / 2)
```
Most recent price gets weight `n`, oldest gets weight `1`. This naturally emphasizes recent data while still considering the full window.

### Why lag reduces by ~50%

The `2 × fast - slow` step is the key. The slow WMA lags by approximately `n/2` bars. The fast WMA lags by approximately `n/4` bars. The difference is `n/4` bars, and doubling it gives `n/2` — which exactly compensates the slow line's lag. The final WMA(√n) smoothing adds only `√n/2` bars of lag, far less than the original `n/2`.

---

## Mathematical Formulas

### WMA (Weighted Moving Average)
```
WMA(n)[i] = Σⱼ₌₀ⁿ⁻¹ (n - j) × price[i - j]  /  (n × (n + 1) / 2)
```
NaN for positions `0..n-1`.

### HMA (Hull Moving Average)
```
HMA(n) = WMA(⌊√n⌋, 2 × WMA(⌊n/2⌋) - WMA(n))
```
Half period = max(2, n/2). Sqrt period = max(2, √n).

### Ichimoku Line (Classic)
```
line[i] = (max(high[i-p+1..i]) + min(low[i-p+1..i])) / 2
```
Initial positions backfilled with first valid value.

### Ichimoku Line (Hull-Enhanced)
```
median[i] = (high[i] + low[i]) / 2
line = HMA(period, median)
```

### EMA (Exponential Moving Average)
```
α = 2 / (period + 1)
ema[0] = data[0]
ema[i] = α × data[i] + (1 - α) × ema[i-1]
```

### ATR (Average True Range)
```
TR[0] = high[0] - low[0]
TR[i] = max(high[i]-low[i], |high[i]-close[i-1]|, |low[i]-close[i-1]|)
ATR[period-1] = mean(TR[0..period])
ATR[i] = ((period-1) × ATR[i-1] + TR[i]) / period    (Wilder's smoothing)
```

---

## Architecture

```
                      Python (application layer)
               ┌──────────────────────────────────────┐
               │         Your Python code              │
               │    import advanced_ichimoku_cloud      │
               └──────────────────┬───────────────────┘
                                  │
                                  ▼
            ┌─────────────────────────────────────────┐
            │     advanced_ichimoku_cloud (Rust)       │
            │                                         │
            │  ┌────────────┐    ┌─────────────────┐  │
            │  │ hull.rs     │    │ hull_signals.rs │  │
            │  │  wma        │    │  hullma_trend   │  │
            │  │  hullma     │    │  hullma_pullback│  │
            │  │  wma_inner ─┼──▶ │  hullma_bounce  │  │
            │  │  hullma_    │    └─────────────────┘  │
            │  │    inner    │                         │
            │  └──────┬──────┘                         │
            │         │ (cross-module reuse)            │
            │         ▼                                │
            │  ┌────────────────┐  ┌────────────────┐  │
            │  │ ichimoku.rs    │  │ ichimoku_hull.rs│  │
            │  │  ichimoku_line │  │  ichimoku_line  │  │
            │  │  ichimoku_     │  │    _hull        │  │
            │  │  components    │  │  ichimoku_      │  │
            │  │  (classic)     │  │  components_hull│  │
            │  └────────────────┘  │  (enhanced)     │  │
            │                      └────────────────┘  │
            │  ┌────────────────┐                      │
            │  │ indicators.rs  │                      │
            │  │  ema           │                      │
            │  │  atr           │                      │
            │  └────────────────┘                      │
            └─────────────────────────────────────────┘
                      │ PyO3 + numpy FFI │
                      ▼                  ▼
            ┌─────────────────────────────────────────┐
            │         NumPy ndarrays (f64)             │
            │     zero-copy read via as_slice()        │
            └─────────────────────────────────────────┘
```

### Internal Function Architecture

Key design: internal computation as plain `fn` (no PyO3 overhead). `#[pyfunction]` wrappers delegate to them. This enables cross-module reuse without double FFI cost:

```
ichimoku_hull.rs ──uses──▶ hull.rs::hullma_inner()
                                      │
                                      ▼
                             hull.rs::wma_inner()

ichimoku.rs::ichimoku_components() ──uses──▶ ichimoku.rs::ichimoku_line_inner()
```

---

## Why Rust + Python

Python excels at prototyping, data wrangling, and orchestration. Rust excels at sustained, predictable, low-latency number crunching without a garbage collector. Combining them via PyO3 gives you both:

| Dimension | Python alone (Numba JIT) | Rust via PyO3 |
|---|---|---|
| **First-call latency** | 2-5 s JIT warm-up per function | Zero — compiled ahead of time |
| **GIL** | Held during Numba execution | Released — other Python threads run freely |
| **Memory safety** | Runtime bounds checks | Compile-time guarantees — no segfaults |
| **Dependency weight** | `numba` + `llvmlite` (~150 MB) | Single `.so` file (~2 MB), no LLVM runtime |
| **Reproducibility** | JIT output can vary across LLVM versions | Deterministic binary — same result everywhere |
| **Distribution** | Requires Numba installed everywhere | `pip install *.whl` — self-contained |

---

## Quickstart

```bash
# 1. Clone
git clone https://github.com/RMANOV/advanced-ichimoku-cloud.git
cd advanced-ichimoku-cloud

# 2. Create virtualenv + install deps
python -m venv .venv && source .venv/bin/activate
pip install maturin numpy

# 3. Build (release mode, optimized)
maturin develop --release

# 4. Verify
python tests/test_parity.py
```

Expected output:

```
============================================================
  Parity Tests: advanced-ichimoku-cloud
============================================================
  PASS  wma
  PASS  hullma
  PASS  hullma_trend
  PASS  hullma_pullback
  PASS  hullma_bounce
  PASS  ichimoku_line
  PASS  ichimoku_components
  PASS  ichimoku_line_hull
  PASS  ichimoku_components_hull
  PASS  ema
  PASS  atr
============================================================
  ALL 11 FUNCTIONS PASS PARITY TESTS
============================================================
```

---

## Integration

```python
from advanced_ichimoku_cloud import (
    # Hull Moving Average
    wma,                       # Weighted Moving Average
    hullma,                    # Hull Moving Average
    hullma_trend,              # Trend direction from HMA crossover
    hullma_pullback,           # Pullback detection near HMA
    hullma_bounce,             # Bounce (reversal) detection
    # Classic Ichimoku
    ichimoku_line,             # Single Ichimoku line (midpoint)
    ichimoku_components,       # All 4 classic cloud components
    # Enhanced Ichimoku (Hull MA)
    ichimoku_line_hull,        # Hull-smoothed Ichimoku line
    ichimoku_components_hull,  # All 4 Hull-enhanced cloud components
    # Supplementary
    ema,                       # Exponential Moving Average
    atr,                       # Average True Range
)
```

---

## API Reference

### Hull Moving Average

#### `wma`

Weighted Moving Average with linear weights.

```python
wma(
    prices: np.ndarray,  # (N,) float64 — price series
    period: int,         # window size
) -> np.ndarray          # (N,) float64 — NaN for first period-1 positions
```

Weight scheme: most recent bar gets weight `period`, oldest gets weight `1`. Denominator = `period × (period + 1) / 2`.

---

#### `hullma`

Hull Moving Average — low-lag smoothed moving average.

```python
hullma(
    prices: np.ndarray,  # (N,) float64 — price series
    period: int,         # window size
) -> np.ndarray          # (N,) float64 — NaN for initial positions
```

First valid value appears approximately at position `period + √period`. Reduces lag by ~50% compared to standard moving averages.

---

#### `hullma_trend`

Determine trend direction from Hull MA crossover.

```python
hullma_trend(
    hullma_short: np.ndarray,  # (N,) float64 — short-period HMA
    hullma_long: np.ndarray,   # (N,) float64 — long-period HMA
) -> int                       # 1=uptrend, -1=downtrend, 0=neutral
```

Compares the last value of each series. Returns `0` if either is NaN or arrays are shorter than 2.

---

#### `hullma_pullback`

Detect pullback opportunities relative to Hull MA.

```python
hullma_pullback(
    prices: np.ndarray,        # (N,) float64
    hullma_long: np.ndarray,   # (N,) float64
    threshold: float = 0.03,   # max distance ratio for pullback (3%)
) -> tuple[bool, float]       # (is_pullback, distance_ratio)
```

A pullback is detected when `|price - hullma| / hullma <= threshold`.

---

#### `hullma_bounce`

Detect recent price bounce (reversal momentum).

```python
hullma_bounce(
    prices: np.ndarray,          # (N,) float64
    threshold: float = 0.002,    # minimum return for bounce (0.2%)
) -> tuple[bool, bool, float]   # (is_bounce_up, is_bounce_down, strength)
```

Compares last two bars: `recent_return = prices[-1] / prices[-2] - 1`. Bounce detected when `|return| > threshold`.

---

### Classic Ichimoku

#### `ichimoku_line`

Classic Ichimoku line: rolling `(max_high + min_low) / 2`.

```python
ichimoku_line(
    high: np.ndarray,   # (N,) float64
    low: np.ndarray,    # (N,) float64
    period: int,
) -> np.ndarray         # (N,) float64 — initial positions backfilled
```

Initial positions `0..period-1` are backfilled with the first valid value (not NaN).

---

#### `ichimoku_components`

All four classic Ichimoku cloud components.

```python
ichimoku_components(
    high: np.ndarray,        # (N,) float64
    low: np.ndarray,         # (N,) float64
    tenkan_period: int,      # conversion line period (e.g., 10 or 9)
    kijun_period: int,       # base line period (e.g., 30 or 26)
    senkou_period: int,      # leading span B period (e.g., 60 or 52)
) -> tuple[
    np.ndarray,  # tenkan     — conversion line
    np.ndarray,  # kijun      — base line
    np.ndarray,  # senkou_a   — leading span A = (tenkan + kijun) / 2
    np.ndarray,  # senkou_b   — leading span B
]
```

---

### Enhanced Ichimoku (Hull MA)

#### `ichimoku_line_hull`

Hull-enhanced Ichimoku line: Hull MA applied to `(high + low) / 2`.

```python
ichimoku_line_hull(
    high: np.ndarray,   # (N,) float64
    low: np.ndarray,    # (N,) float64
    period: int,
) -> np.ndarray         # (N,) float64 — NaN for initial positions
```

Replaces classic midpoint with Hull MA smoothing. Returns NaN (not zeros) for initial positions.

---

#### `ichimoku_components_hull`

All four Hull-enhanced Ichimoku cloud components.

```python
ichimoku_components_hull(
    high: np.ndarray,        # (N,) float64
    low: np.ndarray,         # (N,) float64
    tenkan_period: int,      # e.g., 10
    kijun_period: int,       # e.g., 30
    senkou_period: int,      # e.g., 60
) -> tuple[
    np.ndarray,  # tenkan     — Hull-smoothed conversion line
    np.ndarray,  # kijun      — Hull-smoothed base line
    np.ndarray,  # senkou_a   — NaN-aware (tenkan + kijun) / 2
    np.ndarray,  # senkou_b   — Hull-smoothed leading span B
]
```

`senkou_a` is NaN at any position where either `tenkan` or `kijun` is NaN.

---

### Supplementary Indicators

#### `ema`

Exponential Moving Average.

```python
ema(
    data: np.ndarray,   # (N,) float64
    period: int,
) -> np.ndarray         # (N,) float64 — no NaN, fully populated
```

`alpha = 2 / (period + 1)`. Seeded with `data[0]`, then recursive.

---

#### `atr`

Average True Range with Wilder's smoothing.

```python
atr(
    high: np.ndarray,       # (N,) float64
    low: np.ndarray,        # (N,) float64
    close: np.ndarray,      # (N,) float64
    period: int = 14,
) -> np.ndarray             # (N,) float64 — zeros for 0..period-2
```

True Range uses the standard 3-component formula. ATR uses Wilder's exponential smoothing: `ATR[i] = ((period-1) × ATR[i-1] + TR[i]) / period`.

---

## Application Domains

This library is a general-purpose **technical analysis toolkit**. The Hull-enhanced Ichimoku components are useful anywhere you need trend identification with reduced lag:

**Algorithmic trading**
- Real-time trend detection with sub-bar latency
- Cloud-based support/resistance zone identification
- Hull MA crossover signal generation

**Quantitative research**
- Backtesting Ichimoku strategies across different parameter sets
- Comparing classic vs Hull-enhanced signal timing
- Multi-timeframe cloud analysis

**Portfolio management**
- Regime identification (trending vs ranging) via cloud structure
- Adaptive position sizing based on ATR
- Trend confirmation via Hull MA alignment

**Market microstructure**
- High-frequency indicator computation without JIT warm-up
- Low-latency signal generation for order routing
- Concurrent indicator calculation (GIL-free)

---

## Project Structure

```
advanced-ichimoku-cloud/
├── Cargo.toml              # Rust dependencies
├── pyproject.toml           # maturin build configuration
├── README.md
├── .github/workflows/CI.yml # Cross-platform wheel builds
├── src/
│   ├── lib.rs              # PyO3 module — registers all 11 functions
│   ├── hull.rs             # wma + hullma (+ inner functions for reuse)
│   ├── hull_signals.rs     # hullma_trend, hullma_pullback, hullma_bounce
│   ├── ichimoku.rs         # Classic Ichimoku line + components
│   ├── ichimoku_hull.rs    # Hull-enhanced Ichimoku (uses hull::hullma_inner)
│   └── indicators.rs       # ema, atr
└── tests/
    └── test_parity.py      # 11 reference impls, 25+ assertions, atol=1e-10
```

---

## Design Decisions

| Decision | Rationale |
|---|---|
| **`pub(crate) fn *_inner()`** for computation | No PyO3 overhead for cross-module reuse (e.g., `ichimoku_hull.rs` calls `hull::hullma_inner`) |
| **NaN for Hull MA initial positions** | Honest signal — no data means no value. Classic Ichimoku uses backfill for historical compatibility. |
| **Zeros + backfill for classic Ichimoku** | Matches the original Numba implementation exactly. Initial bars carry the first valid equilibrium value. |
| **No rayon parallelism** | All 11 functions are sequential by nature. Thread pool overhead would exceed computation time. |
| **`as_slice().unwrap()`** for input arrays | Requires contiguous C-order input (the default for NumPy). Panics on Fortran-order — acceptable since NumPy defaults to C. |
| **`PyArray1::from_vec`** for output | Allocates once in Rust, transfers ownership to Python. Zero-copy on the return path. |
| **Configurable periods** (not hardcoded) | Classic (9/26/52) and enhanced (10/30/60) are both valid. Users should choose based on their market and timeframe. |

---

## NaN Handling

| Function | Behavior |
|---|---|
| `wma` | NaN for positions `0..period-1` |
| `hullma` | NaN propagates through WMA chain; first valid at approximately `period + √period` |
| `ichimoku_line` | Zeros initially, backfilled with first valid value |
| `ichimoku_line_hull` | NaN from Hull MA (different from classic) |
| `ichimoku_components` | Inherits from `ichimoku_line` (backfilled) |
| `ichimoku_components_hull` | NaN where any input component is NaN |
| `ema` | No NaN — `ema[0] = data[0]`, recursive from there |
| `atr` | Zeros for `0..period-2`, first valid at `period-1` |
| `hullma_trend` | Returns `0` if either input has NaN at last position |
| `hullma_pullback` | Returns `(false, 0.0)` if NaN or non-positive HMA |
| `hullma_bounce` | Returns `(false, false, 0.0)` if fewer than 2 prices |

---

## Rust Dependency Stack

| Crate | Version | Role |
|---|---|---|
| `pyo3` | 0.27 | Python-Rust bindings, GIL management |
| `numpy` | 0.27 | Zero-copy NumPy ndarray interop |
| `ndarray` | 0.16 | Owned N-dimensional array construction |

Build tool: **maturin** 1.11+ (compiles Rust into `.so`, installs into virtualenv).

---

## Verification

The test suite (`tests/test_parity.py`) runs **25+ assertions** across all 11 functions:

| Test | What it validates |
|---|---|
| `test_wma` | 3 periods (5, 10, 20) — weights, NaN positions |
| `test_hullma` | 3 periods (10, 20, 30) — full 4-step pipeline |
| `test_hullma_trend` | Crossover direction using short/long HMA |
| `test_hullma_pullback` | Distance ratio + boolean threshold |
| `test_hullma_bounce` | Up/down detection + bounce strength |
| `test_ichimoku_line` | 3 periods — midpoint + backfill |
| `test_ichimoku_components` | All 4 components (tenkan, kijun, senkou_a, senkou_b) |
| `test_ichimoku_line_hull` | 3 periods — HMA of median price |
| `test_ichimoku_components_hull` | All 4 Hull-enhanced components + NaN-aware senkou_a |
| `test_ema` | 3 periods (3, 14, 30) — recursive smoothing |
| `test_atr` | 2 periods (14, 20) — TR + Wilder's smoothing |

All tests use `seed=42`, `N=200` bars, `atol=1e-12`.

---

## Building a Wheel for Distribution

```bash
# Build a portable wheel
maturin build --release

# Output: target/wheels/advanced_ichimoku_cloud-0.1.0-cp3XX-...-linux_x86_64.whl

# Install anywhere:
pip install target/wheels/advanced_ichimoku_cloud-*.whl
```

---

## Topics

`ichimoku-cloud` `hull-moving-average` `technical-analysis` `trading-indicators` `ichimoku-kinko-hyo` `weighted-moving-average` `exponential-moving-average` `average-true-range` `rust` `pyo3` `numpy` `high-performance-computing` `quantitative-finance` `algorithmic-trading`
