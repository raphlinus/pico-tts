//! Fundamental frequency targets, following chapter 10 of the MITalk book
//! (O'Shaughnessy's algorithm as adapted for MITalk).
//!
//! The book describes the rules in prose and gives only some of the numbers.
//! Constants that the book quantifies say so; the others are marked GUESS,
//! and are candidates for tuning by ear.
//!
//! Only tune A (declaratives) is implemented. Without a parser, every content
//! word is treated as accent level 7 (noun, adjective, adverb) of table 10-1,
//! and function words get no accent.
//!
//! The output is a set of targets per segment (left boundary, middle, right
//! boundary), placed as in section 10.3: peaks at the right boundary of
//! stressed vowels, or the left boundary for the nuclear stress of a phrase,
//! and falls on the following segment.

use crate::{phones::Flags, prosod::Annotated};

/// Initial value of the lower declination line for tune A (book).
const LOWER_START: f32 = 110.0;
/// GUESS: fall of the lower declination line, Hz per second.
const LOWER_DECL: f32 = 10.0;
/// Final F0 of a statement: 75 Hz, lowered by 10 Hz in section 10.5 (book).
const FINAL: f32 = 65.0;
/// Maximum height of the first peak (book: "about 190 Hz").
const TOP_MAX: f32 = 190.0;
/// GUESS: first-peak height for a one-syllable sentence; longer sentences
/// start higher (book), by `TOP_PER_SYL` per syllable, up to `TOP_MAX`.
const TOP_BASE: f32 = 160.0;
/// GUESS: see `TOP_BASE`.
const TOP_PER_SYL: f32 = 2.0;
/// GUESS: fall of the top (peak) declination line, Hz per second.
const TOP_DECL: f32 = 12.0;
/// GUESS: extra peak excursion for an emphasized (`!`) word.
const EMPH_SCALE: f32 = 1.3;
/// Basic rise, as a fraction of the peak above `LOWER_START` (book).
const RISE: f32 = 0.4;
/// Basic fall, as a fraction of the peak above the lower line (book).
const FALL: f32 = 0.2;
/// GUESS: more rise on a phrase-initial peak (book gives no number).
const PHRASE_INITIAL_RISE: f32 = 1.2;
/// GUESS: less rise on a phrase-final peak (book gives no number).
const PHRASE_FINAL_RISE: f32 = 0.8;
/// GUESS: more fall on a phrase-final peak (book gives no number).
const PHRASE_FINAL_FALL: f32 = 2.0;
/// Rises and falls within a phrase are reduced by 30% (book).
const MEDIAL: f32 = 0.7;
/// Extra rise at the left boundary of a vowel after an unvoiced onset, as a
/// fraction of the peak above `LOWER_START` (book: 20%).
const UNVOICED_ONSET: f32 = 0.2;
/// Excursion on unaccented syllables (book: 5 Hz).
const UNACCENTED_BUMP: f32 = 5.0;
/// Continuation rise before a nonterminal punctuation mark (book).
const CR_PUNCT: f32 = 16.0;
/// Continuation rise at the end of a nonfinal phrase (book), if more than
/// `CR_WORDS` words have passed since the last continuation rise.
const CR_PHRASE: f32 = 8.0;
const CR_WORDS: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum End {
    /// End of a phrase without a pause.
    Phrase,
    /// End of a phrase before a pause, not the end of the sentence.
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
    pub emph: bool,
    pub word: usize,
    /// Set on the last syllable of a word that ends a phrase.
    pub end: Option<End>,
}

struct Values {
    start: f32,
    peak: f32,
    end: f32,
    /// Value at the left boundary of the vowel after an unvoiced onset.
    onset_boost: Option<f32>,
    cr: f32,
}

fn voiced(a: &Annotated) -> bool {
    let p = a.seg.phone;
    p.is(Flags::VOICED) || p.is(Flags::SONORANT) || p.is(Flags::SYLLABIC)
}

