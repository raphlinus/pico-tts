//! An implementation of the Klatt synthesizer

const N_FORMANTS: usize = 5;

#[derive(Default)]
pub struct Klatt {
    rgp: Resonator,
    rgz: AntiResonator,
    cascade: [Resonator; N_FORMANTS],
    /// Amplitude of impulse
    impuls: f32,
    pulse_period: usize,
    pulse_phase: usize,
}

#[derive(Default, Clone, Copy)]
struct Resonator {
    history: [f32; 2],
    a: f32,
    b: f32,
    c: f32,
}

#[derive(Default, Clone, Copy)]
struct AntiResonator {
    history: [f32; 2],
    a: f32,
    b: f32,
    c: f32,
}

impl Resonator {
    fn process(&mut self, inp: f32) -> f32 {
        let y = self.a * inp + self.b * self.history[0] + self.c * self.history[1];
        self.history = [y, self.history[0]];
        y
    }

    /// Bandwidth and frequency in radians per sample
    fn set(&mut self, f: f32, bw: f32) {
        let r = (-0.5 * bw).exp();
        self.c = -r * r;
        self.b = 2. * r * f.cos();
        self.a = 1. - self.b - self.c;
    }
}

impl AntiResonator {
    fn process(&mut self, inp: f32) -> f32 {
        let y = self.a * inp + self.b * self.history[0] + self.c * self.history[1];
        self.history = [inp, self.history[0]];
        y
    }

    /// Bandwidth and frequency in radians per sample
    fn set(&mut self, f: f32, bw: f32) {
        let r = (-0.5 * bw).exp();
        let c = -r * r;
        let b = 2. * r * f.cos();
        let a = 1. - self.b - self.c;
        self.a = 1. / a;
        self.b = -self.a * b;
        self.c = -self.a * c;
    }
}

impl Klatt {
    pub fn process(&mut self) -> f32 {
        // Impulse train combined with first derivative
        let mut input = 0.0;
        if self.pulse_phase < 2 {
            if self.pulse_phase == 0 {
                input = self.impuls;
            } else {
                input = -self.impuls;
            }
        }
        self.pulse_phase += 1;
        if self.pulse_phase >= self.pulse_period {
            self.pulse_phase = 0;
        }
        let ygp = self.rgp.process(input);
        let ygz = self.rgz.process(ygp);
        let mut y = ygz;
        for res in self.cascade.iter_mut().rev() {
            y = res.process(y);
        }
        // TODO: nasal
        y
    }

    pub fn set(&mut self, params: &[f32]) {
        const SAMPLE_RATE: f32 = 10_000.0;
        let radians_per_sample = 2.0 * core::f32::consts::PI / SAMPLE_RATE;
        self.impuls = params[0];
        self.pulse_period = (SAMPLE_RATE / params[1]).round() as usize;
        for i in 0..3 {
            self.cascade[i].set(
                params[2 * i + 2] * radians_per_sample,
                params[2 * i + 3] * radians_per_sample,
            );
        }
        self.cascade[3].set(2500. * radians_per_sample, 250. * radians_per_sample);
        self.cascade[4].set(3750. * radians_per_sample, 200. * radians_per_sample);
        self.rgp.set(0.0, 100.0 * radians_per_sample);
        self.rgz
            .set(1500. * radians_per_sample, 6000.0 * radians_per_sample);
    }
}
