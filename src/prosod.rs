//! Segmental durations, following PROSOD (chapter 9 of the MITalk book).
//!
//! The input is the PHONO2 symbol stream of table 8-1, for example
//! `F: DH IY C: 'OW LX DD C: MM 'AE NN )N C: SS 'AE DX ...`, and the output
//! is a sequence of segments with durations, ready for PHONET.

use crate::{
    phones::{Flags, Phone, phone_duration},
    phonet::Segment,
};

/// F0 used until the F0 rules of chapter 10 are implemented.
const FLAT_F0: f32 = 120.0;

/// Silence at the start of an utterance (as in figure 9-1).
const INITIAL_SILENCE: u32 = 30;
/// Silence at the end of an utterance (as in figure 9-1).
const FINAL_SILENCE: u32 = 400;
/// Rule 1: pause inserted at a comma or clause boundary.
const PAUSE: u32 = 200;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Boundary {
    /// End of a phrase, but no pause (`)N`, `)P`).
    Phrase,
    /// End of a phrase, followed by a pause (`,`, `)C`, `.`, `)?`).
    Pause,
}

#[derive(Debug)]
struct Seg {
    phone: Phone,
    /// 0 = unstressed, 1 = primary, 2 = secondary (of the segment itself,
    /// for syllabics; of the syllable for onset consonants).
    stress: u8,
    /// A `-` syllable boundary precedes this segment.
    syll_break: bool,
    // Computed:
    syllable: usize,
    onset: bool,
}

#[derive(Debug)]
struct Word {
    /// Index of the first segment.
    start: usize,
    end: usize,
    boundary: Option<Boundary>,
}

/// A segment with its computed duration and the factors that produced it,
/// for checking against the book.
pub struct Annotated {
    pub seg: Segment,
    pub note: String,
}

