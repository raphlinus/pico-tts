//! Phonetic segments to Klatt parameters.
//!
//! This is modeled on the PHONET module of MITalk (chapter 11 and appendix C
//! of the book). The input is a sequence of segments, each with a duration
//! and F0 target, in the form produced by PROSOD (see figure 9-1). The output
//! is one set of Klatt parameters per 5 ms frame.

use crate::{
    klatt::KlattParams,
    phones::{
        Flags, Phone, nonvocalic_target, phone_duration, vocalic_target, vocalic_target_glide,
    },
};

/// Frame time in ms
pub const FRAME_TIME: u32 = 5;

/// How the av and avc columns of tables C-1 and C-2 map to Klatt voicing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Voicing {
    /// av is AV; avc is the total (linear sum) of AV and AVS, so AVS is the
    /// remainder.
    #[default]
    Derived,
    /// av is AV; avc is AVS (as in mitalk_reborn).
    Direct,
}

impl Voicing {
    /// Returns (AV, AVS) in dB.
    fn map(self, av: f32, avc: f32) -> (f32, f32) {
        match self {
            Voicing::Derived => {
                let avs = if avc > av {
                    let lin =
                        10f32.powf(avc / 20.) - if av > 0. { 10f32.powf(av / 20.) } else { 0. };
                    20. * lin.log10()
                } else {
                    0.
                };
                (av, avs)
            }
            Voicing::Direct => (av, avc),
        }
    }
}

/// Context rules from appendix C that can be switched off for ablation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Rule {
    /// C.5: no backward smoothing of af after a plosive.
    MintimeAf,
    /// C.5: av is DISSMO out of an unvoiced segment, SMODIS into one.
    AvTrantype,
    /// C.5, figure C-1: pre-aspiration before a pause or an unvoiced
    /// consonant after a sonorant.
    Preasp,
    /// C.5: let the 10 ms sonorant-to-voiceless pre-aspiration rule also
    /// match SIL, overriding the 30 ms breathy offset. Off by default.
    PreaspSil,
    /// C.5: frication starts 20 ms early after a sonorant.
    EarlyAf,
    /// C.5: formant transitions out of a stop use Tcobst (table C-6).
    Tcobst,
    /// C.5: amplitude and formant transition types (the final rule pair).
    AmpTrantype,
    /// C.7.1: diphthong transition smoothing (table C-5, figure C-2).
    /// Otherwise diphthongs glide linearly over the whole segment.
    Diphthong,
    /// C.8.1: obstruent target adjustments (devoicing, alveolar plosive
    /// burst spectra, no voicebar after an obstruent).
    Obstruent,
    /// C.8.3.1: plosive burst at the end of the closure (table C-7).
    Burst,
    /// C.6.1: aspiration of a voiceless obstruent intrudes on the following
    /// sonorant.
    Aspiration,
    /// C.6.1, as printed: the aspiration start moves later by Burdur - 5
    /// after a plosive. By default it moves earlier, into the burst, as the
    /// comment ("aspiration starts during plosive burst") and the narrator
    /// code say. Off by default.
    AspBurstLiteral,
}

/// How to read the final ELSE rule of C.5.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum AmpElse {
    /// Only amplitude transitions still at the default become DISSMO, so the
    /// earlier av rules take effect.
    #[default]
    Preserve,
    /// Every amplitude transition becomes DISSMO, overriding the earlier av
    /// rules.
    Literal,
}

/// Units of Tcdiph in table C-5.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum DipUnits {
    /// 0.1 ms, giving half-widths of roughly 10 to 50 ms.
    #[default]
    Tenth,
    /// ms, taking the printed formula literally. Half-widths then usually
    /// exceed the segment, so the glide spans the whole vowel.
    Ms,
    /// ms, with Segdur in the formula in 5 ms frames (so /45 is /225 ms).
    /// This is how the Amiga narrator.device source comments the rule.
    Frames,
    /// What the Amiga narrator.device code actually computes:
    /// Tcdiph * Segdur / 540 ms (the "1 +" term dropped).
    Narrator,
}

