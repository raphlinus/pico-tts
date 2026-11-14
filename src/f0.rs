//! Fundamental frequency targets, following chapter 10 of the MITalk book
//! (O'Shaughnessy's algorithm as adapted for MITalk).
//!
//! The book describes the rules in prose and gives only some of the numbers.
//! The structure and the missing numbers here come from the F0 code of the
//! Amiga narrator.device (Katz/Barton 1990: f0hls.asm, f0lls.asm,
//! phonetf0.c), which implements the same algorithm. Where the book and the
//! narrator disagree on a constant, [`F0Consts`] picks one.
//!
//! The algorithm works one clause at a time (a clause ends at a pause
//! boundary). Each syllable gets a start, peak and end value:
//!
//! - High level: an accent number per syllable, break numbers at phrase
//!   ("p-unit") boundaries, and tune A on the last word of a statement.
//! - B: the head peak on the first accented syllable, then peaks dropping by
//!   equal steps toward the base of range (BOR); peaks scaled by accent.
//! - C: rises and falls of accented syllables from the break numbers.
//! - D: adjustments for adjacent or widely spaced accents; terminal falls.
//! - E: continuation rises.
//! - F: values for unaccented syllables.
//! - G: split of each rise into jump and rise, and each fall into drop and
//!   fall, by the voicing of the surrounding consonants.
//!
//! The values are then placed on segments as in phonetf0.c.
//!
//! Only tune A (statements, and no tune for clauses ending in a comma) is
//! implemented. Without a parser, every content word is accent level 7
//! (table 10-1), every phrase is a p-unit, and the narrator's "level" rule
//! is not used (it sets all levels to 1 too).

use crate::{
    phones::{Flags, Phone},
    prosod::Annotated,
};

/// Constant sets for the places where the book and the narrator differ.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum F0Consts {
    /// Book values where the book gives them, narrator otherwise.
    #[default]
    Book,
    /// Narrator values throughout.
    Narrator,
}

struct Consts {
    /// Terminal F0 of tune A. Book: 75 Hz lowered by 10 (10.5). Narrator: 75.
    term: f32,
    /// Floor of exponential falls between accents, and the start of the rise
    /// to an accent after three or more unaccented syllables. Book: 95 Hz.
    /// Narrator: BOR - 5 = 105 Hz.
    floor: f32,
    /// Head peak clamp. Book: "a maximum of about 190 Hz" (no minimum given).
    /// Narrator: 124..144 Hz.
    head_min: f32,
    head_max: f32,
    /// Continuation rise before a comma. Book: 16 Hz. Narrator: 24 Hz
    /// (changed from 16 in the code).
    cr_punct: f32,
    /// Book only: 8 Hz continuation rise at the end of a nonfinal phrase, if
    /// more than five words have passed since the last one.
    cr_phrase: Option<f32>,
    /// Rise increase for accents separated by 2, 3, 4+ unaccented syllables.
    /// Book: 15, 20, 30%. Narrator: 15, 25, 30%.
    spaced_rise: [f32; 3],
    head_override: Option<f32>,
}

impl F0Consts {
    fn get(self) -> Consts {
        match self {
            F0Consts::Book => Consts {
                term: 65.,
                floor: 95.,
                head_min: 124.,
                head_max: 190.,
                cr_punct: 16.,
                cr_phrase: Some(8.),
                spaced_rise: [0.15, 0.20, 0.30],
                head_override: None,
            },
            F0Consts::Narrator => Consts {
                term: 75.,
                floor: 105.,
                head_min: 124.,
                head_max: 144.,
                cr_punct: 24.,
                cr_phrase: None,
                spaced_rise: [0.15, 0.25, 0.30],
                head_override: None,
            },
        }
    }
}

