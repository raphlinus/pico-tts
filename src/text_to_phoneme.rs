//! A simple, low-resource text to phoneme implementation

use std::collections::HashMap;

use bitflags::bitflags;

pub struct TextToPhoneme {
    alpha_rules: [Vec<Rule>; 26],
    dict: HashMap<&'static str, &'static str>,
}

#[derive(Debug)]
pub struct Rule {
    pre: &'static str,
    body: &'static str,
    post: &'static str,
    output: &'static str,
}

bitflags! {
    #[derive(Clone, Copy)]
    struct Flags: u8 {
        const VOWEL = 1;
        const FRONT_VOWEL = 2;
        const VOICED_CONSONANT = 4;
        const SIBILANT = 8;
        const LONG_U = 0x10;
    }
}

const ALPHA_FLAGS: [Flags; 26] = [
    Flags::VOWEL,                                 // a
    Flags::VOICED_CONSONANT,                      // b
    Flags::SIBILANT,                              // c
    Flags::VOICED_CONSONANT.union(Flags::LONG_U), // d
    Flags::VOWEL.union(Flags::FRONT_VOWEL),       // e
    Flags::empty(),                               // f
    Flags::VOICED_CONSONANT.union(Flags::LONG_U), // g
    Flags::empty(),                               // h
    Flags::VOWEL.union(Flags::FRONT_VOWEL),       // i
    Flags::VOICED_CONSONANT
        .union(Flags::SIBILANT)
        .union(Flags::LONG_U), // j
    Flags::empty(),                               // k
    Flags::VOICED_CONSONANT.union(Flags::LONG_U), // l
    Flags::VOICED_CONSONANT,                      // m
    Flags::VOICED_CONSONANT.union(Flags::LONG_U), // n
    Flags::VOWEL,                                 // o
    Flags::empty(),                               // p
    Flags::empty(),                               // q
    Flags::VOICED_CONSONANT.union(Flags::LONG_U), // r
    Flags::SIBILANT.union(Flags::LONG_U),         // s
    Flags::LONG_U,                                // t
    Flags::VOWEL,                                 // u
    Flags::VOICED_CONSONANT,                      // v
    Flags::VOICED_CONSONANT,                      // w
    Flags::SIBILANT,                              // x
    Flags::VOWEL.union(Flags::FRONT_VOWEL),       // y
    Flags::VOICED_CONSONANT.union(Flags::LONG_U), // z
];

