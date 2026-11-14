//! An implementation of the Klatt synthesizer
//!
//! Generally follows [Klatt 80] with some tweaks.
//! [Klatt 80]: https://www.fon.hum.uva.nl/david/ma_ssp/doc/Klatt-1980-JAS000971.pdf

const N_FORMANTS: usize = 5;

/// Calibration of the cascade-branch sources (voicing and aspiration)
/// relative to the parallel-branch frication, in dB.
///
/// With the NDBSCA scale factors alone, a vowel at AV=60 comes out about
/// 40 dB weaker than /s/ at AF=60, which is far from natural balance. The
/// MITalk tables assume Klatt's own calibration, so this compensates. It
/// was previously applied as `av += 40` in the callers. The true source of
/// the discrepancy is not yet known. Aspiration, which shares the cascade
/// branch but not RGP, was about as far off (about 60 dB below the vowel at
/// AH=51), so the gap is probably in the cascade branch rather than RGP.
const VOICING_CAL_DB: f32 = 40.0;

/// Calibration of aspiration, which also enters the cascade branch. With
/// none, AH=51 aspiration is about 60 dB below the vowel (inaudible). The
/// voicing value (40) makes HH (AH=60) as loud as the vowel; 30 puts HH
/// about 6 dB and stop aspiration about 16 dB below it, chosen by ear.
const ASP_CAL_DB: f32 = 30.0;

/// Calibration of the parallel bypass path (AB), used for the flat-spectrum
/// frication of FF, TH, DH, VV and the labial bursts. With none, FF came
/// out about 37 dB below the vowel and sounded almost silent; 15 puts it
/// about 22 dB below. Chosen by ear.
const AB_CAL_DB: f32 = 15.0;

#[derive(Default)]
pub struct Klatt {
    rgp: Resonator,
    rgz: AntiResonator,
    /// RGP for the quasi-sinusoidal voicing (AVS) branch
    rgp_s: Resonator,
    /// Low-pass resonator for quasi-sinusoidal voicing
    rgs: Resonator,
    rnp: Resonator,
    rnz: AntiResonator,
    cascade: [Resonator; N_FORMANTS],
    // r2 through r6
    par: [Resonator; 5],
    apar: [f32; 5],
    /// Amplitude of impulse
    impuls: f32,
    /// Amplitude of impulse for quasi-sinusoidal voicing
    impuls_s: f32,
    pulse_period: usize,
    pulse_phase: usize,
    /// Impulse amplitudes latched at the start of the current pulse, so that
    /// the second (negative) half of the doublet matches the first even when
    /// the parameters change between the two samples. Otherwise the doublet
    /// leaves a DC step, which the low-pass AVS branch turns into a thump.
    pulse_amp: (f32, f32),
    noise: Noise,
    aaspir: f32,
    afric: f32,
    abpar: f32,
}

pub struct Noise {
    x: u32,
}