/// Base of range for tune A (book: initial value of the lower declination
/// line; narrator: BOR).
const BOR: f32 = 110.;
/// Narrator: head peak base frequency, raised by `HEAD_PER_PHRASE` minus
/// 2 Hz per clause for each estimated phrase, and lowered by 8 Hz per
/// clause.
const HEAD_BASE: f32 = 122.;
const HEAD_PER_PHRASE: f32 = 12.;
/// 5 Hz excursion on unaccented syllables (book; narrator uses 6).
const BUMP: f32 = 5.;
/// Start of a voiced syllable after a glottal stop (narrator).
const AFTER_GLOTTAL: f32 = 90.;
/// GUESS: accent number added for an emphasized (`!`) word.
const EMPH_ACCENT: i32 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum End {
    /// End of a phrase without a pause.
    Phrase,
    /// End of a clause (a pause), not the end of the sentence.
    Pause,
    /// End of the sentence.
    Final,
}

/// A syllable, with segment indices into the PROSOD output.
#[derive(Debug)]
pub struct Syl {
    pub first: usize,
    pub nucleus: usize,
    /// Inclusive.
    pub last: usize,
    /// Primary-stressed syllable of a content word.
    pub accented: bool,
    /// Stress of the nucleus (0, 1 primary, 2 secondary).
    pub stress: u8,
    pub emph: bool,
    pub word: usize,
    /// Set on the last syllable of a word that ends a phrase.
    pub end: Option<End>,
}

#[derive(Clone, Copy, Default, Debug)]
struct Values {
    start: f32,
    peak: f32,
    end: f32,
    rise: f32,
    fall: f32,
    cr: f32,
}

fn voiced(phone: Phone) -> bool {
    phone.is(Flags::VOICED) || phone.is(Flags::SONORANT) || phone.is(Flags::SYLLABIC)
}

fn glottal(phone: Phone) -> bool {
    matches!(phone, Phone::Qq | Phone::Tq)
}

/// Compute F0 targets and store them in the segments.
/// `head` overrides the computed head peak of every clause.
pub fn assign(
    syls: &[Syl],
    out: &mut [Annotated],
    consts: F0Consts,
    head: Option<f32>,
    left_final: bool,
) {
    let mut c = consts.get();
    c.head_override = head;
    let mut v = vec![Values::default(); syls.len()];
    // Split into clauses.
    let mut lo = 0;
    let mut n_clause = 0;
    for k in 0..syls.len() {
        if matches!(syls[k].end, Some(End::Pause | End::Final)) || k + 1 == syls.len() {
            n_clause += 1;
            clause(&syls[lo..=k], out, &c, n_clause, &mut v[lo..=k]);
            lo = k + 1;
        }
    }
    place(syls, &v, out, left_final);
}

