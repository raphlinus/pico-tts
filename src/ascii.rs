//! ASCII phoneme input: the one-character DECtalk alphabet ("ASCKY"), with
//! MITalk stress and boundary marks, converted to the PHONO2 stream that
//! [`crate::prosod`] reads.
//!
//! Example: `F: Di C: 'old C: m'@n )N C: s'@& F: In F: x C: r'ak-R .`
//!
//! Within a word, `'` and `"` mark primary and secondary stress on the next
//! syllabic, `-` is a syllable boundary, `*` a morpheme boundary, and `!`
//! marks the word as emphasized. Words are separated by whitespace and are
//! content words unless preceded by `F:`. Boundaries are `)N`, `)P`, `)C`,
//! `,`, `.` and `?` (or `)?`), either as separate tokens or attached to the
//! end of a word.
//!
//! Some DECtalk symbols stand for several MITalk phones, so a small allophone
//! pass picks one from context:
//!
//! - `R` after `I`, `E`, `a`, `c`, `U` merges into IXR, EXR, AXR, OXR, UXR;
//!   after another vowel it is RX, before a vowel RR, otherwise ER.
//! - `l` before a vowel is LL, otherwise LX.
//! - `Y` before a vowel is the glide YY, otherwise YU. `y` is always YY.

/// Convert ASCII phonemes to PHONO2.
pub fn ascii_to_phono2(text: &str) -> Result<String, String> {
    let mut out: Vec<String> = vec![];
    let mut pending_marker: Option<&str> = None;
    for tok in text.split_whitespace() {
        match tok {
            "C:" | "F:" => {
                pending_marker = Some(tok);
                continue;
            }
            ")N" | ")P" | ")R" | ")C" | ")?" | "," | "." => {
                out.push(tok.into());
                continue;
            }
            "?" => {
                out.push(")?".into());
                continue;
            }
            _ => {}
        }
        // Split off trailing punctuation.
        let mut word = tok;
        let mut trailing = vec![];
        while let Some(c) = word.chars().last() {
            let b = match c {
                ',' => ",",
                '.' => ".",
                '?' => ")?",
                _ => break,
            };
            trailing.push(b);
            word = &word[..word.len() - 1];
        }
        out.push(pending_marker.take().unwrap_or("C:").into());
        out.extend(convert_word(word).map_err(|e| format!("{e} in {tok}"))?);
        out.extend(trailing.into_iter().rev().map(String::from));
    }
    if pending_marker.is_some() {
        return Err("word marker at end of input".into());
    }
    Ok(out.join(" "))
}