/// Parameters for Klatt synthesis.
///
/// These closely follow Table I in Klatt 80.
///
/// Potentially this could be cleaned up. Some parameters (like sampling rate)
/// could be moved out, and the order also is irregular.
#[expect(unused, reason = "not all parameters are wired up yet")]
#[derive(Clone)]
pub struct KlattParams {
    /// Amplitude of voicing (dB)
    pub av: f32,
    /// Amplitude of frication (dB)
    pub af: f32,
    /// Amplitude of aspiration (dB)
    pub ah: f32,
    /// Amplitude of sinusoidal voicing (dB)
    pub avs: f32,
    /// Fundamental frequency of voicing (Hz)
    pub f0: f32,
    /// First formant frequency (Hz)
    pub f1: f32,
    /// Second formant frequency (Hz)
    pub f2: f32,
    /// Third formant frequency (Hz)
    pub f3: f32,
    /// Fourth formant frequency (Hz)
    pub f4: f32,
    /// Nasal zero frequency (Hz)
    pub fnz: f32,
    /// Nasal formant amplitude (dB)
    pub an: f32,
    /// First formant amplitude (dB)
    pub a1: f32,
    /// Second formant amplitude (dB)
    pub a2: f32,
    /// Third formant amplitude (dB)
    pub a3: f32,
    /// Fourth formant amplitude (dB)
    pub a4: f32,
    /// Fifth formant amplitude (dB)
    pub a5: f32,
    /// Sixth formant amplitude (dB)
    pub a6: f32,
    /// Bypass path amplitude (dB)
    pub ab: f32,
    /// First formant bandwidth (Hz)
    pub b1: f32,
    /// Second formant bandwidth (Hz)
    pub b2: f32,
    /// Third formant bandwidth (Hz)
    pub b3: f32,
    /// Cascade/parallel switch (true is parallel)
    pub sw: bool,
    /// Glottal resonator 1 frequency (Hz)
    pub fgp: f32,
    /// Glottal resonator 1 bandwidth (Hz)
    pub bgp: f32,
    /// Glottal zero frequency (Hz)
    pub fgz: f32,
    /// Glottal zero bandwidth (Hz)
    pub bgz: f32,
    /// Fourth formant bandwidth (Hz)
    pub b4: f32,
    /// Fifth formant frequency (Hz)
    pub f5: f32,
    /// Fifth formant bandwidth (Hz)
    pub b5: f32,
    /// Sixth formant frequency (Hz)
    pub f6: f32,
    /// Sixth formant bandwidth (Hz)
    pub b6: f32,
    /// Nasal pole frequency (Hz)
    pub fnp: f32,
    /// Nasal pole bandwidth (Hz)
    pub bnp: f32,
    /// Nasal zero bandwidth (Hz)
    pub bnz: f32,
    /// Glottal resonator 1 bandwidth (Hz)
    pub bgs: f32,
    /// Sampling rate (Hz)
    pub sr: f32,
    /// Number of waveform samples per chunk
    pub nws: usize,
    /// Overall gain control (dB)
    pub g0: f32,
    /// Number of cascaded formants
    pub nfc: usize,
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

    /// Bandwidth and frequency
    fn set(&mut self, f: f32, bw: f32, radians_per_sample: f32) {
        let r = (-0.5 * bw * radians_per_sample).exp();
        self.c = -r * r;
        self.b = 2. * r * (f * radians_per_sample).cos();
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
    fn set(&mut self, f: f32, bw: f32, radians_per_sample: f32) {
        let r = (-0.5 * bw * radians_per_sample).exp();
        let c = -r * r;
        let b = 2. * r * (f * radians_per_sample).cos();
        let a = 1. - b - c;
        self.a = 1. / a;
        self.b = -self.a * b;
        self.c = -self.a * c;
    }
}

impl Klatt {
    pub fn process(&mut self) -> f32 {
        // Impulse train combined with first derivative
        let mut input = 0.0;
        let mut input_s = 0.0;
        if self.pulse_phase >= self.pulse_period {
            self.pulse_phase = 0;
        }
        if self.pulse_phase < 2 {
            if self.pulse_phase == 0 {
                self.pulse_amp = (self.impuls, self.impuls_s);
                input = self.impuls;
                input_s = self.impuls_s;
            } else {
                input = -self.pulse_amp.0;
                input_s = -self.pulse_amp.1;
            }
        }
        let ygp = self.rgp.process(input);
        let ygz = self.rgz.process(ygp);
        // Quasi-sinusoidal voicing: impulses low-passed by RGP and RGS
        let ygs = self.rgs.process(self.rgp_s.process(input_s));
        let mut noise = self.noise.next_pseudogauss();
        if self.pulse_phase * 2 > self.pulse_period {
            noise *= 0.5;
        }
        self.pulse_phase += 1;
        // TODO: linear smoothing of aspiration amplitude
        let uasp = self.aaspir * noise;
        let ufric = self.afric * noise;
        let uglot = ygz + ygs + uasp;
        let mut y = self.rnz.process(self.rnp.process(uglot));
        for res in self.cascade.iter_mut().rev() {
            y = res.process(y);
        }
        let ulipsv = y;
        // parallel resonators; for now we're processing noise only
        let y2p = self.par[0].process(self.apar[0] * ufric);
        let y3p = self.par[1].process(self.apar[1] * ufric);
        let y4p = self.par[2].process(self.apar[2] * ufric);
        let y5p = self.par[3].process(self.apar[3] * ufric);
        let y6p = self.par[4].process(self.apar[4] * ufric);
        let ulipsf = -y2p + y3p - y4p + y5p - y6p - self.abpar * ufric;
        // scaling is arbitrary, probably want to fine-tune
        (ulipsv + ulipsf) * 0.1
    }

