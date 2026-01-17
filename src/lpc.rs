const ORDER: usize = 18;

#[derive(Default)]
pub struct Reflector {
    ks: [f64; ORDER],
    rms: f64,
}

fn get_correlation(buf: &[f64], lag: usize) -> f64 {
    let mut sum = 0.0;
    for i in 0..buf.len() - lag {
        sum += buf[i] * buf[i + lag]
    }
    sum
}

fn get_correlations(buf: &[f64]) -> [f64; ORDER + 1] {
    core::array::from_fn(|lag| get_correlation(buf, lag))
}

#[allow(unused)]
pub fn confidence(buf: &[f64], period: usize) -> f64 {
    get_correlation(buf, period) / get_correlation(buf, 0)
}

impl Reflector {
    pub fn new(buf: &[f64]) -> Self {
        let coeffs = get_correlations(buf);
        let mut result = Self::default();
        result.translate_coeffs(&coeffs, buf.len());
        result
    }

    fn translate_coeffs(&mut self, coeffs: &[f64], n_samples: usize) {
        let mut b = [0.0; ORDER];
        let mut d = [0.0; ORDER + 1];
        self.ks[0] = -coeffs[1] / coeffs[0];
        d[0] = coeffs[1];
        d[1] = coeffs[0] + (self.ks[0] * coeffs[1]);

        for i in 1..ORDER {
            let mut y = coeffs[i + 1];
            b[0] = y;
            for j in 0..i {
                b[j + 1] = d[j] + self.ks[j] * y;
                y += self.ks[j] * d[j];
                d[j] = b[j];
            }
            self.ks[i] = -y / d[i];
            d[i + 1] = d[i] + self.ks[i] * y;
            d[i] = b[i];
        }
        self.rms = (d[ORDER] / n_samples as f64).sqrt();
    }

    #[allow(unused)]
    pub fn is_unvoiced(&self) -> bool {
        const UNVOICED_THRESHOLD: f64 = 0.3;
        self.ks[0] > UNVOICED_THRESHOLD
    }

    pub fn ks(&self) -> &[f64] {
        &self.ks
    }

    pub fn rms(&self) -> f64 {
        self.rms
    }
}
