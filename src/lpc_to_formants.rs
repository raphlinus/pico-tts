use std::f64::consts::PI;

use rpoly::rpoly;

pub fn lpc_to_formants(ks: &[f64]) {
    const LEN: usize = 18;
    // Levinson recursion, convert PARCORs to prediction coefficients
    let mut alpha = [0.0; LEN + 1];
    for i in 1..LEN + 1 {
        alpha[i] = ks[i - 1];
        let old_alpha = alpha;
        for j in 1..i {
            alpha[j] += ks[i - 1] * old_alpha[i - j];
        }
    }
    alpha[0] = 1.0;
    println!("alpha {alpha:.3?}");
    let mut formants = vec![];
    if let Ok(roots) = rpoly(&alpha) {
        for root in roots {
            let (re, im) = (root.re, root.im);
            if im > 0.0 {
                let angle = im.atan2(re);
                let f_scale = 16_000.0 / (2.0 * PI);
                let r = im.hypot(re);
                let f = angle * f_scale;
                let bw = -2.0 * r.ln() * f_scale;
                //println!("root {root:.3?} f {f:.3} bw {bw:.3} r {r:.3}");
                const R_THRESH: f64 = 0.9;
                if r > R_THRESH {
                    formants.push((f, bw));
                }
            }
        }
        formants.sort_by_key(|(f, _bw)| f.to_bits());
        for (f, bw) in formants {
            println!("{f:.3} {bw:.3}");
        }
    } else {
        println!("failed to find roots");
    }
}