/// Parse a PHONO2 stream and compute durations.
pub fn prosod(text: &str) -> Result<Vec<Annotated>, String> {
    let mut segs: Vec<Seg> = vec![];
    let mut words: Vec<Word> = vec![];
    let mut syll_break = false;
    for tok in text.split_whitespace() {
        match tok {
            "C:" | "F:" => {
                if let Some(w) = words.last_mut() {
                    w.end = segs.len();
                }
                words.push(Word {
                    start: segs.len(),
                    end: segs.len(),
                    boundary: None,
                });
            }
            "-" => syll_break = true,
            "*" => {}
            ")N" | ")P" | ")R" => set_boundary(&mut words, Boundary::Phrase)?,
            "," | ")C" | "." | ")?" => set_boundary(&mut words, Boundary::Pause)?,
            _ => {
                let (stress, name) = match tok.as_bytes()[0] {
                    b'\'' => (1, &tok[1..]),
                    b'"' => (2, &tok[1..]),
                    _ => (0, tok),
                };
                let phone = Phone::parse(&name.to_ascii_lowercase())
                    .ok_or_else(|| format!("unknown phone {tok}"))?;
                if words.is_empty() {
                    return Err(format!("{tok} is not in a word (missing C: or F:)"));
                }
                segs.push(Seg {
                    phone,
                    stress,
                    syll_break,
                    syllable: 0,
                    onset: false,
                });
                syll_break = false;
            }
        }
    }
    if let Some(w) = words.last_mut() {
        w.end = segs.len();
        w.boundary.get_or_insert(Boundary::Pause);
    }
    syllabify(&mut segs, &words);

    let syllabic = |i: usize| segs[i].phone.is(Flags::SYLLABIC);
    let n_syll = |w: &Word| (w.start..w.end).filter(|&i| syllabic(i)).count();

    // Which words end a phrase, and which end before a pause, looking through
    // to the next boundary.
    let mut out = vec![Annotated {
        seg: silence(INITIAL_SILENCE),
        note: String::new(),
    }];
    for (wi, word) in words.iter().enumerate() {
        let nsyl = n_syll(word);
        let last_syl = nsyl.saturating_sub(1);
        for i in word.start..word.end {
            let s = &segs[i];
            let phone = s.phone;
            let Some(d) = phone_duration(phone) else {
                return Err(format!("no duration for {phone:?}"));
            };
            let inhdur = d.inherent_duration as f32;
            let mut mindur = d.min_duration as f32;
            let mut prcnt = 100.0f32;
            let mut notes = vec![];
            let mut apply = |p: f32, rule: &str, prcnt: &mut f32| {
                *prcnt = *prcnt * p / 100.;
                notes.push(format!("{rule}:{p}"));
            };
            let is_syl = syllabic(i);
            let final_syl = s.syllable == last_syl;
            let phrase_final = word.boundary.is_some() && final_syl;
            let pause_final = word.boundary == Some(Boundary::Pause) && final_syl;
            // Consonants after the nucleus of the last syllable.
            let coda = !is_syl && !s.onset;

            // Rule 2: clause-final lengthening.
            if pause_final && (is_syl || coda) {
                apply(140., "2", &mut prcnt);
            }
            // Rule 3: non-phrase-final shortening.
            if is_syl && !phrase_final {
                apply(60., "3", &mut prcnt);
            }
            if phrase_final
                && coda
                && (phone.is(Flags::NASAL) || phone.is(Flags::LIQGLIDE) && !is_glide(phone))
            {
                apply(140., "3b", &mut prcnt);
            }
            // Rule 4: non-word-final shortening.
            if is_syl && !final_syl {
                apply(85., "4", &mut prcnt);
            }
            // Rule 5: polysyllabic shortening.
            if is_syl && nsyl > 1 {
                apply(80., "5", &mut prcnt);
            }
            // Rule 6: non-initial-consonant shortening.
            let word_initial = !is_syl && s.syllable == 0 && s.onset;
            if !is_syl && !word_initial {
                apply(85., "6", &mut prcnt);
            }
            // Rule 7: unstressed shortening.
            if s.stress != 1 {
                if s.stress == 0 {
                    mindur /= 2.;
                }
                let medial = s.syllable > 0 && s.syllable < last_syl;
                let prevocalic_liq = phone.is(Flags::LIQGLIDE)
                    && segs.get(i + 1).is_some_and(|n| n.phone.is(Flags::SYLLABIC));
                let p = if is_syl && medial {
                    50.
                } else if is_syl {
                    70.
                } else if prevocalic_liq {
                    10.
                } else {
                    70.
                };
                apply(p, "7", &mut prcnt);
            }
            // Rule 8 (emphasis) is not implemented: there is no symbol for it.
            // Rule 9: postvocalic context of vowels.
            if is_syl {
                let p = postvocalic(&segs, i, word.end);
                let p = if phrase_final { p } else { 70. + 0.3 * p };
                apply(p, "9", &mut prcnt);
            } else if coda && phone.is(Flags::SONORANT) {
                // The obstruent of a postvocalic sonorant-obstruent cluster
                // also affects the sonorant.
                if let Some(nucleus) = (word.start..i).rev().find(|&j| syllabic(j))
                    && i + 1 < word.end
                    && !segs[i + 1].phone.is(Flags::SONORANT)
                {
                    let p = postvocalic(&segs, nucleus, word.end);
                    let p = if phrase_final { p } else { 70. + 0.3 * p };
                    apply(p, "9", &mut prcnt);
                }
            }
            // Rule 10: shortening in clusters, across words but not across
            // phrase boundaries.
            let phrase_start = (0..wi).rev().find(|&w| words[w].boundary.is_some());
            let lo = phrase_start.map(|w| words[w].end).unwrap_or(0);
            let hi = words[wi..]
                .iter()
                .find(|w| w.boundary.is_some())
                .map(|w| w.end)
                .unwrap_or(segs.len());
            let prev = (i > lo).then(|| syllabic(i - 1));
            let next = (i + 1 < hi).then(|| syllabic(i + 1));
            if is_syl {
                if next == Some(true) {
                    apply(120., "10", &mut prcnt);
                }
                if prev == Some(true) {
                    apply(70., "10", &mut prcnt);
                }
            } else {
                match (prev, next) {
                    (Some(false), Some(false)) => apply(50., "10", &mut prcnt),
                    (Some(false), _) | (_, Some(false)) => apply(70., "10", &mut prcnt),
                    _ => {}
                }
            }
            // Equation 1
            let mut dur = (inhdur - mindur) * prcnt / 100. + mindur;
            // Rule 11: lengthening due to plosive aspiration.
            if s.stress != 0
                && (is_syl || phone.is(Flags::SONORANT))
                && i > 0
                && segs[i - 1].phone.is(Flags::PLOSIVE)
                && !segs[i - 1].phone.is(Flags::VOICED)
            {
                dur += 25.;
                notes.push("11".into());
            }
            // Round up to the nearest 5 ms.
            let dur = ((dur / 5.).ceil() * 5.) as u32;
            out.push(Annotated {
                seg: Segment {
                    phone,
                    dur,
                    f0: FLAT_F0,
                    stressed: s.stress == 1,
                },
                note: notes.join(" "),
            });
        }
        match word.boundary {
            Some(Boundary::Pause) if wi + 1 < words.len() => out.push(Annotated {
                seg: silence(PAUSE),
                note: "1".into(),
            }),
            _ => {}
        }
    }
    out.push(Annotated {
        seg: silence(FINAL_SILENCE),
        note: String::new(),
    });
    Ok(out)
}