/// Compute F0 targets and store them in the segments.
pub fn assign(syls: &[Syl], out: &mut [Annotated]) {
    if syls.is_empty() {
        return;
    }
    // Time of each segment start, in seconds.
    let mut t0 = vec![0.0f32; out.len() + 1];
    for (i, a) in out.iter().enumerate() {
        t0[i + 1] = t0[i] + a.seg.dur as f32 / 1000.;
    }
    let mid = |i: usize| 0.5 * (t0[i] + t0[i + 1]);
    let lower = |t: f32| LOWER_START - LOWER_DECL * t;
    let top0 = (TOP_BASE + TOP_PER_SYL * syls.len() as f32).min(TOP_MAX);
    let top = |t: f32| top0 - TOP_DECL * t;

    let accents: Vec<usize> = (0..syls.len()).filter(|&k| syls[k].accented).collect();
    // Index of the phrase each syllable is in.
    let mut phrase = vec![0; syls.len()];
    for k in 1..syls.len() {
        phrase[k] = phrase[k - 1] + syls[k - 1].end.is_some() as usize;
    }
    // How each phrase ends.
    let phrase_end: Vec<Option<End>> = {
        let mut e = vec![None; phrase[syls.len() - 1] + 1];
        for (k, s) in syls.iter().enumerate() {
            if s.end.is_some() {
                e[phrase[k]] = s.end;
            }
        }
        e
    };
    // Nuclear stress: the last accent of a phrase that ends in a pause.
    let nuclear: Vec<bool> = (0..syls.len())
        .map(|k| {
            syls[k].accented
                && (k + 1..syls.len()).all(|j| phrase[j] != phrase[k] || !syls[j].accented)
                && matches!(phrase_end[phrase[k]], Some(End::Pause | End::Final))
        })
        .collect();
    let n_unacc = |a: usize, b: usize| (a + 1..b).filter(|&k| !syls[k].accented).count();

    let mut v: Vec<Values> = syls
        .iter()
        .map(|_| Values {
            start: 0.,
            peak: 0.,
            end: 0.,
            onset_boost: None,
            cr: 0.,
        })
        .collect();

    // Peaks, rises and falls of accented syllables.
    for (ai, &k) in accents.iter().enumerate() {
        let s = &syls[k];
        let t = mid(s.nucleus);
        let mut exc = top(t) - lower(t);
        if s.emph {
            exc *= EMPH_SCALE;
        }
        // Unaccented syllables before and after this accent.
        let before = match ai {
            0 => k,
            _ => n_unacc(accents[ai - 1], k),
        };
        let after = match accents.get(ai + 1) {
            Some(&n) => n_unacc(k, n),
            None => syls.len() - k - 1,
        };
        exc *= match before {
            0 | 1 => 1.0,
            2 => 0.85,
            _ => 0.75,
        };
        exc *= match after {
            0 | 1 => 1.0,
            2 => 1.10,
            _ => 1.15,
        };
        let peak = (lower(t) + exc).min(TOP_MAX);
        let mut rise = RISE * (peak - LOWER_START);
        let mut fall = FALL * (peak - lower(t));
        let p_initial = accents[..ai].iter().all(|&j| phrase[j] != phrase[k]);
        let p_final = accents[ai + 1..].iter().all(|&j| phrase[j] != phrase[k]);
        if p_initial {
            rise *= PHRASE_INITIAL_RISE;
        }
        if p_final {
            rise *= PHRASE_FINAL_RISE;
            fall *= PHRASE_FINAL_FALL;
        }
        if !p_initial && !p_final {
            rise *= MEDIAL;
            fall *= MEDIAL;
        }
        if ai > 0 {
            rise *= match before {
                0 => 0.6,
                1 => 1.0,
                2 => 1.15,
                3 => 1.2,
                _ => 1.3,
            };
        }
        let unvoiced_onset = (s.first..s.nucleus).any(|i| {
            let p = out[i].seg.phone;
            !voiced(&out[i]) && !p.is(Flags::SONORANT)
        });
        v[k] = Values {
            start: peak - rise,
            peak,
            end: peak - fall,
            onset_boost: unvoiced_onset.then(|| peak + UNVOICED_ONSET * (peak - LOWER_START)),
            cr: 0.,
        };
    }

    // Unaccented syllables.
    let set_line = |v: &mut Values, a: f32, b: f32| {
        v.start = a;
        v.end = b;
        v.peak = 0.5 * (a + b) + UNACCENTED_BUMP;
    };
    match accents.first() {
        None => {
            // No accents: fall along the lower line to the final value.
            let n = syls.len() as f32;
            for k in 0..syls.len() {
                let a = LOWER_START + (FINAL - LOWER_START) * k as f32 / n;
                let b = LOWER_START + (FINAL - LOWER_START) * (k + 1) as f32 / n;
                set_line(&mut v[k], a, b);
            }
        }
        Some(&first) => {
            // Before the first accent: level at its starting value.
            let s0 = v[first].start;
            for k in 0..first {
                set_line(&mut v[k], s0, s0);
            }
            // Between accents: straight or falling.
            for w in accents.windows(2) {
                let (a, b) = (w[0], w[1]);
                let (e1, s2) = (v[a].end, v[b].start);
                let n = (b - a - 1) as f32;
                if s2 > e1 {
                    // Split the difference between the fall and the rise, so
                    // that the unaccented syllables stay level.
                    let m = 0.5 * (e1 + s2);
                    v[a].end = m;
                    v[b].start = m;
                    for k in a + 1..b {
                        set_line(&mut v[k], m, m);
                    }
                } else {
                    // Linear fall. (The book makes it exponential across a
                    // phrase boundary; not done.)
                    for k in a + 1..b {
                        let j = (k - a - 1) as f32;
                        set_line(
                            &mut v[k],
                            e1 + (s2 - e1) * j / n,
                            e1 + (s2 - e1) * (j + 1.) / n,
                        );
                    }
                }
            }
            // After the last accent: fall in equal amounts to the final value.
            let last = *accents.last().unwrap();
            let n = syls.len() - last - 1;
            if n == 0 {
                v[last].end = FINAL;
            } else {
                let e = v[last].end;
                for k in last + 1..syls.len() {
                    let j = (k - last - 1) as f32;
                    let nn = n as f32;
                    set_line(
                        &mut v[k],
                        e + (FINAL - e) * j / nn,
                        e + (FINAL - e) * (j + 1.) / nn,
                    );
                }
            }
        }
    }

    // Continuation rises.
    let mut words_since_cr = 0;
    let mut last_word = usize::MAX;
    for (k, s) in syls.iter().enumerate() {
        if s.word != last_word {
            words_since_cr += 1;
            last_word = s.word;
        }
        match s.end {
            Some(End::Pause) => {
                v[k].cr = CR_PUNCT;
                words_since_cr = 0;
            }
            Some(End::Phrase) if words_since_cr > CR_WORDS => {
                v[k].cr = CR_PHRASE;
                words_since_cr = 0;
            }
            _ => {}
        }
    }

    // Place the targets on segments.
    let mut targets: Vec<(usize, f32, f32)> = vec![];
    for (k, s) in syls.iter().enumerate() {
        let val = &v[k];
        let nuc = s.nucleus;
        let next_is_syl_start = nuc == s.last;
        let mut put = |i: usize, pos: f32, f0: f32| targets.push((i, pos, f0));
        // Start: on the first voiced segment of the syllable.
        if let Some(b) = val.onset_boost {
            put(nuc, 0.0, b);
        } else if let Some(i) = (s.first..=nuc).find(|&i| voiced(&out[i])) {
            if i < nuc {
                put(i, 0.5, val.start);
            } else {
                put(nuc, 0.0, val.start);
            }
        }
        let has_left = val.onset_boost.is_some() || !(s.first..nuc).any(|i| voiced(&out[i]));
        // Peak.
        let peak_pos = if s.accented {
            if nuclear[k] || s.end.is_some() {
                0.0
            } else if next_is_syl_start {
                0.5
            } else {
                1.0
            }
        } else if s.end.is_some() {
            0.0
        } else {
            0.5
        };
        let peak_pos = if peak_pos == 0.0 && has_left {
            0.3
        } else {
            peak_pos
        };
        put(nuc, peak_pos, val.peak);
        // Fall (and continuation rise).
        let last_voiced = (nuc..=s.last).rev().find(|&i| voiced(&out[i])).unwrap_or(nuc);
        if s.end.is_some() {
            if val.cr > 0. {
                put(nuc, 0.75, val.end);
                put(last_voiced, 1.0, val.end + val.cr);
            } else {
                put(last_voiced, 1.0, val.end);
            }
        } else if next_is_syl_start {
            put(nuc, 1.0, val.end);
        } else {
            put(nuc + 1, 0.5, val.end);
        }
    }
    for (i, pos, f0) in targets {
        out[i].seg.f0.push((pos, f0));
    }
    // Keep each segment's targets in order.
    for a in out.iter_mut() {
        a.seg.f0.sort_by(|x, y| x.0.total_cmp(&y.0));
    }
}