// These rules were adapted closely from Elovitz et al. with some tweaks.
// The output is a concise IPA, following the mapping in Table 1 with some
// exceptions: the mapping for AA is ɑ rather than a, and WH is ʍ.
//
// The mappings for OW and UW are a bit controversial. Here I've followed
// the table, which gives o and u respectively. In standard IPA, these would
// generally be expanded to oʊ and uː. However, Geoff Lindsay suggests a
// better expansion would be ow (or əw) and uw (or ʉw). Similarly, I map IY
// to i, while standard IPA might call for iː, and Lindsay suggests ɪj.
//
// I also follow Lindsey's lead to map AW to aw rather than aʊ as in
// standard IPA.
//
// I follow the standard by transcribing AY to aɪ, OY to ɔɪ and EY to ɛɪ.
// Linsdey suggests ɑj, oj, and ɛj.
//
// Also note: don't write "^:" as in the original paper, instead ":^", as
// a consequence of greedy matching.
const RAW_RULES: &[&str] = &[
    "[a] =ə",
    " [ar]o=əɹ",
    "[ar]#=ɛɹ",
    " ^[as]#=ɛɪs",
    "[a]wa=ə",
    "[aw]=ɔ",
    " :[any]=ɛni",
    "[a]^+#=ɛɪ",
    "#:[ally]=əli",
    "^[al]m=ɑ", // added
    " [al]#=əl",
    "[again]=əgɛn",
    "#:[ag]e=ɪdʒ",
    "[a]+%=ɛɪ",
    "[a]^+:#=æ",
    " :[a]^+ =ɛɪ",
    "[a]^%=ɛɪ",
    " [arr]=əɹ",
    "[arr]=æɹ",
    " :[ar] =ɑɹ",
    "[ar] =ɚ",
    "[ar]=ɑɹ",
    "[air]=ɛɹ",
    "[ai]=ɛɪ",
    "[ay]=ɛɪ",
    "[au]=ɔ",
    "#:[al]=əl",
    //"#:[als]=əlz", // subsumed by rule in s
    "[alk]=ɔk",
    "[al]^=ɔl",
    " :[able]=ɛɪbəl",
    "[able]=əbəl",
    "[ang]+=ɛɪndʒ",
    "[a]=æ",
    " [be]^#=bɪ",
    "[being]=biɪŋ",
    " [both]=boθ",
    " [bus]#=bɪz",
    "[buil]=bɪl",
    "[b]=b",
    "[crea]t=kɹiɛɪ1", // added
    " [ch]^=k",
    "^e[ch]=k",
    "[ch]=tʃ",
    " s[ci]#=saɪ",
    "[ci]a=ʃ",
    "[ci]o=ʃ",
    "[ci]en=ʃ",
    "[c]+=s",
    "[ck]=k",
    "[comfor]=kʌmfɚ", // added
    "[com]%=kʌm",
    "[c]=k",
    "#:[ded] =dɪd",
    ".e[d] =d",
    "#:^e[d] =t",
    " [de]^#=dɪ",
    " [does]=dʌz",
    " [doing]=duɪŋ",
    " [dow]=daw",
    "[du]a=dʒu",
    "[d]=d",
    "#:[e] ",
    "' :^[e] ",
    " :[e] =i",
    "#[ed] =d",
    "#:[e]d ",
    "[ev]er=ɛv",
    "g[e]ner=ɛ",       // added
    "^#:[ement]=mɛnt", // moved up and tweaked
    "[e]^%=i",
    "[eri]#=iɹi",
    "[eri]=ɛɹi",
    "#:[er]#=ɚ",
    "[er]#=ɛɹ",
    "[er]=ɚ",
    " [even]=ivɛn",
    "#:[e]w",
    "@[ew]=u",
    "[ew]=ju",
    "[e]o=i",
    "#:&[es] =ɪz",
    "g[es] =ɪz", // added and perhaps controversial
    "#:[e]s ",
    "#:[ely] =li",
    "[eful]=fʊl",
    "[ee]=i",
    "[earn]=ɚn",
    " [ear]^=ɚ",
    "[ead]=ɛd",
    "#:[ea]=iə",
    "[ea]su=ɛ",
    "[ea]lth=ɛ", // added
    "[ea]=i",
    "[eigh]=ɛɪ",
    "[ei]=i",
    " [eye]=aɪ",
    "[ey]=i",
    "[eu]=ju",
    "[e]=ɛ",
    "[ful]=fʊl",
    "[f]=f",
    "[giv]=gɪv",
    " [g]i^=g",
    "[ge]t=gɛ",
    "su[gges]=dʒɛs", // got rid of the g
    "[gg]=g",
    " b#[g]=g",
    "[g]+=dʒ",
    "[great]=gɹɛɪt",
    "#[gh]",
    "[g]=g",
    " [hav]=hæv",
    " [here]=hiɹ",
    " [hour]=awɚ",
    "[how]=haw",
    "[h]#=h",
    "[h]",
    " [in]=ɪn",
    "[in]d=aɪn",
    "#:[ier]=iɚ", // added: earlier, happier (not pier)
    "[ier]=iɹ",
    "#:r[ied] =id",
    "[ied] =aɪd",
    "fr[ien]=ɛn", // added
    "[ien]=iɛn",
    "[ie]t=aɪɛ",
    " :[i]%=aɪ",
    "[i]%=i",
    "[ie]=i",
    "[i]^+:#=ɪ",
    "[ir]#=aɪɹ",
    "[iz]%=aɪz",
    "[is]%=aɪz",
    "[i]d%=aɪ",
    "+^[i]^=ɪ",
    "[i]t%=aɪ",
    "#:^[i]^+=ɪ",
    "[i]^+=aɪ",
    "[ir]=ɚ",
    "[igh]=aɪ",
    "[ild]=aɪld",
    "[ign] =aɪn",
    "[ign]^=aɪn",
    "[ign]%=aɪn",
    "[ique]=ik",
    "[i]=ɪ",
    "[j]=dʒ",
    " [k]n",
    "[k]=k",
    "[lo]c#=lo",
    "l[l]",
    "#:^[l]%=əl",
    "[lead]=lid",
    "[l]=l",
    "[mov]=muv",
    "[m]=m",
    "e[ng]+=ndʒ",
    "[ng]r=ŋg",
    "[ng]#=ŋg",
    "[ngl]%=ŋgəl",
    "[ng]=ŋ",
    "[nk]=ŋk",
    "[n]=n",
    "[of] =əv",
    "[orough]=ɚo",
    "w[or]t=ɚ", // added
    "[or]t=ɔɹ", // added
    "[or]e=ɔɹ", // added
    "#:[or]=ɚ",
    "#:[ors]=ɚz",
    "[or]=ɔɹ",
    " [one]=wʌn",
    "[ow]=o",
    " [over]=ovɚ",
    "[ov]=ʌv",
    "[o]^%=o",
    "[o]^en=o",
    "[o]^i#=o",
    "[ol]d=ol",
    "[ought]=ɔt",
    "[ough]=əf",
    " [ou]=aw",
    "h[ou]s#=aw",
    "[ous]=əs",
    "[our]=ɔɹ",
    "[ould]=ʊd",
    "^[ou]^l=ʌ",
    "[oup]=up",
    "[ou]=aw",
    "[oy]=ɔɪ",
    "[oing]=oɪŋ",
    "[oi]=ɔɪ",
    "[oor]=ɔɹ",
    "[ook]=ʊk",
    "[ood]=ʊd",
    "[oo]=u",
    "[o]e=o",
    "[o] =o",
    "[oa]=o",
    " [only]=onli",
    " [once]=wʌns",
    "[on ' t]=ont",
    "c[o]n=ɑ",
    "[o]ng=ɔ",
    " :^[o]n=ʌ",
    "i[on]=ən",
    "#:[on] =ən",
    "#^[on]=ən",
    "[o]st=o",
    "[of]^=ɔf",
    "[other]=ʌðɚ",
    "[oss]=ɔs",
    "#:^[om]=ʌm",
    "[o]cus=o", // added
    "[o]=ɑ",
    "[phys]=fɪz", // added
    "[ph]=f",
    "[peop]=pip",
    "[pow]=paw",
    "[put]+=pjut", // added
    "[put]=pʊt",
    " [ps]=s", // added
    "[p]=p",
    "[quar]=kwɔɹ",
    "[qu]=kw",
    "[q]=k",
    "[re]^#=ɹi",
    "[rho]=ɹo", // added
    "[rh]=ɹ",   // added
    "[r]=ɹ",
    "[sh]=ʃ",
    "#[sion]=ʒən",
    "[some]=sʌm",
    "#[sur]#=ʒɚ",
    "[sur]#=ʃɚ",
    "#[su]#=ʒu",
    "#[ssu]#=ʃu",
    "#[sed] =zd",
    "#[s]#=z",
    "[said]=sɛd",
    "^[sion]=ʃən",
    "[s]s",
    ".[s] =z",
    "#:e[s] =z",
    "#^.##[s] =z",
    "#^.#[s] =s",
    "u[s] =s",
    " :#[s] =z",
    " [sch]=sk",
    "[s]c+",
    "#[sm]=zm",
    "#[sn] '=zən",
    "[s]=s",
    "[to] =tu",
    "[that] =ðæt",
    " [they]=ðɛɪ",
    " [there]=ðɛɹ",
    "[ther]=ðɚ",
    "[their]=ðɛɹ",
    "[these] =ðiz",
    " [then]=ðɛn",
    "[through]=θɹu",
    "[those]=ðoz",
    "[though] =ðo",
    " [thus]=ðʌs",
    "[th]=θ",
    "#:[ted] =tɪd",
    "s[ti]#n=tʃ",
    "[ti]o=ʃ",
    "[ti]a=ʃ",
    "[tien]=ʃən",
    "[tur]#=tʃɚ",
    "[tu]a=tʃu",
    " [two]=tu",
    "[t]=t",
    " [un]i=jun",
    " [un]=ʌn",
    " [upon]=əpɔn",
    "@[ur]#=ʊɹ",
    "[ur]#=jʊɹ",
    "[ur]=ɚ",
    "[u]^ =ʌ",
    "[u]^^=ʌ",
    "[uy]=aɪ",
    " g[u]#",
    "g[u]%",
    "g[u]#=w",
    "#n[u]=ju",
    "@[u]=u",
    "[u]=ju",
    "[view]=vju",
    " [viol]=vaɪl", // added
    " [volt]=volt", // added
    "[v]=v",
    " [were]=wɚ",
    " [wave]=wɛɪv", // added
    "[wa]s=wɑ",
    "[wa]t=wɑ",
    "[where]=wɛɹ",
    "[what]=wɑt",
    "[whol]=hol",
    "[who]=hu",
    "[wh]=ʍ",
    "[war]=wɔɹ",
    "[wor]^=wɚ",
    "[wr]=ɹ",
    "[w]=w",
    " [x]#=z",   // added
    " e[x]#=gz", // added
    "[x]=ks",
    "[young]=jʌŋ",
    " [you]=ju",
    " [yes]=jɛs",
    " [y]=j",
    "if[y] =aɪ", // added
    "#:^[y] =i",
    "#:^[y]i=i",
    " :[y] =aɪ",
    " :[y]#=aɪ",
    " :[y]^+:#=ɪ",
    " :[y]^#=aɪ",
    "[y]=ɪ",
    "[z]=z",
];