fn clause(syls: &[Syl], out: &[Annotated], c: &Consts, n_clause: usize, v: &mut [Values]) {
    let n = syls.len();
    let tune_a = syls[n - 1].end == Some(End::Final);
    let word_final = |k: usize| k + 1 == n || syls[k + 1].word != syls[k].word;
    let poly = |k: usize| {
        let w = syls[k].word;
        syls.iter().filter(|s| s.word == w).count() > 1
    };
    let in_last_word = |k: usize| syls[k].word == syls[n - 1].word;
    // Accent numbers.
    let accent: Vec<i32> = (0..n)
        .map(|k| {
            let s = &syls[k];
            if s.accented {
                7 + if poly(k) { 2 } else { 0 } + if s.emph { EMPH_ACCENT } else { 0 }
            } else if s.stress == 1 {
                // Stressed syllable of a function word (prepositions are
                // level 2 in table 10-1).
                2
            } else {
                0
            }
        })
        .collect();
    let accents: Vec<usize> = (0..n).filter(|&k| syls[k].accented).collect();

    // Break numbers: +2 on the first accent of a phrase, -2 on the last.
    let mut brk = vec![0i32; n];
    let mut p_lo = 0;
    for k in 0..n {
        if syls[k].end.is_some() || k + 1 == n {
            let in_phrase: Vec<usize> = accents
                .iter()
                .copied()
                .filter(|&j| j >= p_lo && j <= k)
                .collect();
            if let (Some(&f), Some(&l)) = (in_phrase.first(), in_phrase.last()) {
                brk[l] = -2;
                brk[f] = 2;
            }
            p_lo = k + 1;
        }
    }
    // Clause boundary before a pause: break 4 on the last accent and a
    // fall+rise continuation rise on the last syllable.
    if !tune_a {
        if let Some(&l) = accents.last() {
            brk[l] = 4;
        }
        v[n - 1].cr = c.cr_punct;
    }
    // Book: continuation rise at the end of a long nonfinal phrase.
    if let Some(cr) = c.cr_phrase {
        let mut words = 0;
        let mut last_word = usize::MAX;
        for k in 0..n {
            if syls[k].word != last_word {
                words += 1;
                last_word = syls[k].word;
            }
            if syls[k].end == Some(End::Phrase) && words > 5 {
                v[k].cr = cr;
                words = 0;
            }
        }
    }

    if accents.is_empty() {
        // F4B1 and F5 only.
        for k in 0..n {
            v[k].start = BOR;
            v[k].end = BOR;
            v[k].peak = BOR + 2. * accent[k] as f32;
        }
        final_fall(syls, v, c, tune_a, None);
        return;
    }

    // B: peaks.
    let words = {
        let mut w: Vec<usize> = syls.iter().map(|s| s.word).collect();
        w.dedup();
        w.len()
    };
    let n_phr = ((words + 4) / 3) as f32;
    let head = HEAD_BASE + n_phr * (HEAD_PER_PHRASE - 2. * n_clause as f32).max(0.)
        - 8. * n_clause as f32;
    let head = c
        .head_override
        .unwrap_or_else(|| head.clamp(c.head_min, c.head_max));
    let n_as = accents.len();
    let drop = (head - BOR) / n_as as f32;
    let (larger, smaller) = if n_as >= 4 {
        (drop * 1.15, drop - 0.3 * drop / (n_as - 3) as f32)
    } else {
        (drop, drop)
    };
    let mut p = head;
    for (i, &k) in accents.iter().enumerate() {
        if i > 0 {
            p -= if i == 1 || i + 1 == n_as { larger } else { smaller };
        }
        v[k].peak = p;
    }
    for &k in &accents {
        let room = v[k].peak - BOR;
        v[k].peak = (v[k].peak + (accent[k] - 8) as f32 * 0.1 * room).clamp(BOR, 400.);
    }

    // C: rises and falls.
    for &k in &accents {
        let room = v[k].peak - BOR;
        let b = brk[k] as f32;
        v[k].rise = 0.4 * room * (1. + 0.2 * b);
        v[k].fall = (0.2 * room * (1. - b)).max(0.);
        if brk[k] == 0 {
            // Inside a phrase.
            v[k].rise *= 0.7;
            v[k].fall *= 0.7;
        }
    }
    let first = accents[0];
    v[first].rise = v[first].peak - BOR;

    // D1: accent spacing, from the last pair to the first.
    for w in accents.windows(2).rev() {
        let (a, b) = (w[0], w[1]);
        let n_us = b - a - 1;
        let lr = |v: &Values| v.peak - BOR;
        match n_us {
            0 => {
                v[a].rise *= 0.6;
                v[b].rise *= 0.6;
                v[a].peak -= 0.2 * lr(&v[a]);
                v[b].peak += 0.2 * lr(&v[b]);
                // Make the end of the first meet the start of the second.
                let d = (v[a].peak - v[a].fall) - (v[b].peak - v[b].rise);
                if d < 0. {
                    v[b].rise -= d;
                } else {
                    v[a].fall += d;
                }
            }
            1 => {}
            _ => {
                let pct = c.spaced_rise[(n_us - 2).min(2)];
                v[a].rise *= 1. + pct;
                v[b].rise *= 1. + pct;
                v[b].peak -= if n_us == 2 { 0.15 } else { 0.25 } * lr(&v[b]);
                v[a].peak += if n_us == 2 { 0.10 } else { 0.15 } * lr(&v[a]);
                // D1F: an accent starting a phrase after three or more
                // unaccented syllables rises from the floor.
                if n_us >= 3 && brk[b] > 0 {
                    v[b].rise = v[b].peak - c.floor;
                }
            }
        }
    }
    // D2: terminal fall on a word-final accent in the last word.
    for &k in &accents {
        if tune_a && word_final(k) && in_last_word(k) {
            v[k].fall = v[k].peak - c.term;
        }
    }
    // E: a fall+rise continuation rise increases the fall by 30%.
    for &k in &accents {
        if v[k].cr > 0. {
            v[k].fall *= 1.3;
        }
    }

    // F2: starts and ends of accents.
    for &k in &accents {
        v[k].start = v[k].peak - v[k].rise;
        v[k].end = v[k].peak - v[k].fall;
    }
    // F3/F4: unaccented syllables between accents.
    for w in accents.windows(2) {
        let (a, b) = (w[0], w[1]);
        let n_us = b - a - 1;
        if n_us == 0 {
            continue;
        }
        let mut d = v[a].end - v[b].start;
        if d < 0. {
            // No rising F0 on the unaccented syllables: meet halfway.
            let h = -d / 2.;
            v[b].start -= h;
            v[b].rise += h;
            v[a].end += h;
            v[a].fall -= h;
            d = 0.;
        }
        let set = |v: &mut [Values], k: usize, end: f32| {
            v[k].start = v[k - 1].end;
            v[k].end = end;
            v[k].peak = end + BUMP;
        };
        if brk[a] >= 0 {
            // Inside a phrase: linear.
            let delta = d / n_us as f32;
            for k in a + 1..b {
                set(v, k, v[k - 1].end - delta);
            }
        } else {
            // Across a phrase boundary: exponential, to the floor for three
            // or more.
            let (d, pcts): (f32, &[f32]) = match n_us {
                1 => (d, &[1.0]),
                2 => (d, &[0.6, 0.4]),
                _ => (v[a].end - c.floor, &[0.45, 0.35, 0.20]),
            };
            for (i, k) in (a + 1..b).enumerate() {
                let e = v[k - 1].end - pcts.get(i).map(|p| p * d).unwrap_or(0.);
                set(v, k, e);
                if i >= pcts.len() {
                    v[k].peak = e + 2. * accent[k] as f32;
                }
            }
        }
    }
    // F4B1: before the first accent.
    for k in 0..first {
        v[k].start = BOR;
        v[k].end = BOR;
        v[k].peak = BOR + 2. * accent[k] as f32;
    }
    // F5: after the last accent.
    let last_as = *accents.last().unwrap();
    final_fall(syls, v, c, tune_a, Some(last_as));

    // G3: jump and rise.
    for &k in &accents {
        let s = &syls[k];
        if s.first == s.nucleus {
            continue;
        }
        let onset = out[s.first].seg.phone;
        let j = if voiced(onset) {
            0.2
        } else {
            let bump = 0.2 * (v[k].peak - BOR);
            v[k].peak += bump;
            v[k].fall += bump;
            v[k].rise += bump;
            0.8
        };
        v[k].start += j * v[k].rise;
        v[k].rise -= j * v[k].rise;
    }
    // G4: drop and fall.
    for &k in &accents {
        let s = &syls[k];
        let pct = if word_final(k) || syls.get(k + 1).is_some_and(|s| s.accented) {
            if k + 1 == n {
                continue;
            }
            0.5
        } else {
            // Look forward from the vowel.
            let mut pct = 0.2;
            for a in &out[s.nucleus + 1..] {
                let p = a.seg.phone;
                if p.is(Flags::SYLLABIC) || glottal(p) {
                    break;
                }
                if !voiced(p) {
                    pct = 0.66;
                    break;
                }
            }
            pct
        };
        v[k].end += pct * v[k].fall;
        v[k].fall -= pct * v[k].fall;
    }
}