/// Options controlling rendering, mostly for ablation.
#[derive(Clone, Debug)]
pub struct Options {
    /// Rules switched off.
    pub disabled: Vec<Rule>,
    /// Rules that are off by default, switched on.
    pub enabled: Vec<Rule>,
    pub amp_else: AmpElse,
    pub dip_units: DipUnits,
    pub voicing: Voicing,
    /// Raise the nasal zero to 450 Hz in nasals (C.4, C.8.4). Otherwise it
    /// stays at 250 Hz, cancelling the nasal pole.
    pub nasal_zero: bool,
    /// Smooth transitions between segments (figure 11-6). Otherwise every
    /// transition is DISCON.
    pub smooth: bool,
    /// Smooth log F0 with Fujisaki's accent-command filter (a critically
    /// damped second-order lowpass, natural frequency in 1/s), run forward
    /// and backward so it adds no delay. Otherwise F0 is piecewise linear
    /// between targets, as in MITalk.
    pub f0_beta: Option<f32>,
}

/// Rules that are off unless enabled, where the literal reading of the book
/// is doubtful.
const DEFAULT_OFF: &[Rule] = &[Rule::PreaspSil, Rule::AspBurstLiteral];

impl Options {
    fn on(&self, rule: Rule) -> bool {
        if DEFAULT_OFF.contains(&rule) {
            self.enabled.contains(&rule)
        } else {
            !self.disabled.contains(&rule)
        }
    }
}

/// C.4
const ASPAMP: f32 = 51.;
/// C.4: default burst amplitude.
const BURAMP: f32 = 57.;

/// Table C-7: default plosive burst duration.
fn burdur(phone: Phone) -> Option<u32> {
    use Phone::*;
    Some(match phone {
        Bb | Pp => 5,
        Ch | Tq | Tt => 15,
        Dd | Dx | Jj => 10,
        Gg | Gp => 20,
        Kk | Kp => 25,
        Mm | Ng | Nn => 0,
        _ => return None,
    })
}

/// Table C-5: diphthong transition parameters (Tcdiph, Tdmid).
fn diphthong_params(phone: Phone) -> Option<(f32, f32)> {
    use Phone::*;
    Some(match phone {
        Aa => (0., 80.),
        Ae => (100., 75.),
        Ah => (0., 0.),
        Ao => (110., 80.),
        Aw => (120., 70.),
        Axr => (125., 40.),
        Ay => (100., 55.),
        Eh => (60., 70.),
        Er => (180., 50.),
        Exr => (100., 50.),
        Ey => (140., 55.),
        Ih => (90., 65.),
        Ixr => (100., 50.),
        Iy => (200., 45.),
        Ow => (150., 50.),
        Oxr => (110., 60.),
        Oy => (150., 60.),
        Uh => (90., 65.),
        Uw => (140., 55.),
        Uxr => (150., 50.),
        Yu => (100., 45.),
        _ => return None,
    })
}

struct DipState {
    /// Time of the center of the transition, from Cumdur, for f1..f3.
    tcenter: [f32; 3],
    /// Half-width of the transition.
    tcdips: f32,
    /// Boundary value at the center, for f1..f3.
    bval: [f32; 3],
}

/// Table C-6: duration of forward smoothing for obstruents (Tcobst), in ms.
fn tcobst(phone: Phone) -> Option<f32> {
    use Phone::*;
    Some(match phone {
        Bb | Em | Ff | Mm | Vv => 60.,
        Dd | Dh | El | En | Hh | Hx | Nn | Pp | Qq | Sil | Ss | Th | Tq | Tt | Zz => 80.,
        Ch | Gg | Gp | Kk | Kp | Ng | Sh | Zh => 100.,
        _ => return None,
    })
}

/// F0 used when the input does not specify one.
const DEFAULT_F0: f32 = 160.0;

/// One phonetic segment, as output by PROSOD.
#[derive(Clone, Debug)]
pub struct Segment {
    pub phone: Phone,
    /// Duration in ms
    pub dur: u32,
    /// F0 targets as (position within the segment from 0 to 1, Hz). F0 is
    /// interpolated linearly between the targets of all segments.
    pub f0: Vec<(f32, f32)>,
    pub stressed: bool,
}

