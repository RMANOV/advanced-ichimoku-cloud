"""
Parity tests: verify Rust implementations match pure-Python reference algorithms.
Each of the 11 exported functions is tested against its reference implementation.
"""

import numpy as np
import sys

# ── Reference Implementations ────────────────────────────────────────────────

def ref_wma(prices, period):
    n = len(prices)
    if n < period:
        return np.full(n, np.nan)
    wma = np.empty(n)
    wma[:period - 1] = np.nan
    weight_sum = period * (period + 1) / 2.0
    for i in range(period - 1, n):
        weighted_sum = 0.0
        for j in range(period):
            weight = period - j
            weighted_sum += weight * prices[i - j]
        wma[i] = weighted_sum / weight_sum
    return wma


def ref_hullma(prices, period):
    n = len(prices)
    if n < period:
        return np.full(n, np.nan)
    half_period = max(2, period // 2)
    wma_half = ref_wma(prices, half_period)
    wma_full = ref_wma(prices, period)
    raw_hma = np.empty(n)
    for i in range(n):
        if np.isnan(wma_half[i]) or np.isnan(wma_full[i]):
            raw_hma[i] = np.nan
        else:
            raw_hma[i] = 2.0 * wma_half[i] - wma_full[i]
    sqrt_period = max(2, int(np.sqrt(period)))
    return ref_wma(raw_hma, sqrt_period)


def ref_hullma_trend(hullma_short, hullma_long):
    if len(hullma_short) < 2 or len(hullma_long) < 2:
        return 0
    cs = hullma_short[-1]
    cl = hullma_long[-1]
    if np.isnan(cs) or np.isnan(cl):
        return 0
    if cs > cl:
        return 1
    elif cs < cl:
        return -1
    return 0


def ref_hullma_pullback(prices, hullma_long, threshold=0.03):
    if len(prices) == 0 or len(hullma_long) == 0:
        return False, 0.0
    cp = prices[-1]
    ch = hullma_long[-1]
    if np.isnan(cp) or np.isnan(ch) or ch <= 0:
        return False, 0.0
    dr = abs(cp - ch) / ch
    return dr <= threshold, dr


def ref_hullma_bounce(prices, threshold=0.002):
    if len(prices) < 2:
        return False, False, 0.0
    ret = prices[-1] / prices[-2] - 1.0
    strength = abs(ret)
    return ret > threshold, ret < -threshold, strength


def ref_ichimoku_line(high, low, period):
    n = len(high)
    result = np.zeros(n)
    if n < period:
        return result
    for i in range(period - 1, n):
        max_high = high[i - period + 1]
        min_low = low[i - period + 1]
        for j in range(i - period + 2, i + 1):
            if high[j] > max_high:
                max_high = high[j]
            if low[j] < min_low:
                min_low = low[j]
        result[i] = (max_high + min_low) / 2
    if period - 1 < n and period > 1:
        initial = result[period - 1]
        result[:period - 1] = initial
    return result


def ref_ichimoku_components(high, low, tp, kp, sp):
    tenkan = ref_ichimoku_line(high, low, tp)
    kijun = ref_ichimoku_line(high, low, kp)
    senkou_a = (tenkan + kijun) / 2
    senkou_b = ref_ichimoku_line(high, low, sp)
    return tenkan, kijun, senkou_a, senkou_b


def ref_ichimoku_line_hull(high, low, period):
    hl_median = (np.array(high) + np.array(low)) / 2
    return ref_hullma(hl_median, period)


def ref_ichimoku_components_hull(high, low, tp, kp, sp):
    tenkan = ref_ichimoku_line_hull(high, low, tp)
    kijun = ref_ichimoku_line_hull(high, low, kp)
    n = len(high)
    senkou_a = np.full(n, np.nan)
    for i in range(n):
        if not np.isnan(tenkan[i]) and not np.isnan(kijun[i]):
            senkou_a[i] = (tenkan[i] + kijun[i]) / 2
    senkou_b = ref_ichimoku_line_hull(high, low, sp)
    return tenkan, kijun, senkou_a, senkou_b


def ref_ema(data, period):
    alpha = 2.0 / (period + 1)
    ema = np.zeros_like(data, dtype=np.float64)
    ema[0] = data[0]
    for i in range(1, len(data)):
        ema[i] = alpha * data[i] + (1 - alpha) * ema[i - 1]
    return ema


def ref_atr(high, low, close, period=14):
    n = len(high)
    tr = np.zeros(n)
    atr = np.zeros(n)
    tr[0] = high[0] - low[0]
    for i in range(1, n):
        hl = high[i] - low[i]
        hpc = abs(high[i] - close[i - 1])
        lpc = abs(low[i] - close[i - 1])
        tr[i] = max(hl, hpc, lpc)
    if period <= n:
        atr[period - 1] = np.mean(tr[:period])
        for i in range(period, n):
            atr[i] = ((period - 1) * atr[i - 1] + tr[i]) / period
    return atr


# ── Test Helpers ─────────────────────────────────────────────────────────────

def assert_close(name, rust, ref, rtol=1e-10, atol=1e-12):
    """Compare arrays handling NaN positions."""
    rust = np.asarray(rust)
    ref = np.asarray(ref)
    assert rust.shape == ref.shape, f"{name}: shape mismatch {rust.shape} vs {ref.shape}"
    nan_mask_rust = np.isnan(rust)
    nan_mask_ref = np.isnan(ref)
    assert np.array_equal(nan_mask_rust, nan_mask_ref), \
        f"{name}: NaN positions differ\nRust NaN at: {np.where(nan_mask_rust)[0]}\nRef  NaN at: {np.where(nan_mask_ref)[0]}"
    valid = ~nan_mask_rust
    if valid.any():
        np.testing.assert_allclose(rust[valid], ref[valid], rtol=rtol, atol=atol,
                                   err_msg=f"{name}: values differ")


def passed(name):
    print(f"  PASS  {name}")


# ── Generate Test Data ───────────────────────────────────────────────────────

np.random.seed(42)
N = 200
close = 100 + np.cumsum(np.random.randn(N) * 0.5)
high = close + np.abs(np.random.randn(N) * 0.3)
low = close - np.abs(np.random.randn(N) * 0.3)

# ── Run Tests ────────────────────────────────────────────────────────────────

import advance_ichimoku_cloud as ic

failures = 0

def run_test(name, fn):
    global failures
    try:
        fn()
        passed(name)
    except Exception as e:
        print(f"  FAIL  {name}: {e}")
        failures += 1


print("=" * 60)
print("  Parity Tests: advance-ichimoku-cloud")
print("=" * 60)

# 1. WMA
def test_wma():
    for period in [5, 10, 20]:
        r = np.array(ic.wma(close, period))
        p = ref_wma(close, period)
        assert_close(f"wma(period={period})", r, p)
run_test("wma", test_wma)

# 2. HullMA
def test_hullma():
    for period in [10, 20, 30]:
        r = np.array(ic.hullma(close, period))
        p = ref_hullma(close, period)
        assert_close(f"hullma(period={period})", r, p)
run_test("hullma", test_hullma)

# 3. HullMA Trend
def test_hullma_trend():
    short = ic.hullma(close, 20)
    long = ic.hullma(close, 60)
    r = ic.hullma_trend(short, long)
    p = ref_hullma_trend(np.array(short), np.array(long))
    assert r == p, f"hullma_trend: {r} vs {p}"
run_test("hullma_trend", test_hullma_trend)

# 4. HullMA Pullback
def test_hullma_pullback():
    hlong = ic.hullma(close, 60)
    r_pb, r_dr = ic.hullma_pullback(close, hlong, 0.03)
    p_pb, p_dr = ref_hullma_pullback(close, np.array(hlong), 0.03)
    assert r_pb == p_pb, f"pullback flag: {r_pb} vs {p_pb}"
    np.testing.assert_allclose(r_dr, p_dr, rtol=1e-10)
run_test("hullma_pullback", test_hullma_pullback)

# 5. HullMA Bounce
def test_hullma_bounce():
    r_up, r_dn, r_str = ic.hullma_bounce(close, 0.002)
    p_up, p_dn, p_str = ref_hullma_bounce(close, 0.002)
    assert r_up == p_up, f"bounce_up: {r_up} vs {p_up}"
    assert r_dn == p_dn, f"bounce_dn: {r_dn} vs {p_dn}"
    np.testing.assert_allclose(r_str, p_str, rtol=1e-10)
run_test("hullma_bounce", test_hullma_bounce)

# 6. Ichimoku Line
def test_ichimoku_line():
    for period in [10, 30, 60]:
        r = np.array(ic.ichimoku_line(high, low, period))
        p = ref_ichimoku_line(high, low, period)
        assert_close(f"ichimoku_line(period={period})", r, p)
run_test("ichimoku_line", test_ichimoku_line)

# 7. Ichimoku Components
def test_ichimoku_components():
    rt, rk, rsa, rsb = ic.ichimoku_components(high, low, 10, 30, 60)
    pt, pk, psa, psb = ref_ichimoku_components(high, low, 10, 30, 60)
    assert_close("ichimoku tenkan", np.array(rt), pt)
    assert_close("ichimoku kijun", np.array(rk), pk)
    assert_close("ichimoku senkou_a", np.array(rsa), psa)
    assert_close("ichimoku senkou_b", np.array(rsb), psb)
run_test("ichimoku_components", test_ichimoku_components)

# 8. Ichimoku Line Hull
def test_ichimoku_line_hull():
    for period in [10, 30, 60]:
        r = np.array(ic.ichimoku_line_hull(high, low, period))
        p = ref_ichimoku_line_hull(high, low, period)
        assert_close(f"ichimoku_line_hull(period={period})", r, p)
run_test("ichimoku_line_hull", test_ichimoku_line_hull)

# 9. Ichimoku Components Hull
def test_ichimoku_components_hull():
    rt, rk, rsa, rsb = ic.ichimoku_components_hull(high, low, 10, 30, 60)
    pt, pk, psa, psb = ref_ichimoku_components_hull(high, low, 10, 30, 60)
    assert_close("hull tenkan", np.array(rt), pt)
    assert_close("hull kijun", np.array(rk), pk)
    assert_close("hull senkou_a", np.array(rsa), psa)
    assert_close("hull senkou_b", np.array(rsb), psb)
run_test("ichimoku_components_hull", test_ichimoku_components_hull)

# 10. EMA
def test_ema():
    for period in [3, 14, 30]:
        r = np.array(ic.ema(close, period))
        p = ref_ema(close, period)
        assert_close(f"ema(period={period})", r, p)
run_test("ema", test_ema)

# 11. ATR
def test_atr():
    for period in [14, 20]:
        r = np.array(ic.atr(high, low, close, period))
        p = ref_atr(high, low, close, period)
        assert_close(f"atr(period={period})", r, p)
run_test("atr", test_atr)

# ── Summary ──────────────────────────────────────────────────────────────────

print("=" * 60)
if failures == 0:
    print(f"  ALL 11 FUNCTIONS PASS PARITY TESTS")
else:
    print(f"  {failures} FUNCTION(S) FAILED")
print("=" * 60)

sys.exit(failures)