const DICTIONARY: &[(&str, &str)] = &[
    ("a", "ə"),
    ("allow", "əlaw1"),
    ("answer", "æ1nsɚ"),
    ("antenna", "æntɛ1nə"), // mostly stress
    ("anyone", "ɛ1niwʌ2n"),
    ("apply", "əplaɪ"),
    ("are", "ɑɹ"),
    ("argue", "ɑɹgju"),
    ("attempt", "ətɛmt"),
    ("aware", "əwɛ2ɚ"),
    ("bear", "bɛɚ"),
    ("better", "bɛ1tɚ"), // only here for stress
    ("british", "bɹɪ1tɪʃ"),
    ("certain", "sɚ1tən"),
    ("character", "kɛ1ɚəktɚ"),
    ("children", "tʃɪldɹɛn"),
    ("civil", "sɪvəl"),
    ("clothing", "kloðɪŋ"),
    ("colonial", "kəlo1nɪəl"),
    ("color", "kʌ1lɚ"),
    ("consider", "kʌnsɪ1dɚ"),
    ("control", "kʌntɹo1l"),
    ("contempt", "kʌntɛ1mt"),
    ("cost", "kɔst"), // maybe ɑ
    ("country", "kʌ1ntɹi"),
    ("cycle", "saɪkəl"),
    ("decent", "disənt"),
    ("death", "dɛθ"),
    ("design", "dəzaɪn"),
    ("different", "dɪ1fɹɛnt"),
    ("do", "du"),
    ("does", "dʌz"),
    ("done", "dʌn"),
    ("driven", "dɹɪvən"),
    ("elegant", "ɛ1ləgənt"),
    ("enter", "ɛntɚ"),
    ("enforce", "ɛnfɔɹs"),
    ("equal", "ikwəl"),
    ("evil", "i1vɪl"),
    ("father", "fɑ1ðɚ"),
    ("field", "fild"),
    ("fix", "fɪks"), // for suffix behavior
    ("fixes", "fɪksəz"),
    ("food", "fud"),
    ("foot", "fʊt"),
    ("fortunate", "fɔ1ɹtʃənət"),
    ("gauge", "gɛɪdʒ"),
    ("gone", "gɔn"),
    ("half", "hæf"),
    ("heard", "hɚd"),
    ("heart", "hɑɹt"),
    ("hey", "hɛɪ"),
    ("honor", "ʌ1nɚ"),
    ("hundred", "hʌndɹɛd"),
    ("i", "aɪ"),
    ("ideal", "aɪdi1l"),
    ("laugh", "læf"),
    ("listen", "lɪ1sɛn"),
    ("living", "lɪ1vɪŋ"),
    ("livelihood", "laɪ1vlihʊd"),
    ("loose", "lus"),
    ("lose", "luz"),
    ("machine", "mæʃi1n"),
    ("major", "mɛɪ1dʒɚ"),
    ("material", "məti1ɹi2əl"),
    ("mix", "mɪks"), // for suffix behavior
    ("money", "mʌ1ni"),
    ("muscle", "mʌsəl"),
    ("national", "næ1ʃənəl"),
    ("necessary", "nɛ1səsɛ2ɚi"),
    ("none", "nʌn"),
    ("now", "naw"),
    ("often", "ɔ1fɛn"),
    ("operate", "ɔ1pɚɛɪ2t"),
    ("piece", "pis"),
    ("pizza", "pitsə"),
    ("private", "pɹaɪ1vət"),
    ("president", "pɹɛ1zɪdɛ2nt"),
    ("promote", "pɹomo1t"), // stress
    ("pull", "pʊl"),
    ("push", "pʊʃ"),
    ("rational", "ɹæ1ʃənəl"),
    ("reality", "ɹiæ1lɪti"),
    ("relative", "ɹɛ1lətɪv"),
    ("reply", "ɹɛplaɪ"),
    ("senior", "si1njɚ"),
    ("slavery", "slɛɪvɚi"),
    ("significant", "sɪgnɪ1fɪkənt"),
    ("someone", "sʌ1mwʌn"),
    ("sometimes", "sʌ1mtaɪ1mz"),
    ("spirit", "spi1ɹɪt"),
    ("supply", "sʌplaɪ1"),
    ("surprise", "sʌpɹaɪ1z"),
    ("terrible", "tɛ1ɹɪbəl"),
    ("territory", "tɛ1ɹɪtɔɹ2i"),
    ("this", "ðɪs"),
    ("than", "ðæn"),
    ("the", "ðə"),
    ("them", "ðɛm"),
    ("threat", "θɹɛt"),
    ("today", "tədɛɪ"),
    ("together", "təgɛ1ðɚ"),
    ("total", "to1təl"),
    ("tough", "tʌf"),
    ("town", "tawn"),
    ("travel", "tɹævəl"),
    ("vague", "vɛɪg"),
    ("vein", "vɛɪn"),
    ("want", "wʌnt"),
    ("weapon", "wɛ1pən"),
    ("window", "wɪ1ndo"),
    ("woman", "wʊ1mən"),
    ("women", "wɪ1mən"),
];