/// F5: unaccented syllables at the end of a clause fall in equal steps from
/// the end of the last accent to the terminal value (tune A) or BOR.
fn final_fall(syls: &[Syl], v: &mut [Values], c: &Consts, tune_a: bool, last_as: Option<usize>) {
    let n = syls.len();
    if syls[n - 1].accented {
        return;
    }
    let target = if tune_a { c.term } else { BOR };
    let a = last_as.unwrap_or(0);
    let m = n - 1 - a;
    if m > 0 {
        let diff = (v[a].end - target) / m as f32;
        for k in a + 1..n {
            v[k].start = v[k - 1].end;
            v[k].end = v[k].start - diff;
            v[k].peak = v[k].end + BUMP;
        }
    }
    v[n - 1].end = target;
}

/// Place start, peak and end values on segments (phonetf0.c).
fn place(syls: &[Syl], v: &[Values], out: &mut [Annotated], left_final: bool) {
    let mut targets: Vec<(usize, f32, f32)> = vec![];
    for (k, s) in syls.iter().enumerate() {
        let val = &v[k];
        let nuc = s.nucleus;
        let phrase_final = s.end.is_some();
        let phone = |i: usize| out[i].seg.phone;
        let mut put = |i: usize, pos: f32, f0: f32| {
            // A later target at the same place replaces an earlier one.
            targets.retain(|&(j, p, _)| !(j == i && p == pos));
            targets.push((i, pos, f0));
        };
        // Start, on the first voiced segment.
        if let Some(i) = (s.first..=nuc).find(|&i| voiced(phone(i))) {
            let after_glottal = i > 0 && glottal(phone(i - 1));
            let start = if after_glottal { AFTER_GLOTTAL } else { val.start };
            if i == nuc {
                put(i, 0., start);
            } else if after_glottal {
                put(i, 0., start);
                put(i, 0.5, 0.5 * (start + val.peak));
            } else if phrase_final && i + 1 == nuc {
                put(i, 0.5, start);
            } else {
                put(i, 1., start);
            }
        }
        // Peak.
        let next_syl_start = nuc == s.last;
        let after_vl_plosive =
            nuc > 0 && phone(nuc - 1).is(Flags::PLOSIVE) && !voiced(phone(nuc - 1));
        // The narrator puts the peak of every phrase-final syllable at its
        // left edge. For an unaccented one that replaces the start value,
        // so F0 steps down to the terminal value at the vowel onset. Unless
        // `left_final` is set, an unaccented phrase-final peak goes to the
        // middle instead, and F0 falls through the syllable.
        // (The same goes for the left-edge peak after a voiceless plosive.)
        let smooth_final = phrase_final && !s.accented && !left_final;
        let peak_pos = if smooth_final {
            0.5
        } else if after_vl_plosive || phrase_final {
            0.
        } else if s.accented && !next_syl_start {
            1.
        } else {
            0.5
        };
        put(nuc, peak_pos, val.peak);
        // Fall, terminal value and continuation rise.
        if !phrase_final {
            if next_syl_start {
                put(nuc, 1., val.end);
            } else {
                put(nuc + 1, 0.5, val.end);
            }
        } else {
            let last_voiced = (nuc..=s.last).rev().find(|&i| voiced(phone(i))).unwrap_or(nuc);
            if val.cr > 0. {
                put(nuc, 0.5, val.end);
                put(last_voiced, 1., val.end + val.cr);
            } else {
                // The value is held from the end of the voiced run.
                let run_end = (nuc..=s.last)
                    .find(|&i| i + 1 > s.last || !voiced(phone(i + 1)))
                    .unwrap_or(nuc);
                put(run_end, 1., val.end);
                put(last_voiced, 1., val.end);
            }
        }
    }
    for (i, pos, f0) in targets {
        out[i].seg.f0.push((pos, f0));
    }
    for a in out.iter_mut() {
        a.seg.f0.sort_by(|x, y| x.0.total_cmp(&y.0));
    }
}
