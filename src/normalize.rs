//! Text normalization: numbers, acronyms, abbreviations and punctuation
//! to words and phrase boundaries, ahead of letter-to-sound conversion.

/// A normalized token.
#[derive(Debug, PartialEq)]
pub enum Token {
    /// A lowercase word, apostrophes dropped.
    Word(String),
    /// Lowercase letters to spell out, and whether the last one takes a
    /// plural or possessive -s (CDs, FBI's).
    Spell(String, bool),
    /// `,` (phrase break), `.` or `?`.
    Punct(char),
}

/// Pronunciations of the words the normalizer produces, where the
/// letter-to-sound rules get them wrong. Added to the dictionary unless it
/// already has the word.
pub const WORDS: &[(&str, &str)] = &[
    ("zero", "z'iro"),
    ("one", "w'^n"),
    ("two", "t'u"),
    ("three", "Tr'i"),
    ("four", "f'cr"),
    ("five", "f'Av"),
    ("six", "s'Iks"),
    ("seven", "s'Evxn"),
    ("eight", "'et"),
    ("nine", "n'An"),
    ("ten", "t'En"),
    ("eleven", "xl'Evxn"),
    ("twelve", "tw'Elv"),
    ("thirteen", "T\"Rt'in"),
    ("fourteen", "f\"crt'in"),
    ("fifteen", "f\"Ift'in"),
    ("sixteen", "s\"Ikst'in"),
    ("seventeen", "s\"Evxnt'in"),
    ("eighteen", "\"et'in"),
    ("nineteen", "n\"Ant'in"),
    ("twenty", "tw'Enti"),
    ("thirty", "T'Rti"),
    ("forty", "f'crti"),
    ("fifty", "f'Ifti"),
    ("sixty", "s'Iksti"),
    ("seventy", "s'Evxnti"),
    ("eighty", "'eti"),
    ("ninety", "n'Anti"),
    ("hundred", "h'^ndrxd"),
    ("thousand", "T'Wzxnd"),
    ("million", "m'Ilyxn"),
    ("billion", "b'Ilyxn"),
    ("trillion", "tr'Ilyxn"),
    ("zeroth", "z'iroT"),
    ("first", "f'Rst"),
    ("second", "s'Ekxnd"),
    ("third", "T'Rd"),
    ("fourth", "f'crT"),
    ("fifth", "f'IfT"),
    ("sixth", "s'IksT"),
    ("seventh", "s'EvxnT"),
    ("eighth", "'etT"),
    ("ninth", "n'AnT"),
    ("tenth", "t'EnT"),
    ("eleventh", "xl'EvxnT"),
    ("twelfth", "tw'ElfT"),
    ("thirteenth", "T\"Rt'inT"),
    ("fourteenth", "f\"crt'inT"),
    ("fifteenth", "f\"Ift'inT"),
    ("sixteenth", "s\"Ikst'inT"),
    ("seventeenth", "s\"Evxnt'inT"),
    ("eighteenth", "\"et'inT"),
    ("nineteenth", "n\"Ant'inT"),
    ("twentieth", "tw'EntixT"),
    ("thirtieth", "T'RtixT"),
    ("fortieth", "f'crtixT"),
    ("fiftieth", "f'IftixT"),
    ("sixtieth", "s'IkstixT"),
    ("seventieth", "s'EvxntixT"),
    ("eightieth", "'etixT"),
    ("ninetieth", "n'AntixT"),
    ("hundredth", "h'^ndrxdT"),
    ("thousandth", "T'WzxndT"),
    ("millionth", "m'IlyxnT"),
    ("billionth", "b'IlyxnT"),
    ("trillionth", "tr'IlyxnT"),
    ("point", "p'Ont"),
    ("oh", "'o"),
    ("only", "'onli"), // not one + -ly
    ("oclock", "xkl'ak"),
    ("dollar", "d'alR"),
    ("cent", "s'Ent"),
    ("percent", "pRs'Ent"),
    ("euro", "y'Uro"),
    ("minus", "m'Anxs"),
    ("plus", "pl'^s"),
    ("equals", "'ikwxlz"),
    ("versus", "v'Rsxs"),
    ("doctor", "d'aktR"),
    ("mister", "m'IstR"),
    ("missus", "m'Isxz"),
    ("miz", "m'Iz"),
    ("saint", "s'ent"),
    ("street", "str'it"),
    ("et", "'Et"),
    ("cetera", "s'Etxrx"),
    ("junior", "J'unyR"),
    ("senior", "s'inyR"),
    ("professor", "prxf'EsR"),
    ("number", "n'^mbR"),
];

