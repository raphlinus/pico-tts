const EMPH: f64 = 0.9375;

pub struct Synth {
    y: f64,
    x: Vec<f64>,
    phase: u16,
}

pub struct Params {
    pub k: Vec<f64>,
    pub period: u16,
    pub rms: f64,
}

impl Synth {
    pub fn new(n: usize) -> Self {
        Self {
            y: 0.0,
            x: vec![0.0; n + 1],
            phase: 1,
        }
    }

    pub fn get_sample(&mut self, params: &Params) -> f64 {
        let mut u;
        if params.period > 0 {
            if self.phase == 0 {
                self.y += 1.0 / EMPH;
            }
            u = self.y;
            self.y *= EMPH;
            self.phase += 1;
            if self.phase == params.period {
                self.phase = 0;
            }
        } else {
            self.phase = (self.phase >> 1) ^ if (self.phase & 1) != 0 { 0xb800 } else { 0 };
            u = if (self.phase & 1) != 0 { 1.0 } else { -1.0 };
        }
        u *= params.rms;
        let n = params.k.len();
        for i in (0..n).rev() {
            u -= params.k[i] * self.x[i];
            self.x[i + 1] = self.x[i] + params.k[i] * u;
        }
        self.x[0] = u;
        u
    }
}