impl TextToPhoneme {
    pub fn new() -> Self {
        let mut alpha_rules = [const { Vec::new() }; 26];
        for rule in RAW_RULES {
            let mut split1 = rule.split('[');
            let pre = split1.next().expect("missing open bracket");
            let mut split2 = split1.next().expect("missing open bracket").split(']');
            let body = split2.next().expect("missing close bracket");
            let mut split3 = split2.next().expect("missing close bracket").split('=');
            let post = split3.next().unwrap_or_default();
            let output = split3.next().unwrap_or_default();
            let first_ch = body.as_bytes()[0];
            assert!(first_ch.is_ascii_lowercase());
            if pre == " " && post == " " {
                println!("{rule}");
            }
            let rule = Rule {
                pre,
                body,
                post,
                output,
            };
            alpha_rules[(first_ch - b'a') as usize].push(rule);
        }
        let mut dict = HashMap::new();
        for (word, phonemes) in DICTIONARY {
            dict.insert(*word, *phonemes);
        }
        TextToPhoneme { alpha_rules, dict }
    }

    /// Translate text to phonemes.
    ///
    /// In this version, input should be normalized - lowercase, space separated.
    pub fn translate(&self, text: &str) -> String {
        let mut result = String::new();
        let mut ix = 1;
        while ix < text.len() {
            if let Some(pos) = text[ix + 1..].as_bytes().iter().position(|c| *c == b' ') {
                if !result.is_empty() {
                    result.push(' ');
                }
                let word_end = ix + 1 + pos;
                if let Some(phonemes) = self.dict.get(&text[ix..word_end]) {
                    result.push_str(phonemes);
                } else {
                    self.translate_word(text.as_bytes(), ix, &mut result);
                }
                ix = word_end + 1;
            } else {
                break;
            }
        }
        result
    }

    fn translate_word(&self, text: &[u8], mut ix: usize, result: &mut String) {
        while ix < text.len() {
            let c = text[ix];
            if !c.is_ascii_lowercase() {
                break;
            }
            let mut match_len = 0;
            for rule in &self.alpha_rules[(c - b'a') as usize] {
                if rule.matches(text, ix) {
                    //println!("matched {rule:?}");
                    result.push_str(rule.output);
                    match_len = rule.body.len();
                    break;
                }
            }
            if match_len == 0 {
                unreachable!("no rule matched");
            }
            ix += match_len;
        }
    }
}

impl Rule {
    fn matches(&self, text: &[u8], ix: usize) -> bool {
        //println!("trying {self:?}");
        let mut end_ix = ix + self.body.len();
        // Match body
        if end_ix > text.len() || &text[ix..end_ix] != self.body.as_bytes() {
            return false;
        }
        // Match prefix
        let mut start_ix = ix;
        let mut vowels_matched = 0;
        for pre in self.pre.as_bytes().iter().rev() {
            //println!("pre {pre}, start_ix={start_ix}");
            if start_ix == 0 {
                return *pre == b':';
            }
            if *pre != b'#' {
                vowels_matched = 0;
            }
            let mut prev = text[start_ix - 1];
            if pre.is_ascii_lowercase() || *pre == b' ' || *pre == b'\'' {
                if prev != *pre {
                    return false;
                }
                start_ix -= 1;
            } else if *pre == b'#' {
                // one or more vowels
                while Flags::VOWEL.is(prev) {
                    start_ix -= 1;
                    if start_ix == 0 {
                        break;
                    }
                    prev = text[start_ix - 1];
                    vowels_matched += 1;
                }
                if vowels_matched > 0 {
                    vowels_matched -= 1;
                } else {
                    return false;
                }
            } else if *pre == b':' {
                // zero or more consonants
                while Flags::VOWEL.is_not(prev) {
                    start_ix -= 1;
                    if start_ix == 0 {
                        break;
                    }
                    prev = text[start_ix - 1];
                }
            } else if *pre == b'^' {
                // one consonant
                if !Flags::VOWEL.is_not(prev) {
                    return false;
                }
                start_ix -= 1;
            } else if *pre == b'+' {
                // front_vowel
                if !Flags::FRONT_VOWEL.is(prev) {
                    return false;
                }
                start_ix -= 1;
            } else if *pre == b'.' {
                // a voiced consonant
                if !Flags::VOICED_CONSONANT.is(prev) {
                    return false;
                }
                start_ix -= 1;
            } else if *pre == b'&' {
                // sibilant
                if prev == b'h' {
                    if start_ix < 2 {
                        return false;
                    }
                    let first = text[start_ix - 2];
                    if !(first == b'c' || first == b's') {
                        return false;
                    }
                    start_ix -= 2;
                } else if !Flags::SIBILANT.is(prev) {
                    return false;
                }
                start_ix -= 1;
            } else if *pre == b'@' {
                // consonant influencing long u
                if prev == b'h' {
                    if start_ix < 2 {
                        return false;
                    }
                    let first = text[start_ix - 2];
                    if !(first == b'c' || first == b's' || first == b't') {
                        return false;
                    }
                    start_ix -= 2;
                } else if !Flags::LONG_U.is(prev) {
                    return false;
                }
                start_ix -= 1;
            } else {
                unreachable!("unknown prefix pattern character {pre}");
            }
        }
        // Match suffix
        for post in self.post.as_bytes().iter() {
            if end_ix == text.len() {
                return false;
            }
            let mut next = text[end_ix];
            if post.is_ascii_lowercase() || *post == b' ' || *post == b'\'' {
                if next != *post {
                    return false;
                }
                end_ix += 1;
            } else if *post == b'#' {
                // one or more vowels
                let mut matched = false;
                while Flags::VOWEL.is(next) || next == b'+' {
                    end_ix += 1;
                    if end_ix == text.len() {
                        break;
                    }
                    next = text[end_ix];
                    matched = true;
                }
                if !matched {
                    return false;
                }
            } else if *post == b':' {
                // zero or more consonants
                while Flags::VOWEL.is_not(next) {
                    end_ix += 1;
                    if end_ix == text.len() {
                        break;
                    }
                    next = text[end_ix];
                }
            } else if *post == b'^' {
                // one consonant
                if !Flags::VOWEL.is_not(next) {
                    return false;
                }
                end_ix += 1;
            } else if *post == b'+' {
                // front vowel
                if !(next == b'+' || Flags::FRONT_VOWEL.is(next)) {
                    return false;
                }
                end_ix += 1;
            } else if *post == b'%' {
                // suffix
                if next == b'i' {
                    if end_ix + 2 < text.len()
                        && text[end_ix + 1] == b'n'
                        && text[end_ix + 2] == b'g'
                    {
                        end_ix += 3;
                    } else {
                        return false;
                    }
                } else if next == b'e' {
                    fn oneof_d_r_s(c: u8) -> bool {
                        c == b'd' || c == b'r' || c == b's'
                    }
                    if end_ix + 1 < text.len() && oneof_d_r_s(text[end_ix + 1]) {
                        end_ix += 2;
                    } else if end_ix + 2 < text.len()
                        && text[end_ix + 1] == b'l'
                        && text[end_ix + 2] == b'y'
                    {
                        end_ix += 3;
                    } else {
                        end_ix += 1;
                    }
                } else if next == b'+' {
                    end_ix += 1;
                } else {
                    return false;
                }
            } else {
                unreachable!("unknown suffix pattern character {post}");
            }
        }
        true
    }
}