/// Letter names in ASCII phonemes, a to z, for spelled acronyms.
pub const LETTERS: [&str; 26] = [
    "'e", "b'i", "s'i", "d'i", "'i", "'Ef", "J'i", "'eC", "'A", "J'e", "k'e", "'El", "'Em",
    "'En", "'o", "p'i", "ky'u", "'ar", "'Es", "t'i", "y'u", "v'i", "d'^bxlyu", "'Eks", "w'A",
    "z'i",
];

const ONES: [&str; 20] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen",
    "nineteen",
];
const TENS: [&str; 10] = [
    "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];
const SCALES: [(u64, &str); 4] = [
    (1_000_000_000_000, "trillion"),
    (1_000_000_000, "billion"),
    (1_000_000, "million"),
    (1_000, "thousand"),
];

/// Cardinal number in words ("one hundred twenty three"); `None` past
/// the trillions.
fn cardinal(mut n: u64) -> Option<Vec<String>> {
    if n >= 1_000_000_000_000_000 {
        return None;
    }
    if n == 0 {
        return Some(vec!["zero".into()]);
    }
    let mut out = vec![];
    for (s, name) in SCALES {
        if n >= s {
            below_1000(n / s, &mut out);
            out.push(name.into());
            n %= s;
        }
    }
    below_1000(n, &mut out);
    Some(out)
}

fn below_1000(n: u64, out: &mut Vec<String>) {
    let (h, r) = (n / 100, n % 100);
    if h > 0 {
        out.push(ONES[h as usize].into());
        out.push("hundred".into());
    }
    if r >= 20 {
        out.push(TENS[(r / 10) as usize].into());
        if r % 10 > 0 {
            out.push(ONES[(r % 10) as usize].into());
        }
    } else if r > 0 {
        out.push(ONES[r as usize].into());
    }
}

/// A year as it is read: nineteen eighty four, nineteen oh five, two
/// thousand three, twenty ten.
fn year(n: u64) -> Vec<String> {
    let (hi, lo) = (n / 100, n % 100);
    if (2000..2010).contains(&n) {
        return cardinal(n).unwrap();
    }
    let mut out = cardinal(hi).unwrap();
    match lo {
        0 => out.push("hundred".into()),
        1..=9 => {
            out.push("oh".into());
            out.push(ONES[lo as usize].into());
        }
        _ => below_1000(lo, &mut out),
    }
    out
}

/// Ordinal of the last word of a cardinal (twenty one → twenty first).
fn ordinal(words: &mut [String]) {
    let w = words.last_mut().unwrap();
    *w = match w.as_str() {
        "one" => "first".into(),
        "two" => "second".into(),
        "three" => "third".into(),
        "five" => "fifth".into(),
        "eight" => "eighth".into(),
        "nine" => "ninth".into(),
        "twelve" => "twelfth".into(),
        s if s.ends_with('y') => format!("{}ieth", &s[..s.len() - 1]),
        s => format!("{s}th"),
    };
}

/// Plural of the last word (the 1990s → nineteen nineties).
fn plural(words: &mut [String]) {
    let w = words.last_mut().unwrap();
    *w = if let Some(s) = w.strip_suffix('y') {
        format!("{s}ies")
    } else if w.ends_with('x') {
        format!("{w}es")
    } else {
        format!("{w}s")
    };
}

/// Digit by digit (zip codes, numbers with a leading zero).
fn digits(s: &str) -> Vec<String> {
    s.chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| ONES[c as usize - '0' as usize].to_string())
        .collect()
}

