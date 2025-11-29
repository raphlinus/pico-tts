//! A simple, low-resource text to phoneme implementation

use bitflags::bitflags;

pub struct TextToPhoneme {
    alpha_rules: [Vec<Rule>; 26],
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
    " [are] =ɑɹ", // note: should probably go into dict instead
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
    " [do] =du",
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
    " [i] =aɪ",
    "[in]d=aɪn",
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
    " [now] =naw",
    "[n]=n",
    "[of] =əv",
    "[orough]=ɚo",
    "w[or]t=ɚ", // added
    "[or]t=ɔɹ", // added
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
    "[ph]=f",
    "[peop]=pip",
    "[pow]=paw",
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
    " [the] =ðə",
    "[to] =tu",
    "[that] =ðæt",
    " [this] =ðɪs",
    " [they]=ðeɪ",
    " [there]=ðɛɹ",
    "[ther]=ðɚ",
    "[their]=ðɛɹ",
    " [than] =ðæn",
    " [them] =ðɛm",
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
    "[v]=v",
    " [were]=wɚ",
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
            let rule = Rule {
                pre,
                body,
                post,
                output,
            };
            alpha_rules[(first_ch - b'a') as usize].push(rule);
        }
        TextToPhoneme { alpha_rules }
    }

    /// Translate text to phonemes.
    ///
    /// In this version, input should be normalized - lowercase, space separated.
    pub fn translate(&self, text: &str) -> String {
        let mut result = String::new();
        // TODO: iterate words etc
        self.translate_word(text.as_bytes(), 1, &mut result);
        result
    }

    fn translate_word(&self, text: &[u8], mut ix: usize, result: &mut String) {
        while ix < text.len() {
            let c = text[ix];
            if c == b' ' {
                ix += 1;
                if ix + 1 < text.len() {
                    result.push(' ');
                }
                continue;
            }
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
                while Flags::VOWEL.is(next) {
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
                if !Flags::FRONT_VOWEL.is(next) {
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
    fn from_letter(letter: u8) -> Self {
        assert!(letter.is_ascii_lowercase());
        ALPHA_FLAGS[(letter - b'a') as usize]
    }

    fn is(&self, letter: u8) -> bool {
        letter.is_ascii_lowercase() && ALPHA_FLAGS[(letter - b'a') as usize].contains(*self)
    }

    fn is_not(&self, letter: u8) -> bool {
        letter.is_ascii_lowercase() && !ALPHA_FLAGS[(letter - b'a') as usize].contains(*self)
    }
}