impl Flags {
    fn is(&self, letter: u8) -> bool {
        letter.is_ascii_lowercase() && ALPHA_FLAGS[(letter - b'a') as usize].contains(*self)
    }

    fn is_not(&self, letter: u8) -> bool {
        letter.is_ascii_lowercase() && !ALPHA_FLAGS[(letter - b'a') as usize].contains(*self)
    }
}

/// Words read as function words (`F:`) in the ASCII output: determiners,
/// pronouns, prepositions, conjunctions and auxiliaries. They get no
/// lexical stress.
const FUNCTION_WORDS: &[&str] = &[
    "a", "an", "the", "this", "that", "these", "those", "some", "any", "no", "every", "each",
    "i", "me", "my", "you", "your", "he", "him", "his", "she", "her", "it", "its", "we", "us",
    "our", "they", "them", "their", "who", "whom", "whose", "which", "what", "of", "in", "on",
    "at", "to", "for", "from", "by", "with", "as", "into", "onto", "upon", "about", "over",
    "under", "than", "and", "or", "but", "nor", "so", "if", "while", "when", "because", "is",
    "am", "are", "was", "were", "be", "been", "being", "do", "does", "did", "has", "have", "had",
    "will", "would", "shall", "should", "can", "could", "may", "might", "must", "not", "there",
];

/// An element of a word on its way from IPA to the ASCII alphabet.
#[derive(Clone, Copy, PartialEq)]
struct Seg {
    c: char,
    vowel: bool,
    stress: u8,
}

/// IPA (as produced by the rules above) to one-character DECtalk symbols.
fn ipa_to_segs(ipa: &str) -> Result<Vec<Seg>, String> {
    let chars: Vec<char> = ipa.chars().collect();
    let mut out: Vec<Seg> = vec![];
    let mut i = 0;
    while i < chars.len() {
        let pair = chars.get(i + 1).map(|&n| (chars[i], n));
        let two = match pair {
            Some(('ɛ' | 'e', 'ɪ')) => Some('e'),
            Some(('a', 'ɪ')) => Some('A'),
            Some(('a', 'w')) => Some('W'),
            Some(('ɔ', 'ɪ')) => Some('O'),
            Some(('t', 'ʃ')) => Some('C'),
            Some(('d', 'ʒ')) => Some('J'),
            _ => None,
        };
        let (c, len) = if let Some(c) = two {
            (c, 2)
        } else {
            let c = match chars[i] {
                'ə' => 'x',
                'ɚ' => 'R',
                'ɛ' => 'E',
                'æ' => '@',
                'ɑ' => 'a',
                'ɔ' => 'c',
                'o' => 'o',
                'u' | 'ʉ' => 'u',
                'ʊ' => 'U',
                'ʌ' => '^',
                'ɪ' => 'I',
                'i' => 'i',
                'ɹ' => 'r',
                'ŋ' => 'G',
                'ʃ' => 'S',
                'ʒ' => 'Z',
                'θ' => 'T',
                'ð' => 'D',
                'ʍ' => 'w',
                'j' => 'y',
                '1' | '2' => {
                    if let Some(v) = out.iter_mut().rev().find(|s| s.vowel) {
                        v.stress = if chars[i] == '1' { 1 } else { 2 };
                    }
                    i += 1;
                    continue;
                }
                'ː' => {
                    i += 1;
                    continue;
                }
                c @ ('b' | 'd' | 'f' | 'g' | 'h' | 'k' | 'l' | 'm' | 'n' | 'p' | 's' | 't'
                | 'v' | 'w' | 'z') => c,
                c => return Err(format!("no ASCII phoneme for '{c}' in {ipa}")),
            };
            (c, 1)
        };
        let vowel = "iIeE@aAW^coOUuxR".contains(c);
        // Collapse doubled consonants from the spelling rules ("little").
        if !vowel && out.last().is_some_and(|s| s.c == c) {
            i += len;
            continue;
        }
        out.push(Seg { c, vowel, stress: 0 });
        i += len;
    }
    Ok(out)
}

/// Suffixes that are neutral to stress, with their syllable counts. They
/// are stripped (repeatedly) before the stress-determining suffixes are
/// looked for.
const NEUTRAL_SUFFIXES: &[(&str, usize)] = &[
    ("ings", 1),
    ("ing", 1),
    ("ments", 1),
    ("ment", 1),
    ("ness", 1),
    ("fully", 2),
    ("ful", 1),
    ("less", 1),
    ("ly", 1),
    ("ers", 1),
    ("er", 1),
];