/// An element of a word before the allophone pass.
enum El {
    Phone { c: char, stress: u8 },
    Mark(&'static str),
}

fn is_vowel(c: char) -> bool {
    "iIeE@aAW^coOUux|YR".contains(c)
}

fn convert_word(word: &str) -> Result<Vec<String>, String> {
    let mut els = vec![];
    let mut stress = 0;
    let mut emph = false;
    for c in word.chars() {
        match c {
            '\'' => stress = 1,
            '"' => stress = 2,
            '!' => emph = true,
            '-' => els.push(El::Mark("-")),
            '*' => els.push(El::Mark("*")),
            _ => {
                // Stress marks apply to the next syllabic.
                let s = if is_vowel(c) {
                    std::mem::take(&mut stress)
                } else {
                    0
                };
                els.push(El::Phone { c, stress: s });
            }
        }
    }
    if stress != 0 {
        return Err("stress mark with no following vowel".into());
    }
    // Neighboring phone characters, skipping boundary marks.
    let phone_at = |i: usize| match els.get(i) {
        Some(El::Phone { c, .. }) => Some(*c),
        _ => None,
    };
    let next_phone = |i: usize| (i + 1..els.len()).find_map(phone_at);
    let prev_phone = |i: usize| (0..i).rev().find_map(phone_at);
    // A vowel before R is either merged with it or makes it RX, so the next
    // "real" vowel is one that is not itself R.
    let vowel_next = |i: usize| next_phone(i).is_some_and(|c| is_vowel(c) && c != 'R');

    let mut out = vec![];
    if emph {
        out.push("!".to_string());
    }
    let mut skip = false;
    for (i, el) in els.iter().enumerate() {
        if std::mem::take(&mut skip) {
            continue;
        }
        let (c, s) = match el {
            El::Mark(m) => {
                out.push(m.to_string());
                continue;
            }
            El::Phone { c, stress } => (*c, *stress),
        };
        let name = match c {
            'R' => match prev_phone(i) {
                // A syllable break before R makes it syllabic (earli-er).
                _ if matches!(i.checked_sub(1).map(|p| &els[p]), Some(El::Mark("-"))) => "ER",
                Some(p) if is_vowel(p) && p != 'R' => {
                    if vowel_next(i) {
                        "RR"
                    } else {
                        "RX"
                    }
                }
                // Unstressed R before a weak vowel loses its syllable
                // (every, memory), but not before a full vowel (operate).
                _ if vowel_next(i)
                    && s == 0
                    && next_phone(i).is_some_and(|c| "iIx".contains(c)) =>
                {
                    "RR"
                }
                _ => "ER",
            },
            'I' | 'i' | 'E' | 'a' | 'c' | 'o' | 'U' | 'u'
                if phone_at(i + 1) == Some('R') && !vowel_next(i + 1) =>
            {
                skip = true;
                match c {
                    'I' | 'i' => "IXR",
                    'o' => "OXR",
                    'E' => "EXR",
                    'a' => "AXR",
                    'c' => "OXR",
                    _ => "UXR",
                }
            }
            'l' => {
                if vowel_next(i) {
                    "LL"
                } else {
                    "LX"
                }
            }
            'Y' => {
                if vowel_next(i) {
                    "YY"
                } else {
                    "YU"
                }
            }
            _ => simple(c).ok_or_else(|| format!("unknown phoneme '{c}'"))?,
        };
        let prefix = match s {
            1 => "'",
            2 => "\"",
            _ => "",
        };
        out.push(format!("{prefix}{name}"));
    }
    Ok(out)
}

/// One-to-one symbols.
fn simple(c: char) -> Option<&'static str> {
    Some(match c {
        'i' => "IY",
        'I' => "IH",
        'e' => "EY",
        'E' => "EH",
        '@' => "AE",
        'a' => "AA",
        'A' => "AY",
        'W' => "AW",
        '^' => "AH",
        'c' => "AO",
        'o' => "OW",
        'O' => "OY",
        'U' => "UH",
        'u' => "UW",
        'x' => "AX",
        '|' => "IX",
        'y' => "YY",
        'w' => "WW",
        'r' => "RR",
        'h' => "HH",
        'm' => "MM",
        'n' => "NN",
        'G' => "NG",
        'L' => "EL",
        'N' => "EN",
        'f' => "FF",
        'v' => "VV",
        'T' => "TH",
        'D' => "DH",
        's' => "SS",
        'z' => "ZZ",
        'S' => "SH",
        'Z' => "ZH",
        'p' => "PP",
        'b' => "BB",
        't' => "TT",
        'd' => "DD",
        'k' => "KK",
        'g' => "GG",
        '&' | 'F' => "DX",
        'Q' => "TQ",
        'q' => "QQ",
        'C' => "CH",
        'J' => "JJ",
        '_' => "SIL",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_man() {
        let p = ascii_to_phono2("F: Di C: 'old C: m'@n )N C: s'@& F: In F: x C: r'ak*-R.").unwrap();
        assert_eq!(
            p,
            "F: DH IY C: 'OW LX DD C: MM 'AE NN )N C: SS 'AE DX F: IH NN F: AX C: RR 'AA KK * - ER ."
        );
    }

    #[test]
    fn allophones() {
        let p = ascii_to_phono2("b'IR v'Eri k'aR b'Rd l'Yt Y'@Gk").unwrap();
        assert_eq!(
            p,
            "C: BB 'IXR C: VV 'EH RR IY C: KK 'AXR C: BB 'ER DD C: LL 'YU TT C: YY 'AE NG KK"
        );
    }
}