fn set_boundary(words: &mut [Word], b: Boundary) -> Result<(), String> {
    let w = words.last_mut().ok_or("boundary before any word")?;
    // A pause boundary wins over a phrase boundary.
    if w.boundary != Some(Boundary::Pause) {
        w.boundary = Some(b);
    }
    Ok(())
}

fn silence(dur: u32) -> Segment {
    Segment {
        phone: Phone::Sil,
        dur,
        f0: FLAT_F0,
        stressed: false,
    }
}

fn is_glide(phone: Phone) -> bool {
    matches!(phone, Phone::Ww | Phone::Wh | Phone::Yy)
}

/// Assign segments to syllables within each word. Consonants between two
/// nuclei go to the following syllable (maximal onset) unless a `-`
/// boundary says otherwise.
fn syllabify(segs: &mut [Seg], words: &[Word]) {
    for w in words {
        let nuclei: Vec<usize> = (w.start..w.end)
            .filter(|&i| segs[i].phone.is(Flags::SYLLABIC))
            .collect();
        for i in w.start..w.end {
            let syl = nuclei.iter().position(|&n| n >= i);
            let (syllable, onset) = match syl {
                // At or before a nucleus.
                Some(k) => {
                    if nuclei[k] == i {
                        (k, false)
                    } else if k > 0 && (nuclei[k - 1] + 1..=i).all(|j| !segs[j].syll_break) {
                        // An intervocalic consonant with no `-` between it
                        // and the previous nucleus: onset of syllable k...
                        // unless a `-` follows it before the next nucleus.
                        if (i + 1..=nuclei[k]).any(|j| segs[j].syll_break) {
                            (k - 1, false)
                        } else {
                            (k, true)
                        }
                    } else {
                        (k, true)
                    }
                }
                // After the last nucleus (or no nucleus at all).
                None => (nuclei.len().saturating_sub(1), nuclei.is_empty()),
            };
            segs[i].syllable = syllable;
            segs[i].onset = onset;
            if onset {
                let n = nuclei.get(syllable).copied();
                segs[i].stress = n.map(|n| segs[n].stress).unwrap_or(0);
            } else if !segs[i].phone.is(Flags::SYLLABIC) {
                segs[i].stress = 0;
            }
        }
    }
}

/// Rule 9a: effect of the postvocalic consonant(s) in the same word on the
/// vowel at `i`.
fn postvocalic(segs: &[Seg], i: usize, word_end: usize) -> f32 {
    let mut j = i + 1;
    // Skip sonorants: in a sonorant-obstruent cluster the obstruent decides.
    while j < word_end
        && segs[j].phone.is(Flags::SONORANT)
        && !segs[j].phone.is(Flags::NASAL)
        && !segs[j].phone.is(Flags::SYLLABIC)
        && j + 1 < word_end
        && !segs[j + 1].phone.is(Flags::SONORANT)
    {
        j += 1;
    }
    if j >= word_end {
        return 120.;
    }
    let c = segs[j].phone;
    if c.is(Flags::SYLLABIC) {
        100.
    } else if c.is(Flags::FRICATIVE) && c.is(Flags::VOICED) {
        160.
    } else if c.is(Flags::PLOSIVE) && c.is(Flags::VOICED) {
        120.
    } else if c.is(Flags::NASAL) {
        85.
    } else if c.is(Flags::PLOSIVE) {
        70.
    } else {
        100.
    }
}