/// Where a suffix puts the stress, counting syllables of the stem from its
/// end (1 is the last syllable before the suffix).
#[derive(Clone, Copy)]
enum SuffixStress {
    /// The syllable just before the suffix (-tion, -ic, -ity).
    Before,
    /// Two syllables before the suffix's own syllable when the stem has at
    /// least three, else the first (-ate, -ize, -ous).
    Antepenult,
    /// The first syllable of the suffix (-ation, -ee, -eer, -ese).
    Final,
}

/// Stress-determining suffixes, with their syllable counts.
const STRESS_SUFFIXES: &[(&str, usize, SuffixStress)] = &[
    ("ations", 2, SuffixStress::Final),
    ("ation", 2, SuffixStress::Final),
    ("itions", 2, SuffixStress::Final),
    ("ition", 2, SuffixStress::Final),
    ("tions", 1, SuffixStress::Before),
    ("tion", 1, SuffixStress::Before),
    ("sions", 1, SuffixStress::Before),
    ("sion", 1, SuffixStress::Before),
    ("cian", 1, SuffixStress::Before),
    ("cial", 1, SuffixStress::Before),
    ("tial", 1, SuffixStress::Before),
    ("cious", 1, SuffixStress::Before),
    ("tious", 1, SuffixStress::Before),
    ("ically", 3, SuffixStress::Before),
    ("ical", 2, SuffixStress::Before),
    ("ics", 1, SuffixStress::Before),
    ("ic", 1, SuffixStress::Before),
    ("ities", 2, SuffixStress::Before),
    ("ity", 2, SuffixStress::Before),
    ("ety", 2, SuffixStress::Before),
    ("ify", 2, SuffixStress::Before),
    ("ified", 2, SuffixStress::Before),
    ("ating", 2, SuffixStress::Antepenult),
    ("ated", 2, SuffixStress::Antepenult),
    ("ates", 1, SuffixStress::Antepenult),
    ("ate", 1, SuffixStress::Antepenult),
    ("izing", 2, SuffixStress::Antepenult),
    ("ized", 1, SuffixStress::Antepenult),
    ("izes", 2, SuffixStress::Antepenult),
    ("ize", 1, SuffixStress::Antepenult),
    ("ous", 1, SuffixStress::Antepenult),
    ("ees", 1, SuffixStress::Final),
    ("ee", 1, SuffixStress::Final),
    ("eers", 1, SuffixStress::Final),
    ("eer", 1, SuffixStress::Final),
    ("ese", 1, SuffixStress::Final),
    ("ette", 1, SuffixStress::Final),
];

/// Unstressed prefixes (when the rules give them a reduced or lax vowel).
const UNSTRESSED_PREFIXES: &[&str] = &[
    "be", "de", "re", "pre", "con", "com", "ex", "en", "em", "a",
];

/// Put primary stress on a content word that has none from the
/// dictionary, using affix rules: stress-determining suffixes (after
/// stripping stress-neutral ones), otherwise the first syllable after an
/// unstressed prefix, otherwise the first full vowel.
fn assign_stress(word: &str, segs: &mut [Seg]) {
    let syl: Vec<usize> = (0..segs.len())
        .filter(|&k| segs[k].vowel || segs[k].c == 'L')
        .collect();
    let n = syl.len();
    if n == 0 {
        return;
    }
    let mut chosen = None;
    // Strip stress-neutral suffixes.
    let mut stem = word;
    let mut m = n;
    'strip: loop {
        for &(suf, k) in NEUTRAL_SUFFIXES {
            if stem.len() > suf.len() + 2 && stem.ends_with(suf) && m > k {
                stem = &stem[..stem.len() - suf.len()];
                m -= k;
                continue 'strip;
            }
        }
        break;
    }
    for &(suf, k, rule) in STRESS_SUFFIXES {
        if stem.len() > suf.len() && stem.ends_with(suf) && m > k {
            let before = m - k; // syllables before the suffix
            chosen = match rule {
                SuffixStress::Before => Some(before - 1),
                SuffixStress::Antepenult if before >= 2 => Some(before - 2),
                SuffixStress::Antepenult => Some(0),
                SuffixStress::Final => Some(before),
            };
            break;
        }
    }
    if chosen.is_none() && n >= 2 {
        let first = segs[syl[0]].c;
        let lax = matches!(first, 'x' | 'I' | 'i' | 'E');
        let prefix = UNSTRESSED_PREFIXES
            .iter()
            .find(|p| word.len() > p.len() + 2 && word.starts_with(**p));
        let unstressed = match prefix {
            // "a-" only when the rules already reduced it (about, along).
            Some(&"a") => first == 'x',
            // con-, com- in longer words (consider, computer, but common).
            Some(&("con" | "com")) => lax || n >= 3,
            Some(_) => lax,
            None => false,
        };
        if unstressed {
            chosen = Some(1);
        }
    }
    let k = match chosen {
        Some(c) => syl[c.min(n - 1)],
        None => segs
            .iter()
            .position(|s| s.vowel && s.c != 'x')
            .unwrap_or(syl[0]),
    };
    // Never stress syllabic L; move to a neighboring vowel.
    let k = if segs[k].c == 'L' {
        segs[..k].iter().rposition(|s| s.vowel).unwrap_or(k)
    } else {
        k
    };
    segs[k].stress = 1;
}

/// Prefixes stripped to find a dictionary stem, with their IPA. They are
/// left unstressed; the stem carries the primary stress.
const PREFIXES: &[(&str, &str)] = &[
    ("un", "ʌn"),
    ("non", "nɑn"),
    ("dis", "dɪs"),
    ("mis", "mɪs"),
    ("re", "ɹi"),
    ("pre", "pɹi"),
    ("in", "ɪn"),
    ("im", "ɪm"),
    ("over", "ovɚ"),
    ("under", "ʌndɚ"),
];

