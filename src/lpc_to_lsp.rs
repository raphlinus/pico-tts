//! Conversion between LPC coefficients and line spectral pairs.
//!
//! AI disclosure: this code was mostly written by Gemini 3 thinking.
//!
//! Based on the Kabal and Ramachandran (1986) method for
//! computing Line Spectral Frequencies via Chebyshev polynomials.

use std::f64::consts::PI;

/// Converts Reflection Coefficients (PARCOR) to LPC coefficients.
///
/// k: reflection coefficients [k1, k2, ..., kM]
/// Returns: LPC coefficients [1.0, a1, a2, ..., aM]
pub fn parcor_to_lpc(k: &[f64]) -> Vec<f64> {
    let m = k.len();
    let mut a = vec![0.0; m + 1];
    let mut a_prev = vec![0.0; m + 1];

    a[0] = 1.0;
    a_prev[0] = 1.0;

    for i in 1..=m {
        let ki = k[i - 1];
        a[i] = ki;
        for j in 1..i {
            a[j] = a_prev[j] + ki * a_prev[i - j];
        }
        a_prev.copy_from_slice(&a);
    }
    a
}

/// Converts LPC coefficients to Line Spectral Pairs (LSPs) in radians [0, PI].
///
/// a: LPC coefficients [1.0, a1, ..., aM]
pub fn lpc_to_lsp(a: &[f64], nb_points: usize) -> Vec<f64> {
    let m = a.len() - 1; // Order (e.g., 10)
    let n = m / 2; // Number of root pairs

    // Create the reduced polynomials P' and Q'
    // This removes the trivial roots at z = +/- 1
    let mut p = vec![0.0; n + 1];
    let mut q = vec![0.0; n + 1];

    p[0] = 1.0;
    q[0] = 1.0;
    for i in 1..=n {
        p[i] = (a[i] + a[m + 1 - i]) - p[i - 1];
        q[i] = (a[i] - a[m + 1 - i]) + q[i - 1];
    }
    p[n] *= 0.5;
    q[n] *= 0.5;

    let mut lsps = Vec::with_capacity(m);
    let mut x_prev = 1.0;
    let mut current_poly = &p;
    let mut is_p = true;
    let mut y_prev = evaluate_chebyshev_poly(x_prev, current_poly);

    // Grid search
    for i in 1..=nb_points {
        let x_curr = (PI * i as f64 / nb_points as f64).cos();
        let y_curr = evaluate_chebyshev_poly(x_curr, current_poly);

        if y_curr * y_prev <= 0.0 {
            // Refine with 4 steps of bisection
            let mut low = x_curr;
            let mut high = x_prev;
            for _ in 0..4 {
                let mid = (low + high) * 0.5;
                if evaluate_chebyshev_poly(mid, current_poly)
                    * evaluate_chebyshev_poly(low, current_poly)
                    <= 0.0
                {
                    high = mid;
                } else {
                    low = mid;
                }
            }
            lsps.push(((low + high) * 0.5).acos());

            if lsps.len() >= m {
                break;
            }

            // Switch P <-> Q
            is_p = !is_p;
            current_poly = if is_p { &p } else { &q };
            y_prev = evaluate_chebyshev_poly(x_curr, current_poly);
        } else {
            y_prev = y_curr;
        }
        x_prev = x_curr;
    }
    lsps
}

fn evaluate_chebyshev_poly(x: f64, c: &[f64]) -> f64 {
    let n = c.len() - 1;
    let mut d1 = 0.0;
    let mut d2 = 0.0;
    let x2 = 2.0 * x;
    for i in 0..n {
        (d1, d2) = (x2 * d1 - d2 + c[i], d1);
    }
    x * d1 - d2 + c[n]
}

/// Converts Line Spectral Pairs (LSPs) back to LPC coefficients.
///
/// lsps: LSP frequencies in radians [0, PI], expected length M.
/// Returns: LPC coefficients [1.0, a1, a2, ..., aM].
pub fn lsp_to_lpc(lsps: &[f64]) -> Vec<f64> {
    let m = lsps.len();
    let mut p = vec![0.0; m + 1];
    let mut q = vec![0.0; m + 1];

    // Initial conditions for the product series
    p[0] = 1.0;
    q[0] = 1.0;

    // 1. Reconstruct P and Q by multiplying quadratic factors
    // We iterate through LSPs in pairs (P root at i, Q root at i+1)
    for i in (0..m).step_by(2) {
        let gp = -2.0 * lsps[i].cos(); // P root factor
        let gq = -2.0 * lsps[i + 1].cos(); // Q root factor

        // Convolve the current polynomial with (1 + g*z^-1 + z^-2)
        // We go backwards to update in-place safely
        for j in (1..=(i + 2)).rev() {
            // Update P
            let p_prev_1 = if j >= 1 { p[j - 1] } else { 0.0 };
            let p_prev_2 = if j >= 2 { p[j - 2] } else { 0.0 };
            p[j] = p[j] + gp * p_prev_1 + p_prev_2;

            // Update Q
            let q_prev_1 = if j >= 1 { q[j - 1] } else { 0.0 };
            let q_prev_2 = if j >= 2 { q[j - 2] } else { 0.0 };
            q[j] = q[j] + gq * q_prev_1 + q_prev_2;
        }
    }

    // 2. Multiply P by (1 + z^-1) and Q by (1 - z^-1)
    for j in (1..=m).rev() {
        p[j] += p[j - 1];
        q[j] -= q[j - 1];
    }

    // 3. Average P and Q to get the final LPC coefficients: A(z) = (P(z) + Q(z)) / 2
    let mut a = vec![0.0; m + 1];
    a[0] = 1.0;
    for i in 1..=m {
        a[i] = 0.5 * (p[i] + q[i]);
    }

    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_roundtrip() {
        // 1. Define stable PARCOR coefficients (all must be in range (-1, 1))
        let k_orig = vec![
            -0.6, 0.45, -0.21, 0.12, -0.05, 0.18, -0.1, 0.02, -0.08, 0.04,
        ];

        // 2. PARCOR -> LPC
        let lpc_orig = parcor_to_lpc(&k_orig);

        // 3. LPC -> LSP
        let lsps = lpc_to_lsp(&lpc_orig, 1024);

        // Verify we found all 10 roots
        assert_eq!(lsps.len(), k_orig.len(), "Failed to find all LSP roots");

        // 4. LSP -> LPC (Roundtrip)
        let lpc_reconstructed = lsp_to_lpc(&lsps);

        // 5. Comparison
        println!("Original LPC:      {:?}", lpc_orig);
        println!("Reconstructed LPC: {:?}", lpc_reconstructed);

        let epsilon = 1e-3;
        for i in 0..lpc_orig.len() {
            let diff = (lpc_orig[i] - lpc_reconstructed[i]).abs();
            assert!(diff < epsilon, "LPC coefficient {} diverged by {}", i, diff);
        }
    }
}
