use crate::{
    phonemes::{Kind, Phoneme},
    synth::{Params, Synth},
};

pub struct Sequence<T: AsRef<[&'static Phoneme]>> {
    seq: T,
    ix: usize,
    env: Env,
    env_ix: usize,
    synth: Synth,
}

struct Env {
    attack_len: usize,
    decay_len: usize,
    sustain_level: f64,
    sustain_len: usize,
    release_len: usize,
}

impl Env {
    fn get(&self, mut ix: usize) -> f64 {
        if ix < self.attack_len {
            return ix as f64 / self.attack_len as f64;
        }
        ix -= self.attack_len;
        if ix < self.decay_len {
            return 1.0 - (1.0 - self.sustain_level) * ix as f64 / self.decay_len as f64;
        }
        ix -= self.decay_len;
        if ix < self.sustain_len {
            return self.sustain_level;
        }
        ix -= self.sustain_len;
        if ix < self.decay_len {
            return self.sustain_level * (1.0 - ix as f64 / self.decay_len as f64);
        }
        0.0
    }

    fn len(&self) -> usize {
        self.attack_len + self.decay_len + self.sustain_len + self.release_len
    }
}

const VOLUME: f64 = 5e-4;
const BLEND_LEN: usize = 1500;

impl<T: AsRef<[&'static Phoneme]>> Sequence<T> {
    pub fn new(seq: T) -> Self {
        Self {
            seq,
            ix: 0,
            env: Env {
                attack_len: 500,
                decay_len: 100,
                sustain_level: 0.9,
                sustain_len: 1000,
                release_len: 500,
            },
            env_ix: 0,
            synth: Synth::new(18),
        }
    }

    pub fn get(&mut self) -> Option<f64> {
        let seq = self.seq.as_ref();
        if self.ix == seq.len() {
            return None;
        }
        let phoneme = &seq[self.ix];
        let k = phoneme.ks.to_vec();
        let period = if phoneme.voiced { 140 } else { 0 };
        let rms = phoneme.rms * VOLUME;
        let params = Params { k, period, rms };
        if self.env_ix == 0 {
            self.env = self.env_for_phoneme(phoneme);
        }
        if let Some(next) = seq.get(self.ix + 1) {
            let blend = phoneme.kind.blends(next.kind);
            if blend > 0.0 && self.env_ix >= 600 {
                let next_k = next.ks.to_vec();
                let next_rms = next.rms * VOLUME;
                let next_params = Params {
                    k: next_k,
                    period,
                    rms: next_rms,
                };
                let t = (self.env_ix - 600) as f64 / BLEND_LEN as f64;
                let t = (0.5 + (t - 0.5) / blend).clamp(0., 1.);
                let blend_params = params.lerp(&next_params, t);
                let y = self.synth.get_sample(&blend_params);
                self.env_ix += 1;
                if self.env_ix == 600 + BLEND_LEN {
                    self.ix += 1;
                    self.env_ix = 600;
                }
                return Some(y * 0.9);
            }
        }
        // TODO: don't allocate here
        let y = self.synth.get_sample(&params);
        let env_level = self.env.get(self.env_ix);
        self.env_ix += 1;
        if self.env_ix == self.env.len() {
            self.ix += 1;
            self.env_ix = 0;
        }
        Some(y * env_level)
    }

    fn env_for_phoneme(&self, phoneme: &Phoneme) -> Env {
        match phoneme.kind {
            Kind::Plosive => Env {
                attack_len: 160,
                decay_len: 160,
                sustain_level: 0.3,
                sustain_len: 1,
                release_len: 320,
            },
            _ => Env {
                attack_len: 500,
                decay_len: 100,
                sustain_level: 0.9,
                sustain_len: 1000,
                release_len: 500,
            },
        }
    }
}