/// Suffixes stripped to find a dictionary stem. `None` means the IPA
/// depends on the stem's last sound (-s, -es, -ed).
const SUFFIXES: &[(&str, Option<&str>)] = &[
    ("ly", Some("li")),
    ("ness", Some("nəs")),
    ("ments", Some("mənts")),
    ("ment", Some("mənt")),
    ("ful", Some("fəl")),
    ("less", Some("ləs")),
    ("ers", Some("ɚz")),
    ("er", Some("ɚ")),
    ("est", Some("əst")),
    ("ings", Some("ɪŋz")),
    ("ing", Some("ɪŋ")),
    ("able", Some("əbəl")),
    ("ism", Some("ɪzəm")),
    ("ist", Some("ɪst")),
    ("ize", Some("aɪz")),
    ("ship", Some("ʃɪp")),
    ("hood", Some("hʊd")),
    ("ed", None),
    ("es", None),
    ("s", None),
];

/// IPA of -s/-es or -ed after a stem whose pronunciation is `stem`.
fn inflection_ipa(stem: &str, suffix: &str) -> String {
    let last = stem.chars().rev().find(|c| !c.is_ascii_digit());
    let voiceless = matches!(last, Some('p' | 't' | 'k' | 'f' | 'θ' | 's' | 'ʃ'));
    if suffix == "ed" {
        match last {
            Some('t' | 'd') => "ɪd",
            _ if voiceless => "t",
            _ => "d",
        }
    } else {
        match last {
            Some('s' | 'z' | 'ʃ' | 'ʒ') => "ɪz",
            _ if voiceless => "s",
            _ => "z",
        }
    }
    .to_string()
}

impl TextToPhoneme {
    /// Dictionary lookup, stripping affixes to find a stem.
    fn lookup(&self, word: &str) -> Option<String> {
        if let Some(p) = self.dict.get(word) {
            return Some(p.to_string());
        }
        for &(pre, ipa) in PREFIXES {
            if let Some(rest) = word.strip_prefix(pre)
                && rest.len() >= 3
                && let Some(x) = self.lookup(rest)
            {
                return Some(format!("{ipa}{x}"));
            }
        }
        for &(suf, ipa) in SUFFIXES {
            let Some(stem) = word.strip_suffix(suf) else { continue };
            // Spelled -es only after a sibilant or o (fixes, goes), not
            // honoress.
            if suf == "es"
                && !["s", "x", "z", "sh", "ch", "o"].iter().any(|e| stem.ends_with(e))
            {
                continue;
            }
            if stem.len() < 2 {
                continue;
            }
            // Spelling changes: dropped e, y to i, doubled consonant.
            let mut cands = vec![stem.to_string(), format!("{stem}e")];
            if let Some(s) = stem.strip_suffix('i') {
                cands.push(format!("{s}y"));
            }
            let b = stem.as_bytes();
            if b.len() >= 2 && b[b.len() - 1] == b[b.len() - 2] {
                cands.push(stem[..stem.len() - 1].to_string());
            }
            for c in cands {
                if let Some(x) = self.lookup(&c) {
                    let ipa = ipa.map_or_else(|| inflection_ipa(&x, suf), String::from);
                    return Some(format!("{x}{ipa}"));
                }
            }
        }
        None
    }

    /// Pronunciation of a word in IPA: the dictionary (with affixes
    /// stripped), else the letter-to-sound rules. For -ed and -ing, the
    /// rules are applied to the stem, so that rules for a final e (as in
    /// "fix" + "ed") don't see the suffix.
    fn pronounce(&self, word: &str) -> String {
        if let Some(p) = self.lookup(word) {
            return p;
        }
        // n't contractions (apostrophe already dropped): isnt, didnt,
        // couldnt. The n is syllabic, and the stem keeps its voicing.
        if let Some(stem) = word.strip_suffix("nt")
            && (stem.ends_with('s') || stem.ends_with('d'))
            && stem.len() >= 2
        {
            let x = self.pronounce(stem);
            let x = x.strip_suffix('s').map_or(x.clone(), |s| format!("{s}z"));
            return format!("{x}ənt");
        }
        let has_vowel = |s: &str| s.bytes().any(|c| b"aeiouy".contains(&c));
        for suf in ["ed", "ing"] {
            if let Some(stem) = word.strip_suffix(suf)
                && stem.len() >= 3
                && has_vowel(stem)
                && !stem.ends_with('e')
            {
                let b = stem.as_bytes();
                // We pass a generic front vowel context in place of the suffix, so "change" instead of "chang".
                let mut context = "+";
                let stem = if b[b.len() - 1] == b[b.len() - 2] && !stem.ends_with("ll") && !stem.ends_with("ss") {
                    context = "";
                    &stem[..stem.len() - 1]
                } else {
                    stem
                };
                eprintln!("translating {stem}{context}");
                let x = self.translate(&format!(" {stem}{context} "));
                let ipa = if suf == "ed" { inflection_ipa(&x, suf) } else { "ɪŋ".into() };
                return format!("{x}{ipa}");
            }
        }
        self.translate(&format!(" {word} "))
    }