/// Parse segments from text.
///
/// Each nonblank line is `phone duration_ms [f0_hz]`, with an optional
/// leading `'` on the phone to mark stress. Text after `#` is a comment.
/// A line with no numbers is a whitespace-separated list of phones, each
/// using its inherent duration from table 9-1.
pub fn parse_segments(text: &str) -> Result<Vec<Segment>, String> {
    let mut segs = vec![];
    for (lineno, line) in text.lines().enumerate() {
        let err = |e: String| format!("line {}: {e}", lineno + 1);
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let toks: Vec<&str> = line.split_whitespace().collect();
        let has_numbers = toks.iter().skip(1).any(|t| t.parse::<f32>().is_ok());
        if !has_numbers {
            for tok in toks {
                segs.push(parse_one(tok, None, None).map_err(err)?);
            }
            continue;
        }
        let dur = toks[1]
            .parse::<u32>()
            .map_err(|_| err(format!("bad duration {}", toks[1])))?;
        let f0 = match toks.get(2) {
            Some(s) => Some(s.parse::<f32>().map_err(|_| err(format!("bad f0 {s}")))?),
            None => None,
        };
        segs.push(parse_one(toks[0], Some(dur), f0).map_err(err)?);
    }
    Ok(segs)
}

fn parse_one(tok: &str, dur: Option<u32>, f0: Option<f32>) -> Result<Segment, String> {
    let (stressed, name) = match tok.strip_prefix('\'') {
        Some(rest) => (true, rest),
        None => (false, tok),
    };
    let phone =
        Phone::parse(&name.to_ascii_lowercase()).ok_or_else(|| format!("unknown phone {tok}"))?;
    let dur = match dur {
        Some(d) => d,
        None => phone_duration(phone)
            .map(|d| d.inherent_duration as u32)
            .ok_or_else(|| format!("no inherent duration for {tok}"))?,
    };
    Ok(Segment {
        phone,
        dur,
        f0: vec![(0.5, f0.unwrap_or(DEFAULT_F0))],
        stressed,
    })
}

// Parameters drawn by the smoothing engine, in the order of tables C-1 and
// C-2, with fnz appended. F0 is handled separately.
const AV: usize = 0;
const AVC: usize = 1;
const ASP: usize = 2;
const AF: usize = 3;
const A2: usize = 4;
const AB: usize = 9;
const F1: usize = 10;
const F2: usize = 11;
const F3: usize = 12;
const F4: usize = 13;
const B1: usize = 14;
const B3: usize = 16;
const FNZ: usize = 17;
const N_PAR: usize = 18;

type Pars = [f32; N_PAR];

/// Table C-3: default duration of forward smoothing (Tcf), in ms.
const TCF_DEFAULT: Pars = [
    25., 20., 20., 40., // av, avc, asp, af
    40., 40., 40., 40., 40., 40., // a2..a6, ab
    80., 80., 80., 80., // f1..f4
    80., 80., 80.,  // b1..b3
    150., // fnz
];

/// Transition types, figure 11-6.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(
    dead_code,
    reason = "SMODIS and DISSMO are set by context rules, not yet implemented"
)]
enum Trantype {
    /// Discontinuous: jump straight to the target.
    Discon,
    /// Smooth backward and forward.
    Setsmo,
    /// Smooth backward to the boundary, then jump.
    Smodis,
    /// Jump at the boundary, then smooth forward.
    Dissmo,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Manner {
    Vowel = 0,
    Stop = 1,
    Fricative = 2,
    Sonorant = 3,
}

impl Manner {
    /// C.4
    fn of(phone: Phone) -> Self {
        if phone.is(Flags::VOWEL) {
            Manner::Vowel
        } else if phone.is(Flags::STOP) {
            Manner::Stop
        } else if phone.is(Flags::FRICATIVE) {
            Manner::Fricative
        } else {
            Manner::Sonorant
        }
    }
}

/// Table C-4: default Bper, indexed by [previous manner][current manner].
const BPER_DEFAULT: [[f32; 4]; 4] = [
    [50., 35., 50., 75.],
    [65., 50., 50., 65.],
    [50., 50., 50., 75.],
    [25., 35., 25., 50.],
];

/// Table targets for a phone (C.4), before any context rules.
fn table_target(phone: Phone, next: Option<Phone>, opts: &Options) -> Pars {
    let mut out = [0.; N_PAR];
    let target = vocalic_target(phone)
        .or_else(|| nonvocalic_target(phone))
        .unwrap_or_else(|| nonvocalic_target(Phone::Sil).unwrap());
    out[..17].copy_from_slice(&target.values());
    // HH and HX inherit frequency targets from the next segment.
    if matches!(phone, Phone::Hh | Phone::Hx) {
        if let Some(next) = next {
            let next_target = table_target(next, None, opts);
            out[F1..=F4].copy_from_slice(&next_target[F1..=F4]);
        }
    }
    out[FNZ] = if opts.nasal_zero && phone.is(Flags::NASAL) {
        450.
    } else {
        250.
    };
    out
}

/// Parameter tracks on a 1 ms grid.
struct Tracks {
    v: Vec<Pars>,
}

impl Tracks {
    fn at(&self, p: usize, t: u32) -> f32 {
        self.v[(t as usize).min(self.v.len() - 1)][p]
    }