/// Whether an all-caps word can be read as a word (NASA, NATO) rather than
/// spelled (FBI, HTML).
fn pronounceable(w: &str) -> bool {
    let b = w.as_bytes();
    if b.len() < 4 {
        return false;
    }
    let vowel = |i: usize| b"aeiou".contains(&b[i]) || (b[i] == b'y' && i > 0);
    let mut run_c = 0;
    let mut run_v = 0;
    let mut any_v = false;
    for i in 0..b.len() {
        if vowel(i) {
            any_v = true;
            run_v += 1;
            run_c = 0;
            if run_v > 2 {
                return false;
            }
        } else {
            run_c += 1;
            run_v = 0;
            // Initial and final clusters of at most two (st, nd); medial
            // of at most three.
            let limit = if i + 1 == b.len() || !any_v { 2 } else { 3 };
            if run_c > limit {
                return false;
            }
        }
    }
    if !any_v {
        return false;
    }
    // Final clusters (checked once the word is done).
    run_c <= 2
}

enum AbbrevEnd {
    /// Precedes a name or number: the period never ends a sentence.
    Never,
    /// Ends a sentence when followed by a capital or the end of text.
    Maybe,
}

/// Abbreviations with a period, lowercase, and their expansions.
fn abbreviation(w: &str, next_cap: bool, next_digit: bool) -> Option<(&'static str, AbbrevEnd)> {
    use AbbrevEnd::*;
    Some(match w {
        "mr" => ("mister", Never),
        "mrs" => ("missus", Never),
        "ms" => ("miz", Never),
        "dr" => ("doctor", Never),
        "prof" => ("professor", Never),
        "st" if next_cap => ("saint", Never),
        "st" => ("street", Maybe),
        "mt" => ("mount", Never),
        "jr" => ("junior", Maybe),
        "sr" => ("senior", Maybe),
        "vs" => ("versus", Never),
        "etc" => ("et cetera", Maybe),
        "inc" => ("incorporated", Maybe),
        "ltd" => ("limited", Maybe),
        "corp" => ("corporation", Maybe),
        "dept" => ("department", Maybe),
        "approx" => ("approximately", Never),
        "no" if next_digit => ("number", Never),
        "fig" => ("figure", Never),
        "vol" => ("volume", Never),
        "ave" => ("avenue", Maybe),
        "jan" => ("january", Never),
        "feb" => ("february", Never),
        "aug" => ("august", Never),
        "sep" | "sept" => ("september", Never),
        "oct" => ("october", Never),
        "nov" => ("november", Never),
        "dec" => ("december", Never),
        _ => return None,
    })
}

struct Out {
    toks: Vec<Token>,
}

impl Out {
    fn word(&mut self, w: &str) {
        for w in w.split(' ') {
            self.toks.push(Token::Word(w.into()));
        }
    }

    fn words(&mut self, ws: Vec<String>) {
        self.toks.extend(ws.into_iter().map(Token::Word));
    }

    /// Punctuation; of several in a row the strongest is kept, and none
    /// starts the text.
    fn punct(&mut self, p: char) {
        let rank = |p: char| if p == ',' { 1 } else { 2 };
        match self.toks.last_mut() {
            None => {}
            Some(Token::Punct(q)) => {
                if rank(p) > rank(*q) {
                    *q = p;
                }
            }
            _ => self.toks.push(Token::Punct(p)),
        }
    }
}