    /// Translate text to the ASCII phoneme format read by [`crate::ascii`],
    /// with word markers, stress and phrase boundaries.
    ///
    /// Stress comes from the dictionary when it has it; otherwise a content
    /// word is stressed on its first full (non-schwa) vowel. Function words
    /// come from a fixed list. Commas, colons and semicolons become `,`;
    /// question marks `?`; other sentence punctuation `.`.
    pub fn to_ascii(&self, text: &str) -> Result<String, String> {
        // Tokenize into words and punctuation.
        let mut tokens: Vec<String> = vec![];
        let mut word = String::new();
        for ch in text.chars().flat_map(char::to_lowercase) {
            if ch.is_ascii_lowercase() {
                word.push(ch);
                continue;
            }
            if ch == '\'' || ch == '’' {
                continue;
            }
            if !word.is_empty() {
                tokens.push(std::mem::take(&mut word));
            }
            match ch {
                ',' | ';' | ':' => tokens.push(",".into()),
                '.' | '!' => tokens.push(".".into()),
                '?' => tokens.push("?".into()),
                _ => {}
            }
        }
        if !word.is_empty() {
            tokens.push(word);
        }
        if !tokens
            .last()
            .is_some_and(|t| matches!(t.as_str(), "." | "?"))
        {
            tokens.push(".".into());
        }

        // Words to segments.
        struct Word {
            function: bool,
            segs: Vec<Seg>,
        }
        enum Tok {
            Word(Word),
            Punct(String),
        }
        let mut toks = vec![];
        for t in tokens {
            if !t.as_bytes()[0].is_ascii_lowercase() {
                toks.push(Tok::Punct(t));
                continue;
            }
            let ipa = self.pronounce(&t);
            let mut segs = ipa_to_segs(&ipa).map_err(|e| format!("{e} (\"{t}\")"))?;
            let function = FUNCTION_WORDS.contains(&t.as_str());
            if function {
                segs.iter_mut().for_each(|s| s.stress = 0);
            } else if !segs.iter().any(|s| s.stress == 1) {
                assign_stress(&t, &mut segs);
            }
            toks.push(Tok::Word(Word { function, segs }));
        }

        // Allophones that depend on context.
        let n = toks.len();
        for k in 0..n {
            // The first segment of the next word, if the next token is a word.
            let next_first = match toks.get(k + 1) {
                Some(Tok::Word(w)) => w.segs.first().copied(),
                _ => None,
            };
            let Tok::Word(w) = &mut toks[k] else { continue };
            let segs = &mut w.segs;
            let mut j = 0;
            while j < segs.len() {
                let prev = j.checked_sub(1).map(|p| segs[p]);
                let next = segs.get(j + 1).copied().or(next_first);
                let in_word_next = segs.get(j + 1).copied();
                match segs[j].c {
                    // Post-vocalic r is r-coloring (merged with the vowel or
                    // RX by the allophone pass in ascii.rs).
                    'r' if prev.is_some_and(|p| p.vowel && p.c != 'R')
                        && !in_word_next.is_some_and(|n| n.vowel) =>
                    {
                        segs[j].c = 'R';
                        segs[j].vowel = false;
                    }
                    // Schwa + n after an alveolar obstruent, before a
                    // consonant or word end, is syllabic N (button, isn't).
                    'x' if prev.is_some_and(|p| "tdsz".contains(p.c))
                        && j >= 2
                        && (segs[j - 2].vowel || segs[j - 2].c == 'R')
                        && in_word_next.is_some_and(|n| n.c == 'n')
                        && !segs[j + 2..].iter().any(|n| n.vowel) =>
                    {
                        segs[j] = Seg {
                            c: 'N',
                            vowel: true,
                            stress: 0,
                        };
                        segs.remove(j + 1);
                    }
                    // Schwa + l before a consonant or word end is syllabic L.
                    'x' if in_word_next.is_some_and(|n| n.c == 'l')
                        && !segs.get(j + 2).is_some_and(|n| n.vowel) =>
                    {
                        segs[j] = Seg {
                            c: 'L',
                            vowel: true,
                            stress: 0,
                        };
                        segs.remove(j + 1);
                    }
                    // Flap: t or d between a vowel and an unstressed vowel
                    // (or syllabic L), also across a word boundary.
                    't' | 'd'
                        if prev.is_some_and(|p| p.vowel || p.c == 'R')
                            && next.is_some_and(|n| (n.vowel || n.c == 'L') && n.stress == 0)
                            // Not before syllabic n (didn't, button).
                            && !(in_word_next.is_some_and(|n| n.c == 'x')
                                && segs.get(j + 2).is_some_and(|n| n.c == 'n')
                                && !segs[j + 3..].iter().any(|n| n.vowel)) =>
                    {
                        segs[j].c = '&';
                    }
                    _ => {}
                }
                j += 1;
            }
        }

        // Emit.
        let mut out = vec![];
        for t in toks {
            match t {
                Tok::Punct(p) => out.push(p),
                Tok::Word(w) => {
                    out.push(if w.function { "F:" } else { "C:" }.into());
                    let mut s = String::new();
                    let mut prev_vowel = false;
                    for seg in w.segs {
                        // Syllabic R after a vowel (earlier, happier) gets a
                        // syllable break so it isn't read as r-coloring.
                        if seg.c == 'R' && seg.vowel && prev_vowel {
                            s.push('-');
                        }
                        prev_vowel = seg.vowel;
                        match seg.stress {
                            1 => s.push('\''),
                            2 => s.push('"'),
                            _ => {}
                        }
                        s.push(seg.c);
                    }
                    out.push(s);
                }
            }
        }
        Ok(out.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_man() {
        let ttp = TextToPhoneme::new();
        assert_eq!(
            ttp.to_ascii("The old man sat in a rocker.").unwrap(),
            "F: Dx C: 'old C: m'@n C: s'@& F: In F: x C: r'akR ."
        );
    }

    #[test]
    fn affixes() {
        let mut ttp = TextToPhoneme::new();
        ttp.dict.insert("fortunate", "fɔ1ɹtʃənət");
        ttp.dict.insert("material", "mətɪ1ɹiəl");
        assert_eq!(
            ttp.to_ascii("materialism materialists materialized").unwrap(),
            "C: mxt'IrixlIzxm C: mxt'IrixlIsts C: mxt'IrixlAzd ."
        );
        assert_eq!(
            ttp.to_ascii("Unfortunately, fortunates.").unwrap(),
            "C: ^nf'cRCxnxtli , C: f'cRCxnxts ."
        );
        assert_eq!(ttp.to_ascii("fixed").unwrap(), "C: f'Ikst .");
    }

    /// Every rule output and dictionary entry must map to ASCII phonemes.
    #[test]
    fn all_outputs_map() {
        let mut bad = vec![];
        for rule in RAW_RULES {
            let out = rule.rsplit_once('=').map_or("", |r| r.1);
            if let Err(e) = ipa_to_segs(out) {
                bad.push(format!("{rule}: {e}"));
            }
        }
        for (w, p) in DICTIONARY {
            if let Err(e) = ipa_to_segs(p) {
                bad.push(format!("{w}: {e}"));
            }
        }
        assert!(bad.is_empty(), "{bad:#?}");
    }
}