    /// Draw a straight line from (t0, v0) to (t1, v1), inclusive.
    fn line(&mut self, p: usize, t0: u32, v0: f32, t1: u32, v1: f32) {
        let t1 = (t1 as usize).min(self.v.len() - 1);
        let t0 = t0 as usize;
        for t in t0..=t1 {
            let u = if t1 > t0 {
                (t - t0) as f32 / (t1 - t0) as f32
            } else {
                1.0
            };
            self.v[t][p] = v0 + (v1 - v0) * u;
        }
    }

    /// Hold a value from t0 to the end of the utterance.
    fn hold(&mut self, p: usize, t0: u32, v: f32) {
        for frame in &mut self.v[t0 as usize..] {
            frame[p] = v;
        }
    }
}

/// Render segments to a sequence of frames, one per `FRAME_TIME` ms.
///
/// Parameter tracks are drawn segment by segment as in section 11.3: the
/// target for each segment is drawn, then the boundary with the previous
/// segment is smoothed using one of the templates of figure 11-6. Only the
/// default (context-free) parts of appendix C are applied so far: table
/// targets (C.4), Tcf from table C-3, Bper from table C-4, SETSMO, and the
/// generic boundary values and minimum formant separation of C.10.
///
/// Diphthongs move linearly toward their second target over the segment
/// (a placeholder for C.7.1). F0 is interpolated linearly between the
/// segments' F0 targets.
pub fn render(segs: &[Segment], opts: &Options) -> Vec<KlattParams> {
    // F0 anchor points.
    let mut f0_points = vec![];
    let mut start = 0;
    for seg in segs {
        for &(pos, f0) in &seg.f0 {
            f0_points.push((start as f32 + pos * seg.dur as f32, f0));
        }
        start += seg.dur;
    }
    if f0_points.is_empty() {
        f0_points.push((0., DEFAULT_F0));
    }
    let total = start;

    // The utterance starts from silence.
    let sil = table_target(Phone::Sil, None, opts);
    let mut tracks = Tracks {
        v: vec![sil; total as usize + 1],
    };

    let mut start = 0;
    let mut prev_start = 0;
    // Burst duration of the previous segment, if it had a burst.
    let mut prev_burdur: Option<u32> = None;
    for (i, seg) in segs.iter().enumerate() {
        let phone = seg.phone;
        let prev = if i > 0 { segs[i - 1].phone } else { Phone::Sil };
        let next = segs.get(i + 1).map(|s| s.phone);
        let end = start + seg.dur;

        // C.4 initialization
        let mut target = table_target(phone, next, opts);
        let mut cumdur = [start; N_PAR];
        let mut mintime = [prev_start; N_PAR];
        let mut trantype = [Trantype::Setsmo; N_PAR];
        let mut tcf = TCF_DEFAULT;
        let bper = BPER_DEFAULT[Manner::of(prev) as usize][Manner::of(phone) as usize];
        let mut aspdux = 0;
        let mut aspaml = ASPAMP;
        let mut oldval_avc_inc = 0.;

        // C.5 general rules. (The F0 rules are omitted, since F0 is not yet
        // drawn by this engine.)
        // Suppress amplitude smoothing after plosive.
        if opts.on(Rule::MintimeAf) && prev.is(Flags::PLOSIVE) {
            mintime[AF] = cumdur[AF];
        }
        // Discontinuous transition out of unvoiced segment.
        if opts.on(Rule::AvTrantype) && !prev.is(Flags::VOICED) {
            trantype[AV] = Trantype::Dissmo;
        }
        // Breathy offset into a pause.
        if opts.on(Rule::Preasp) && prev != Phone::Sil && phone == Phone::Sil {
            aspdux = 30;
            oldval_avc_inc = 6.;
            aspaml = ASPAMP;
        }
        // Aspiration between voiced sonorant and following unvoiced
        // consonant intrudes on the voicing.
        // Read literally, this also matches SIL (which is -voiced), cutting
        // the 30 ms breathy offset above to 10 ms. By default SIL is
        // excluded, as the comment speaks of a consonant.
        if opts.on(Rule::Preasp)
            && prev.is(Flags::SONORANT)
            && !phone.is(Flags::VOICED)
            && (phone != Phone::Sil || opts.on(Rule::PreaspSil))
        {
            aspdux = 10;
        }
        // Start frication early if fricative follows sonorant.
        if opts.on(Rule::EarlyAf)
            && prev.is(Flags::SONORANT)
            && !phone.is(Flags::VOICED)
            && phone.is(Flags::FRICATIVE)
        {
            cumdur[AF] = cumdur[AF].saturating_sub(20);
        }
        if opts.on(Rule::Preasp)
            && prev.is(Flags::SONORANT)
            && !phone.is(Flags::VOICED)
            && phone.is(Flags::STOP)
        {
            aspaml -= 6.;
        }
        // F0 transitions into and out of voiceless segments are
        // discontinuous.
        if opts.on(Rule::AvTrantype) && !phone.is(Flags::VOICED) {
            trantype[AV] = Trantype::Smodis;
        }
        if opts.on(Rule::Tcobst) && prev.is(Flags::STOP) && !phone.is(Flags::STOP) {
            if let Some(t) = tcobst(prev) {
                tcf[F1..=B3].fill(t);
            }
        }
        if opts.on(Rule::AmpTrantype) {
            if prev.is(Flags::STOP | Flags::NASAL)
                && !phone.is(Flags::STOP)
                && phone.is(Flags::VOICED)
            {
                trantype[AV] = Trantype::Discon;
                trantype[AVC] = Trantype::Discon;
                trantype[AF..=B3].fill(Trantype::Dissmo);
            } else {
                for p in AV..=AB {
                    // Read literally, this ELSE overrides every amplitude
                    // transition type set above. The alternative reading only
                    // replaces ones still at the default.
                    if opts.amp_else == AmpElse::Literal || trantype[p] == Trantype::Setsmo {
                        trantype[p] = Trantype::Dissmo;
                    }
                }
                trantype[F1..=B3].fill(Trantype::Setsmo);
            }
        }
        // C.8.1 fricatives and plosives
        let next_is = |f: Flags| next.is_some_and(|n| n.is(f));
        if opts.on(Rule::Obstruent) && phone.is(Flags::PLOSIVE | Flags::FRICATIVE) {
            // Devoice unless a vowel follows.
            if phone.is(Flags::VOICED) && !next_is(Flags::VOWEL) {
                target[AVC] = (target[AVC] - 30.).max(0.);
            }
            if phone.is(Flags::PALATAL) {
                target[F4] -= 150.;
            }
            if phone.is(Flags::ALVEOLAR) && phone.is(Flags::PLOSIVE) && !next_is(Flags::ALVEOLAR) {
                let a: [f32; 5] = if next_is(Flags::RETRO) {
                    [0., 67., 60., 65., 65.]
                } else {
                    [0., 52., 60., 70., 73.]
                };
                target[A2..A2 + 5].copy_from_slice(&a);
            }
            // No voicebar in a voiced plosive after an obstruent.
            if phone.is(Flags::VOICED)
                && phone.is(Flags::PLOSIVE)
                && (!prev.is(Flags::VOICED) || !prev.is(Flags::SONORANT))
            {
                target[AVC] = 0.;
            }
        }
        // C.8.3.1: insert a burst at the expense of the closure.
        let burst = if opts.on(Rule::Burst)
            && phone.is(Flags::PLOSIVE)
            && phone != Phone::Qq
            && next.is_some_and(|n| !n.is(Flags::STOP) && n != Phone::Sil)
        {
            let mut bd = burdur(phone).unwrap_or(0);
            if phone == Phone::Tt && next_is(Flags::RETRO) {
                bd += 10;
            }
            bd = bd.min(end - cumdur[AF]);
            if bd == 0 {
                bd = 5;
            }
            let mut amp = BURAMP;
            if phone.is(Flags::VELAR) && !next_is(Flags::VOWEL) {
                amp -= 5.;
            }
            if !phone.is(Flags::VOICED) {
                amp += 6.;
            }
            if !seg.stressed {
                amp -= if prev.is(Flags::NASAL) { 6. } else { 3. };
            }
            if next == Some(Phone::Axp) || next_is(Flags::FRICATIVE) {
                amp -= 3.;
            }
            Some((bd, amp))
        } else {
            None
        };

        if !opts.smooth {
            trantype = [Trantype::Discon; N_PAR];
        }

        // Pre-aspiration (figure C-1): shift Cumdur[asp] earlier by Aspdux,
        // force asp there to Aspaml, and smooth linearly over the preceding
        // 30 ms.
        if aspdux > 0 {
            let ta = start.saturating_sub(aspdux);
            let t0 = ta.saturating_sub(30);
            tracks.line(ASP, t0, tracks.at(ASP, t0), ta, aspaml);
            tracks.hold(ASP, ta, aspaml);
            cumdur[ASP] = ta;
            mintime[ASP] = ta;
        }

        // C.6.1: aspiration of a preceding voiceless obstruent intrudes on
        // the sonorant.
        if opts.on(Rule::Aspiration)
            && phone.is(Flags::SONORANT)
            && !prev.is(Flags::VOICED)
            && !matches!(prev, Phone::Qq | Phone::Tq | Phone::Sil | Phone::Hh)
        {
            let prev_stressed = i > 0 && segs[i - 1].stressed;
            let mut aspdur: i32 = 40;
            let mut aml = ASPAMP;
            if !prev_stressed {
                aspdur = 20;
                aml -= 3.;
            }
            if !phone.is(Flags::VOWEL) {
                aspdur = if !prev.is(Flags::LABIAL) && !phone.is(Flags::LATERAL) {
                    55
                } else {
                    50
                };
            }
            if i >= 2 && segs[i - 2].phone.is(Flags::VOICED) && !seg.stressed {
                aspdur -= 10;
            }
            if phone == Phone::Axp {
                aspdur = 70;
            }
            if prev.is(Flags::PLOSIVE)
                && let Some(bd) = prev_burdur
            {
                // Aspiration starts during the burst.
                let shift = bd as i32 - 5;
                for p in [ASP, AV, AVC, B1] {
                    cumdur[p] = if opts.on(Rule::AspBurstLiteral) {
                        (cumdur[p] as i32 + shift) as u32
                    } else {
                        (cumdur[p] as i32 - shift) as u32
                    };
                }
                aspdur += shift;
            }
            if prev.is(Flags::FRICATIVE) {
                aspdur = if phone.is(Flags::VOWEL) {
                    10
                } else {
                    (end.saturating_sub(cumdur[ASP]) / 2) as i32
                };
            }
            aspdur = aspdur
                .min(end.saturating_sub(cumdur[ASP]) as i32)
                .min(end.saturating_sub(cumdur[AV]) as i32)
                .max(0);
            if prev.is(Flags::ALVEOLAR) && phone.is(Flags::RETRO) {
                aml += 9.;
            }
            if aspdur > 0 {
                let aspdur = aspdur as u32;
                let c0 = cumdur[ASP];
                if prev.is(Flags::FRICATIVE) {
                    let t0 = c0.saturating_sub(30);
                    tracks.line(ASP, t0, tracks.at(ASP, t0), c0, aml);
                }
                for (p, v) in [(ASP, aml), (AV, 0.), (AVC, 0.), (B1, 150.)] {
                    tracks.hold(p, cumdur[p], v);
                    cumdur[p] += aspdur;
                    mintime[p] = cumdur[p];
                }
            }
        }

        let mut oldval: Pars = std::array::from_fn(|p| tracks.at(p, cumdur[p]));
        oldval[AVC] += oldval_avc_inc;

        // C.10 modifications (generic parts)
        // Minimum formant separation is 200 Hz.
        for p in F1..F4 {
            target[p + 1] = target[p + 1].max(target[p] + 200.);
        }
        let tcb = tcf;
        let bvalf: Pars = std::array::from_fn(|p| {
            if p <= AB {
                // Amplitudes: average of target and old value.
                0.5 * (target[p] + oldval[p])
            } else {
                (bper * target[p] + (100. - bper) * oldval[p]) / 100.
            }
        });
        let bvalb = bvalf;

        // C.7.1 diphthong rules
        let glide = vocalic_target_glide(phone).map(|g| g.values());
        let dip = match (glide, diphthong_params(phone)) {
            (Some(diptar), Some((tcdiph, tdmid))) if phone.is(Flags::DIPHTHONG) => {
                let segdur = (end - cumdur[F1]) as f32;
                let mut tcenter = [tdmid / 100. * segdur; 3];
                let tcdips = match opts.dip_units {
                    DipUnits::Tenth => tcdiph / 10. * 0.5 * (1. + segdur / 45.),
                    DipUnits::Ms => tcdiph * 0.5 * (1. + segdur / 45.),
                    DipUnits::Frames => tcdiph * 0.5 * (1. + segdur / 225.),
                    DipUnits::Narrator => tcdiph * segdur / 540.,
                };
                // Earlier diphthongization after HH (printed as +lax, though
                // the comment says tense vowels).
                if prev == Phone::Hh && phone.is(Flags::LAX) {
                    tcenter.iter_mut().for_each(|t| *t *= 0.67);
                }
                let mut bval: [f32; 3] =
                    std::array::from_fn(|k| 0.5 * (target[F1 + k] + diptar[k]));
                if phone == Phone::Oy {
                    bval[0] += 70.;
                }
                if matches!(phone, Phone::Ay | Phone::Oy) {
                    bval[1] += 200.;
                    bval[2] -= 200.;
                }
                if phone == Phone::Yu {
                    tcenter[2] *= 0.3;
                }
                if matches!(phone, Phone::Exr | Phone::Ixr) {
                    tcenter[2] *= 0.6;
                }
                Some(DipState {
                    tcenter,
                    tcdips,
                    bval,
                })
            }
            _ => None,
        };

        // Draw all parameter tracks (figure 11-6).
        for p in 0..N_PAR {
            let t1 = cumdur[p];
            if p == AF
                && let Some((bd, amp)) = burst
            {
                // Lower af by 20 dB at Cumdur and smooth backward over
                // 30 ms; af is zero through the closure, then the burst.
                let t0 = t1.saturating_sub(30).max(mintime[AF]);
                tracks.line(AF, t0, tracks.at(AF, t0), t1, (oldval[AF] - 20.).max(0.));
                tracks.hold(AF, t1, 0.);
                tracks.hold(AF, end - bd, amp);
                continue;
            }
            if matches!(trantype[p], Trantype::Setsmo | Trantype::Smodis) {
                let tb = t1.saturating_sub(tcb[p] as u32).max(mintime[p]);
                tracks.line(p, tb, tracks.at(p, tb), t1, bvalb[p]);
            }
            if matches!(trantype[p], Trantype::Setsmo | Trantype::Dissmo) {
                let tf = t1 + tcf[p] as u32;
                if tf > end {
                    // Target not reached within the segment.
                    let u = (end - t1) as f32 / (tf - t1) as f32;
                    tracks.line(p, t1, bvalf[p], end, bvalf[p] + (target[p] - bvalf[p]) * u);
                } else {
                    tracks.line(p, t1, bvalf[p], tf, target[p]);
                    tracks.hold(p, tf, target[p]);
                }
            } else {
                tracks.hold(p, t1, target[p]);
            }
            if let Some(glide) = glide
                && (F1..=F3).contains(&p)
            {
                let diptar = glide[p - F1];
                if let Some(dip) = dip.as_ref().filter(|_| opts.on(Rule::Diphthong)) {
                    // C.7.1: draw a SETSMO transition from Target to Diptar
                    // centered at Cumdur + Tcenter (figure C-2).
                    let k = p - F1;
                    let tc = t1 + dip.tcenter[k].round() as u32;
                    let tcdips = dip.tcdips.round() as u32;
                    let tb = tc.saturating_sub(tcdips).max(t1);
                    tracks.line(p, tb, tracks.at(p, tb), tc, dip.bval[k]);
                    let tf = tc + tcdips;
                    if tf > end {
                        let u = (end - tc) as f32 / (tf - tc).max(1) as f32;
                        let v_end = dip.bval[k] + (diptar - dip.bval[k]) * u;
                        tracks.line(p, tc, dip.bval[k], end, v_end);
                    } else {
                        tracks.line(p, tc, dip.bval[k], tf, diptar);
                        tracks.hold(p, tf, diptar);
                    }
                } else {
                    // Placeholder: add a linear glide toward the second
                    // target over the course of the segment.
                    let delta = diptar - target[p];
                    for t in t1..=end.min(total) {
                        let u = (t - t1) as f32 / (end - t1) as f32;
                        tracks.v[t as usize][p] += delta * u;
                    }
                }
            }
            let end_val = tracks.at(p, end);
            tracks.hold(p, end.min(total), end_val);
        }
        prev_burdur = burst.map(|(bd, _)| bd);
        prev_start = start;
        start = end;
    }

    let mut frames = vec![];
    let mut out = KlattParams::default();
    let mut t = 0;
    while t < total {
        let v = &tracks.v[t as usize];
        let (av, avs) = opts.voicing.map(v[AV], v[AVC]);
        out.av = av;
        out.avs = avs;
        out.ah = v[ASP];
        out.af = v[AF];
        out.a2 = v[A2];
        out.a3 = v[A2 + 1];
        out.a4 = v[A2 + 2];
        out.a5 = v[A2 + 3];
        out.a6 = v[A2 + 4];
        out.ab = v[AB];
        out.f1 = v[F1];
        out.f2 = v[F2];
        out.f3 = v[F3];
        out.f4 = v[F4];
        out.b1 = v[B1];
        out.b2 = v[B1 + 1];
        out.b3 = v[B3];
        out.fnz = v[FNZ];
        // Voiceless segments have av = avc = 0 in the tables, so F0 can
        // stay on throughout.
        out.f0 = interp_f0(&f0_points, t as f32);
        frames.push(out.clone());
        t += FRAME_TIME;
    }
    if let Some(beta) = opts.f0_beta {
        let mut lf: Vec<f32> = frames.iter().map(|f| f.f0.ln()).collect();
        smooth_critically_damped(&mut lf, beta * FRAME_TIME as f32 / 1000.);
        lf.reverse();
        smooth_critically_damped(&mut lf, beta * FRAME_TIME as f32 / 1000.);
        lf.reverse();
        for (f, l) in frames.iter_mut().zip(lf) {
            f.f0 = l.exp();
        }
    }
    frames
}

/// Two cascaded one-pole lowpasses with the same pole: the impulse response
/// is proportional to `t exp(-beta t)`, Fujisaki's accent response.
/// `beta_dt` is beta times the sample period. The state starts at the first
/// value, so a constant input passes unchanged.
fn smooth_critically_damped(x: &mut [f32], beta_dt: f32) {
    let Some(&x0) = x.first() else { return };
    let a = (-beta_dt).exp();
    let (mut y1, mut y2) = (x0, x0);
    for v in x.iter_mut() {
        y1 = a * y1 + (1. - a) * *v;
        y2 = a * y2 + (1. - a) * y1;
        *v = y2;
    }
}

fn interp_f0(points: &[(f32, f32)], t: f32) -> f32 {
    let i = points.partition_point(|&(pt, _)| pt <= t);
    if i == 0 {
        return points[0].1;
    }
    if i == points.len() {
        return points[i - 1].1;
    }
    let (t0, f0) = points[i - 1];
    let (t1, f1) = points[i];
    f0 + (f1 - f0) * (t - t0) / (t1 - t0)
}
