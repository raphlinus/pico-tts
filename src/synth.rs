const EMPH: f64 = 0.9375;

pub struct Synth {
    y: f64,
    x: Vec<f64>,
    phase: usize,
}

pub struct Params {
    pub k: Vec<f64>,
    pub period: usize,
}

impl Synth {
    pub fn new(n: usize) -> Self {
        Self {
            y: 0.0,
            x: vec![0.0; n],
            phase: 0,
        }
    }

    pub fn get_sample(&mut self, params: &Params) -> f64 {
        if self.phase == 0 {
            self.y += 1.0 / EMPH;
        }
        let mut u = self.y;
        self.y *= EMPH;
        self.phase += 1;
        if self.phase == params.period {
            self.phase = 0;
        }
        let n = params.k.len();
        for i in (0..n).rev() {
            u -= params.k[i] * self.x[i];
            self.x[i + 1] = self.x[i] + params.k[i] * u;
        }
        self.x[0] = u;
        u
    }
}