/// Normalize text to words, spelled acronyms and phrase boundaries.
pub fn normalize(text: &str) -> Vec<Token> {
    let cs: Vec<char> = text.chars().collect();
    let n = cs.len();
    let at = |i: usize| cs.get(i).copied().unwrap_or('\0');
    // All-caps text is not acronyms.
    let upper = cs.iter().filter(|c| c.is_uppercase()).count();
    let lower = cs.iter().filter(|c| c.is_lowercase()).count();
    let shouting = upper > 12 && upper > 2 * lower;
    // The next non-space character after i.
    let next_nonspace = |i: usize| cs[i.min(n)..].iter().copied().find(|c| *c != ' ');
    let next_cap = |i: usize| next_nonspace(i).is_some_and(|c| c.is_uppercase());
    let end_or_cap = |i: usize| next_nonspace(i).is_none_or(|c| c.is_uppercase() || c == '\n');

    let mut out = Out { toks: vec![] };
    let mut i = 0;
    while i < n {
        let c = cs[i];
        if c.is_ascii_alphabetic() {
            // Dotted letters: U.S.A., e.g., p.m.
            let mut j = i;
            let mut letters = String::new();
            while at(j).is_ascii_alphabetic() && at(j + 1) == '.' {
                letters.push(at(j).to_ascii_lowercase());
                j += 2;
            }
            if at(j).is_ascii_alphabetic() {
                letters.clear();
            }
            if letters.len() >= 2 {
                match letters.as_str() {
                    "eg" => out.word("for example"),
                    "ie" => out.word("that is"),
                    _ => out.toks.push(Token::Spell(letters.clone(), false)),
                }
                // The final period ends a sentence only at the end of text
                // or before a capital (not "U.S. Army", though).
                if letters != "us" && end_or_cap(j) || next_nonspace(j).is_none() {
                    out.punct('.');
                }
                i = j;
                continue;
            }

            // An ordinary word; apostrophes inside are dropped.
            let mut j = i;
            let mut w = String::new();
            while at(j).is_ascii_alphabetic()
                || matches!(at(j), '\'' | '’') && at(j + 1).is_ascii_alphabetic()
            {
                if at(j).is_ascii_alphabetic() {
                    w.push(at(j));
                }
                j += 1;
            }
            let lw = w.to_ascii_lowercase();
            if at(j) == '.' {
                let after = j + 1;
                let next_digit = next_nonspace(after).is_some_and(|c| c.is_ascii_digit());
                if let Some((exp, end)) = abbreviation(&lw, next_cap(after), next_digit) {
                    out.word(exp);
                    if matches!(end, AbbrevEnd::Maybe) && end_or_cap(after) {
                        out.punct('.');
                    }
                    i = after;
                    continue;
                }
                // A middle initial: John F. Kennedy.
                if w.len() == 1 && w.chars().all(|c| c.is_ascii_uppercase()) && next_cap(after) {
                    out.toks.push(Token::Spell(lw, false));
                    i = after;
                    continue;
                }
            }
            // Acronyms: all capitals, perhaps with a plural or possessive s.
            let (stem, s) = match w.strip_suffix('s') {
                Some(st) if st.len() >= 2 && st.chars().all(|c| c.is_ascii_uppercase()) => (st, true),
                _ => (w.as_str(), false),
            };
            let caps = stem.len() >= 2 && stem.chars().all(|c| c.is_ascii_uppercase());
            if caps && !shouting && !(pronounceable(&stem.to_ascii_lowercase()) && !s) {
                out.toks.push(Token::Spell(stem.to_ascii_lowercase(), s));
            } else if lw.len() == 1 && lw != "a" && lw != "i" {
                out.toks.push(Token::Spell(lw, false));
            } else {
                out.toks.push(Token::Word(lw));
            }
            i = j;
            continue;
        }

        if c.is_ascii_digit() {
            i = number(&cs, i, &mut out);
            continue;
        }

        match c {
            ',' | ';' | ':' => out.punct(','),
            '.' | '!' => out.punct('.'),
            '?' => out.punct('?'),
            '(' | ')' | '[' | ']' | '{' | '}' | '—' | '–' | '…' => out.punct(','),
            '-' => {
                let mut j = i;
                while at(j) == '-' {
                    j += 1;
                }
                let space_before = i == 0 || at(i - 1).is_whitespace();
                if j - i >= 2 || space_before && (at(j).is_whitespace() || at(j) == '\0') {
                    out.punct(',');
                } else if space_before && at(j).is_ascii_digit() {
                    out.word("minus");
                }
                // Otherwise inside a word (well-known): a word break.
                i = j;
                continue;
            }
            '$' | '£' | '€' if at(i + 1).is_ascii_digit() => {
                i = number(&cs, i, &mut out);
                continue;
            }
            '&' => out.word("and"),
            '+' => out.word("plus"),
            '=' => out.word("equals"),
            '@' => out.word("at"),
            '%' => out.word("percent"),
            '#' if at(i + 1).is_ascii_digit() => out.word("number"),
            '\n' if at(i + 1) == '\n' => out.punct('.'),
            _ => {}
        }
        i += 1;
    }
    out.punct('.');
    if !matches!(out.toks.last(), Some(Token::Punct('.' | '?'))) {
        out.toks.push(Token::Punct('.'));
    }
    out.toks
}