    pub fn set(&mut self, params: &KlattParams) {
        let radians_per_sample = 2.0 * core::f32::consts::PI / params.sr;
        // amplitude scale factors are from NDBSCA in Klatt 80
        let av_scale = 72.0 - VOICING_CAL_DB;
        self.impuls = db_to_linear(params.av, av_scale) * params.f0;
        // AVS uses the same scale as AV, so that the two branches have
        // comparable low-frequency gain.
        self.impuls_s = db_to_linear(params.avs, av_scale) * params.f0;
        self.pulse_period = (params.sr / params.f0).round() as usize;
        self.cascade[0].set(params.f1, params.b1, radians_per_sample);
        self.cascade[1].set(params.f2, params.b2, radians_per_sample);
        self.cascade[2].set(params.f3, params.b3, radians_per_sample);
        self.cascade[3].set(params.f4, params.b4, radians_per_sample);
        self.cascade[4].set(params.f5, params.b5, radians_per_sample);
        self.par[0].set(params.f2, params.b2, radians_per_sample);
        self.par[1].set(params.f3, params.b3, radians_per_sample);
        self.par[2].set(params.f4, params.b4, radians_per_sample);
        self.par[3].set(params.f5, params.b5, radians_per_sample);
        self.par[4].set(params.f6, params.b6, radians_per_sample);
        self.rgp.set(params.fgp, params.bgp, radians_per_sample);
        // Source comment says "set gain to constant in mid-frequency region for rgp"
        // self.rgp.a = 0.007;
        self.rgz.set(params.fgz, params.bgz, radians_per_sample);
        self.rgp_s.set(params.fgp, params.bgp, radians_per_sample);
        self.rgs.set(0.0, params.bgs, radians_per_sample);
        self.rnp.set(params.fnp, params.bnp, radians_per_sample);
        self.rnz.set(params.fnz, params.bnz, radians_per_sample);
        self.aaspir = db_to_linear(params.ah, 102.0 - ASP_CAL_DB);
        self.afric = db_to_linear(params.af, 72.0);
        self.apar[0] = db_to_linear(params.a2, 65.0);
        self.apar[1] = db_to_linear(params.a3, 73.0);
        self.apar[2] = db_to_linear(params.a4, 78.0);
        self.apar[3] = db_to_linear(params.a5, 79.0);
        self.apar[4] = db_to_linear(params.a6, 80.0);
        self.abpar = db_to_linear(params.ab, 84.0 - AB_CAL_DB);
    }
}

impl Default for KlattParams {
    // Default values taken from "Typ" column of Table I in Klatt 80.
    fn default() -> Self {
        Self {
            av: 0.0,
            af: 0.0,
            ah: 0.0,
            avs: 0.0,
            f0: 0.0,
            f1: 450.0,
            f2: 1450.0,
            f3: 2450.0,
            f4: 3300.0,
            fnz: 250.0,
            an: 0.0,
            a1: 0.0,
            a2: 0.0,
            a3: 0.0,
            a4: 0.0,
            a5: 0.0,
            a6: 0.0,
            ab: 0.0,
            b1: 50.0,
            b2: 70.0,
            b3: 110.0,
            sw: false,
            fgp: 0.0,
            bgp: 100.0,
            fgz: 1500.0,
            bgz: 6000.0,
            b4: 250.0,
            f5: 3750.0,
            b5: 200.0,
            f6: 4900.0,
            b6: 1000.0,
            fnp: 250.0,
            bnp: 100.0,
            bnz: 100.0,
            bgs: 200.0,
            sr: 10_000.0,
            nws: 50,
            g0: 47.0,
            nfc: 5,
        }
    }
}

/// Convert decibels to linear scale
///
/// Might replace with a lookup table.
fn db_to_linear(db: f32, scale: f32) -> f32 {
    if db <= 0.0 {
        0.0
    } else {
        ((db - scale) * (core::f32::consts::LN_10 / 20.0)).exp()
    }
}

impl Default for Noise {
    fn default() -> Self {
        Self { x: 0x1ebdf0c5 }
    }
}

impl Noise {
    // Classic xorshift algorithm
    fn next_u32(&mut self) -> u32 {
        let mut x = self.x;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.x = x;
        x
    }

    // This is not fast at all but will exactly match Klatt 80
    fn next_pseudogauss(&mut self) -> f32 {
        let mut sum = 0.0;
        for _ in 0..16 {
            sum += (self.next_u32() as i32 as f32) * (1.0 / 4_294_967_296.0);
        }
        sum
    }
}