/// A number at `cs[i]` (with any currency sign, ordinal or plural suffix,
/// percent sign or range dash). Returns the index after it.
fn number(cs: &[char], mut i: usize, out: &mut Out) -> usize {
    let at = |i: usize| cs.get(i).copied().unwrap_or('\0');
    let currency = match at(i) {
        '$' => Some(("dollar", "dollars", "cent", "cents")),
        '£' => Some(("pound", "pounds", "penny", "pence")),
        '€' => Some(("euro", "euros", "cent", "cents")),
        _ => None,
    };
    if currency.is_some() {
        i += 1;
    }
    let start = i;
    // Integer part, with thousands separators.
    let mut int = String::new();
    let mut commas = false;
    loop {
        while at(i).is_ascii_digit() {
            int.push(at(i));
            i += 1;
        }
        // The first group has 1 to 3 digits, later ones exactly 3.
        if at(i) == ','
            && (if commas { true } else { (1..=3).contains(&int.len()) })
            && (1..=3).all(|k| at(i + k).is_ascii_digit())
            && !at(i + 4).is_ascii_digit()
        {
            commas = true;
            i += 1;
            continue;
        }
        break;
    }
    let mut frac = None;
    let mut minutes = None;
    if at(i) == '.' && at(i + 1).is_ascii_digit() {
        let mut f = String::new();
        i += 1;
        while at(i).is_ascii_digit() {
            f.push(at(i));
            i += 1;
        }
        frac = Some(f);
    } else if at(i) == ':'
        && !commas
        && int.len() <= 2
        && at(i + 1).is_ascii_digit()
        && at(i + 2).is_ascii_digit()
        && !at(i + 3).is_ascii_digit()
    {
        minutes = Some(10 * (at(i + 1) as u64 - '0' as u64) + (at(i + 2) as u64 - '0' as u64));
        i += 3;
    }
    // Phone numbers and the like: groups of digits joined by dashes.
    if currency.is_none() && frac.is_none() && minutes.is_none() && !commas && at(i) == '-' {
        let mut j = i;
        let mut groups = vec![int.len()];
        while at(j) == '-' && at(j + 1).is_ascii_digit() {
            let k = j + 1;
            j = k;
            while at(j).is_ascii_digit() {
                j += 1;
            }
            groups.push(j - k);
        }
        if groups.len() >= 2 && groups[1..].iter().all(|g| *g >= 3) && groups.iter().sum::<usize>() >= 7 {
            for (g, chunk) in cs[start..j].split(|c| *c == '-').enumerate() {
                if g > 0 {
                    out.punct(',');
                }
                out.words(digits(&chunk.iter().collect::<String>()));
            }
            return j;
        }
    }
    let suffix: String = cs[i..]
        .iter()
        .take_while(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let digits_int: String = int.chars().filter(|c| *c != ',').collect();
    let value: Option<u64> = digits_int.parse().ok();

    let mut words = if let Some(m) = minutes {
        let mut w = cardinal(value.unwrap()).unwrap();
        match m {
            0 => w.push("oclock".into()),
            1..=9 => {
                w.push("oh".into());
                w.push(ONES[m as usize].into());
            }
            _ => below_1000(m, &mut w),
        }
        w
    } else if digits_int.len() > 1 && digits_int.starts_with('0') && currency.is_none() {
        digits(&digits_int)
    } else {
        match value.and_then(cardinal) {
            Some(w) => w,
            None => digits(&digits_int),
        }
    };
    let v = value.unwrap_or(0);
    let is_year = currency.is_none()
        && frac.is_none()
        && minutes.is_none()
        && !commas
        && digits_int.len() == 4
        && (1100..2000).contains(&v) | (2010..2100).contains(&v)
        && !matches!(suffix.as_str(), "st" | "nd" | "rd" | "th");
    if is_year {
        words = year(v);
    }
    match suffix.as_str() {
        "st" | "nd" | "rd" | "th" if frac.is_none() => {
            ordinal(&mut words);
            i += 2;
        }
        "s" if frac.is_none() => {
            plural(&mut words);
            i += 1;
        }
        _ => {}
    }
    if let Some((one, many, sub_one, sub_many)) = currency {
        let cents = frac.as_ref().filter(|f| f.len() == 2).map(|f| f.parse::<u64>().unwrap());
        if let (Some(c), false) = (cents, v == 0 && cents.is_some_and(|c| c > 0)) {
            out.words(words);
            out.word(if v == 1 { one } else { many });
            if c > 0 {
                out.word("and");
                out.words(cardinal(c).unwrap());
                out.word(if c == 1 { sub_one } else { sub_many });
            }
            return i;
        }
        if let Some(c) = cents {
            out.words(cardinal(c).unwrap());
            out.word(if c == 1 { sub_one } else { sub_many });
            return i;
        }
        if let Some(f) = &frac {
            words.push("point".into());
            words.extend(digits(f));
        }
        out.words(words);
        // $3 million: three million dollars.
        let mut j = i;
        while at(j) == ' ' {
            j += 1;
        }
        let w: String = cs[j..].iter().take_while(|c| c.is_ascii_alphabetic()).collect();
        if matches!(w.to_ascii_lowercase().as_str(), "thousand" | "million" | "billion" | "trillion") {
            out.word(&w.to_ascii_lowercase());
            i = j + w.len();
        }
        out.word(if v == 1 && frac.is_none() { one } else { many });
        return i;
    }
    if let Some(f) = &frac {
        words.push("point".into());
        words.extend(digits(f));
    }
    out.words(words);
    let mut j = i;
    while at(j) == ' ' {
        j += 1;
    }
    if at(j) == '%' {
        out.word("percent");
        i = j + 1;
    } else if at(i) == '-' && at(i + 1).is_ascii_digit() {
        // A range: 1990-1995, pages 5-10.
        out.word("to");
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm(s: &str) -> String {
        normalize(s)
            .iter()
            .map(|t| match t {
                Token::Word(w) => w.clone(),
                Token::Spell(l, s) => format!("<{l}{}>", if *s { "+s" } else { "" }),
                Token::Punct(p) => p.to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn numbers() {
        assert_eq!(norm("123"), "one hundred twenty three .");
        assert_eq!(norm("1,234,567"), "one million two hundred thirty four thousand five hundred sixty seven .");
        assert_eq!(norm("in 1984 and 2005 and 1905 and 1900"),
            "in nineteen eighty four and two thousand five and nineteen oh five and nineteen hundred .");
        assert_eq!(norm("the 1990s"), "the nineteen nineties .");
        assert_eq!(norm("the 21st and 2nd and 40th"), "the twenty first and second and fortieth .");
        assert_eq!(norm("3.14"), "three point one four .");
        assert_eq!(norm("$5"), "five dollars .");
        assert_eq!(norm("$1.50"), "one dollar and fifty cents .");
        assert_eq!(norm("$0.25"), "twenty five cents .");
        assert_eq!(norm("$3 million"), "three million dollars .");
        assert_eq!(norm("50%"), "fifty percent .");
        assert_eq!(norm("at 3:30 or 4:00"), "at three thirty or four oclock .");
        assert_eq!(norm("pages 5-10"), "pages five to ten .");
        assert_eq!(norm("call 555-1234"), "call five five five , one two three four .");
        assert_eq!(norm("it is -5 out"), "it is minus five out .");
        assert_eq!(norm("007"), "zero zero seven .");
    }

    #[test]
    fn acronyms() {
        assert_eq!(norm("The FBI and NASA"), "the <fbi> and nasa .");
        assert_eq!(norm("two CDs"), "two <cd+s> .");
        assert_eq!(norm("the U.S. Army"), "the <us> army .");
        assert_eq!(norm("in the U.S.A. Then"), "in the <usa> . then .");
        assert_eq!(norm("HTML"), "<html> .");
        assert_eq!(norm("plan B"), "plan <b> .");
        assert_eq!(norm("THIS IS ALL IN CAPITALS"), "this is all in capitals .");
    }

    #[test]
    fn abbreviations() {
        assert_eq!(norm("Mr. Smith met Dr. Jones."), "mister smith met doctor jones .");
        assert_eq!(norm("St. Paul on Main St. today"), "saint paul on main street today .");
        assert_eq!(norm("apples, pears, etc. They"), "apples , pears , et cetera . they .");
        assert_eq!(norm("fruit, e.g. apples"), "fruit , for example apples .");
        assert_eq!(norm("John F. Kennedy"), "john <f> kennedy .");
    }

    #[test]
    fn punctuation() {
        assert_eq!(norm("one (two) three"), "one , two , three .");
        assert_eq!(norm("one -- two — three - four"), "one , two , three , four .");
        assert_eq!(norm("well-known"), "well known .");
        assert_eq!(norm("end. (Next)"), "end . next .");
        assert_eq!(norm("(Start) here"), "start , here .");
    }
}
