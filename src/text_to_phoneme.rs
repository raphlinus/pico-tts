//! A simple, low-resource text to phoneme implementation

use std::collections::HashMap;

use bitflags::bitflags;

pub struct TextToPhoneme {
    alpha_rules: [Vec<Rule>; 26],
    dict: HashMap<&'static str, &'static str>,
    /// Print each dictionary hit, affix split and rule firing to stderr.
    pub trace: bool,
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
// The output is in the one-character ASCII phoneme alphabet of
// [`crate::ascii`] (DECtalk symbols), with `'` and `"` before a vowel for
// primary and secondary stress. The dictionary uses the same notation.
//
// Also note: don't write "^:" as in the original paper, instead ":^", as
// a consequence of greedy matching.
const RAW_RULES: &[&str] = &[
    "[a] =x",
    " [ar]o=xr",
    "[ar]#=Er",
    " ^[as]#=es",
    " [a]^o=x", // added: about, above, around, another
    " [a]^ai=x", // added: again, available
    "[a]wa=x",
    "[aw]=c",
    " :[any]=Eni",
    "[a]^+#=e",
    "#:[ally]=xli",
    "^[al]m=a", // added
    " [al]#=xl",
    "[again]=xgEn",
    "#:[ag]e=IJ",
    "[a]+%=e",
    "[a]^+:#=@",
    " :[a]^+ =e",
    "[a]^%=e",
    " [arr]=xr",
    "[arr]=@r",
    " :[ar] =ar",
    "[ar] =R",
    "[ar]=ar",
    "[air]=Er",
    "[ai]=e",
    "[ay]=e",
    "[au]=c",
    "#:[al]=xl",
    //"#:[als]=əlz", // subsumed by rule in s
    "[alk]=ck",
    "[al]^=cl",
    " :[able]=ebxl",
    "[able]=xbxl",
    "[ang]+=enJ",
    "[a]=@",
    " [be]^#=bI",
    "[being]=biIG",
    " [both]=boT",
    " [bus]#=bIz",
    "[buil]=bIl",
    "[b]=b",
    "[crea]t=kri'e", // added
    " [ch]^=k",
    "^e[ch]=k",
    "[ch]=C",
    " s[ci]#=sA",
    "[ci]a=S",
    "[ci]o=S",
    "[ci]en=S",
    "[c]+=s",
    "[ck]=k",
    "[comfor]=k^mfR", // added
    "[com]%=k^m",
    "[c]=k",
    "#:[ded] =dId",
    ".e[d] =d",
    "#:^e[d] =t",
    " [de]^#=dI",
    " [does]=d^z",
    " [doing]=duIG",
    " [dow]=dW",
    "[du]a=Ju",
    "[d]=d",
    "#:[e] ",
    "' :^[e] ",
    " :[e] =i",
    "#[ed] =d",
    "#:[e]d ",
    "[ev]er=Ev",
    "g[e]ner=E",       // added
    "^#:[ement]=mEnt", // moved up and tweaked
    "[e]^%=i",
    "[eri]#=iri",
    "[eri]=Eri",
    "#:[er]#=R",
    "[er]#=Er",
    "[er]=R",
    " [even]=ivEn",
    "#:[e]w",
    "@[ew]=u",
    "[ew]=yu",
    "[e]o=i",
    "#:&[es] =Iz",
    "g[es] =Iz", // added and perhaps controversial
    "#:[e]s ",
    "#:[ely] =li",
    "[eful]=fUl",
    "[ee]=i",
    "[earn]=Rn",
    "[ear]ch=R", // added: search, research
    " [ear]l=R", // added: early, earl (not clearly)
    "[ear]th=R", // added: earth
    " [ear]^=R",
    "[ead]=Ed",
    "#:[ea]=ix",
    "[ea]su=E",
    "[ea]lth=E", // added
    "[ea]=i",
    "[eigh]=e",
    "[ei]=i",
    " [eye]=A",
    "[ey]=i",
    "[eu]=yu",
    "[e]=E",
    "[ful]=fUl",
    "[f]=f",
    "[giv]=gIv",
    " [g]i^=g",
    "[ge]t=gE",
    "su[gges]=JEs", // got rid of the g
    "[gg]=g",
    " b#[g]=g",
    "[g]+=J",
    "[great]=gret",
    "#[gh]",
    "[g]=g",
    " [hav]=h@v",
    " [here]=hir",
    " [hour]=WR",
    "[how]=hW",
    "[h]#=h",
    "[h]",
    " [in]=In",
    "[in]d=An",
    "#:[ier]=iR", // added: earlier, happier (not pier)
    "[i]ty =I", // added: city, -ity
    "[ier]=ir",
    "#:r[ied] =id",
    "[ied] =Ad",
    "fr[ien]=En", // added
    "[ien]=iEn",
    "[ie]t=AE",
    " :[i]%=A",
    "[i]%=i",
    "[ie]=i",
    "[i]^+:#=I",
    "[ir]#=Ar",
    "[iz]%=Az",
    "[is]%=Az",
    "[i]d%=A",
    "+^[i]^=I",
    "[i]t%=A",
    "#:^[i]^+=I",
    "[i]^+=A",
    "[ir]=R",
    "[igh]=A",
    "[ild]=Ald",
    "[ign] =An",
    "[ign]^=An",
    "[ign]%=An",
    "[ique]=ik",
    "[i]=I",
    "[j]=J",
    " [k]n",
    "[k]=k",
    "[lo]c#=lo",
    "l[l]",
    "#:^[l]%=xl",
    "[lead]=lid",
    "[l]=l",
    "[mov]=muv",
    "[m]=m",
    "e[ng]+=nJ",
    "[ng]e =nJ", // added: fringe, hinge, sponge
    "[ng]es =nJ", // added: fringes
    "[ng]r=Gg",
    "[ng]#=Gg",
    "[ngl]%=Ggxl",
    "[ng]=G",
    "[nk]=Gk",
    "[n]=n",
    "[of] =xv",
    "[orough]=Ro",
    "w[or]t=R", // added
    "[or]t=cr", // added
    "[or]e=cr", // added
    "#:[or]=R",
    "#:[ors]=Rz",
    "[or]=cr",
    " [one]=w^n",
    "[ow]=o",
    " [over]=ovR",
    "[ov]=^v",
    "[o]^%=o",
    "[o]^en=o",
    "[o]^i#=o",
    "[ol]d=ol",
    "[ought]=ct",
    "[ough]=xf",
    " [ou]=W",
    "[ouse] =Ws", // added: house, mouse, blouse (not famous)
    "h[ou]s#=W",
    "[ous]=xs",
    "[our]=cr",
    "[ould]=Ud",
    "^[ou]^l=^",
    "[oup]=up",
    "[ou]=W",
    "[oy]=O",
    "[oing]=oIG",
    "[oi]=O",
    "[oor]=cr",
    "[ook]=Uk",
    "[ood]=Ud",
    "[oo]=u",
    "[o]e=o",
    "[o] =o",
    "[oa]=o",
    " [only]=onli",
    " [once]=w^ns",
    "[on ' t]=ont",
    "c[o]n=a",
    "[o]ng=c",
    " :^[o]n=^",
    "i[on]=xn",
    "#:[on] =xn",
    "#^[on]=xn",
    "[o]st=o",
    "[of]^=cf",
    "[other]=^DR",
    "[oss]=cs",
    "#:^[om]=^m",
    "[o]cus=o", // added
    "[o]=a",
    "[phys]=fIz", // added
    "[photo]=foto",
    "[ph]=f",
    "[peop]=pip",
    "[pow]=pW",
    "[put]+=pyut", // added
    "[put]=pUt",
    " [ps]=s", // added
    "[p]=p",
    "[quar]=kwcr",
    "[qu]=kw",
    "[q]=k",
    "[re]^#=ri",
    "[rho]=ro", // added
    "[rh]=r",   // added
    "[r]=r",
    "[sh]=S",
    "#[sion]=Zxn",
    "[some]=s^m",
    "#[sur]#=ZR",
    "[sur]#=SR",
    "#[su]#=Zu",
    "#[ssu]#=Su",
    "#[sed] =zd",
    "#[s]#=z",
    "[said]=sEd",
    "^[sion]=Sxn",
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
    "#[sn] '=zxn",
    "[s]=s",
    "[to] =tu",
    "[that] =D@t",
    " [they]=De",
    " [there]=DEr",
    "[ther]=DR",
    "[their]=DEr",
    "[these] =Diz",
    " [then]=DEn",
    "[through]=Tru",
    "[those]=Doz",
    "[though] =Do",
    " [thus]=D^s",
    "[th]=T",
    "#:[ted] =tId",
    "s[ti]#n=C",
    "[ti]o=S",
    "[ti]a=S",
    "[tien]=Sxn",
    "[tur]#=CR",
    "[tu]a=Cu",
    " [two]=tu",
    "[t]=t",
    " [un]i=yun",
    " [un]=^n",
    " [upon]=xpcn",
    "@[ur]#=Ur",
    "[ur]#=yUr",
    "[ur]=R",
    "[u]^ =^",
    "[u]^^=^",
    "[uy]=A",
    " g[u]#",
    "g[u]%",
    "g[u]#=w",
    "#n[u]=yu",
    "@[u]=u",
    "[u]=yu",
    "[view]=vyu",
    " [viol]=vAl", // added
    " [volt]=volt", // added
    "[v]=v",
    " [were]=wR",
    " [wave]=wev", // added
    "[wa]s=wa",
    "[wa]t=wa",
    "[where]=wEr",
    "[what]=wat",
    "[whol]=hol",
    "[who]=hu",
    "[wh]=w",
    "[war]=wcr",
    "[wor]^=wR",
    "[wr]=r",
    "[w]=w",
    " [x]#=z",   // added
    " e[x]#=gz", // added
    "[x]=ks",
    "[young]=y^G",
    " [you]=yu",
    " [yes]=yEs",
    " [y]=y",
    "if[y] =A", // added
    "#:^[y] =i",
    "#:^[y]i=i",
    " :[y] =A",
    " :[y]#=A",
    " :[y]^+:#=I",
    " :[y]^#=A",
    "#[y]#=y", // added: player, beyond (consonant between vowels)
    "aw[y]e=y", // added: lawyer, sawyer
    "[y]=I",
    "[z]=z",
];

const DICTIONARY: &[(&str, &str)] = &[
    ("a", "x"),
    ("across", "xkrcs"),
    ("allow", "xl'W"),
    ("answer", "'@nsR"),
    ("antenna", "@nt'Enx"), // mostly stress
    ("anyone", "'Eniw\"^n"),
    ("apply", "xplA"),
    ("are", "ar"),
    ("argue", "argyu"),
    ("attempt", "xtEmt"),
    ("aware", "xw\"ER"),
    ("bear", "bER"),
    ("better", "b'EtR"), // only here for stress
    ("british", "br'ItIS"),
    ("category", "k'@txgcri"),
    ("certain", "s'Rtxn"),
    ("character", "k'ERxktR"),
    ("chief", "Cif"),
    ("children", "CIldrEn"),
    ("civil", "sIvxl"),
    ("clothing", "kloDIG"),
    ("colonial", "kxl'onIxl"),
    ("color", "k'^lR"),
    ("company", "k'^mpxni"),
    ("consider", "k^ns'IdR"),
    ("control", "k^ntr'ol"),
    ("contempt", "k^nt'Emt"),
    ("cost", "kcst"), // maybe ɑ
    ("country", "k'^ntri"),
    ("cycle", "sAkxl"),
    ("decent", "disxnt"),
    ("death", "dET"),
    ("design", "dxzAn"),
    ("different", "d'IfrEnt"),
    ("do", "du"),
    ("does", "d^z"),
    ("done", "d^n"),
    ("driven", "drIvxn"),
    ("elegant", "'Elxgxnt"),
    ("enter", "EntR"),
    ("enforce", "Enfcrs"),
    ("equal", "ikwxl"),
    ("evil", "'ivIl"),
    ("father", "f'aDR"),
    ("field", "fild"),
    ("fix", "fIks"), // for suffix behavior
    ("fixes", "fIksxz"),
    ("food", "fud"),
    ("foot", "fUt"),
    ("fortunate", "f'crCxnxt"),
    ("gauge", "geJ"),
    ("gone", "gcn"),
    ("grief", "grif"),
    ("half", "h@f"),
    ("heard", "hRd"),
    ("heart", "hart"),
    ("hey", "he"),
    ("honest", "cnEst"),
    ("honor", "'^nR"),
    ("horizon", "hcr'Azxn"),
    ("hostile", "h'astxl"),
    ("i", "A"),
    ("ideal", "Ad'il"),
    ("incline", "Inkl'An"),
    ("laugh", "l@f"),
    ("level", "lEvxl"),
    ("listen", "l'IsEn"),
    ("living", "l'IvIG"),
    ("livelihood", "l'AvlihUd"),
    ("loose", "lus"),
    ("lose", "luz"),
    ("machine", "m@S'in"),
    ("major", "m'eJR"),
    ("material", "mxt'ir\"ixl"),
    ("meant", "mEnt"),
    ("mix", "mIks"), // for suffix behavior
    ("money", "m'^ni"),
    ("muscle", "m^sxl"),
    ("national", "n'@Sxnxl"),
    ("necessary", "n'Esxs\"ERi"),
    ("none", "n^n"),
    ("now", "nW"),
    ("often", "'cfEn"),
    ("online", "anlAn"),
    ("operate", "'cpR\"et"),
    ("panel", "p@nxl"),
    ("piece", "pis"),
    ("pizza", "pitsx"),
    ("poll", "pol"),
    ("primetime", "prAmtAm"),
    ("private", "pr'Avxt"),
    ("president", "pr'EzId\"Ent"),
    ("privy", "pr'Ivi"),
    ("profile", "profAl"),
    ("program", "progr@m"),
    ("promote", "prom'ot"), // stress
    ("pull", "pUl"),
    ("push", "pUS"),
    ("quality", "kw'alIti"),
    ("rational", "r'@Sxnxl"),
    ("reality", "ri'@lIti"),
    ("reference", "rEfrEns"),
    ("relative", "r'ElxtIv"),
    ("reply", "rEplA"),
    ("revive", "rxvAv"),
    ("senior", "s'inyR"),
    ("slavery", "slevRi"),
    ("significant", "sIgn'IfIkxnt"),
    ("software", "scftwEr"),
    ("someone", "s'^mw^n"),
    ("sometimes", "s'^mt'Amz"),
    ("spirit", "sp'irIt"),
    ("supply", "s^pl'A"),
    ("surprise", "s^pr'Az"),
    ("survive", "sRv'Av"),
    ("tax", "t@ks"),
    ("terrible", "t'ErIbxl"),
    ("territory", "t'ErIt\"cri"),
    ("this", "DIs"),
    ("than", "D@n"),
    ("the", "Dx"),
    ("them", "DEm"),
    ("threat", "TrEt"),
    ("today", "txde"),
    ("together", "txg'EDR"),
    ("total", "t'otxl"),
    ("tough", "t^f"),
    ("town", "tWn"),
    ("travel", "tr@vxl"),
    ("vague", "veg"),
    ("veteran", "v'Etxrxn"),
    ("vein", "ven"),
    ("want", "w^nt"),
    ("weapon", "w'Epxn"),
    ("widespread", "wAdsprEd"),
    ("window", "w'Indo"),
    ("woman", "w'Umxn"),
    ("women", "w'Imxn"),
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
        for (word, phonemes) in crate::normalize::WORDS {
            dict.entry(*word).or_insert(*phonemes);
        }
        TextToPhoneme {
            alpha_rules,
            dict,
            trace: false,
        }
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
                    if self.trace {
                        eprintln!("  {:<16} dictionary {phonemes}", &text[ix..word_end]);
                    }
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
                    if self.trace {
                        let word = String::from_utf8_lossy(text).trim().to_string();
                        eprintln!(
                            "  {word:<16} {:>2}  \"{}[{}]{}={}\"",
                            ix - 1,
                            rule.pre,
                            rule.body,
                            rule.post,
                            rule.output
                        );
                    }
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

/// An element of a word: one ASCII phoneme with its stress.
#[derive(Clone, Copy, PartialEq)]
struct Seg {
    c: char,
    vowel: bool,
    stress: u8,
}

/// Syllabic phonemes of the ASCII alphabet.
const VOWELS: &str = "iIeE@aAW^coOUuxRLN";
/// Non-syllabic phonemes of the ASCII alphabet.
const CONSONANTS: &str = "bdfghklmnprstvwzyGSZTDCJ";

/// Parse ASCII phonemes (DECtalk one-character symbols, as produced by the
/// rules and dictionary) into segments. `'` and `"` before a vowel mark
/// primary and secondary stress.
fn ascii_to_segs(ascii: &str) -> Result<Vec<Seg>, String> {
    let mut out: Vec<Seg> = vec![];
    let mut stress = 0;
    for c in ascii.chars() {
        match c {
            '\'' => stress = 1,
            '"' => stress = 2,
            _ if VOWELS.contains(c) => {
                out.push(Seg {
                    c,
                    vowel: true,
                    stress: std::mem::take(&mut stress),
                });
            }
            _ if CONSONANTS.contains(c) => {
                // Collapse doubled consonants from the spelling rules
                // ("little").
                if out.last().is_some_and(|s| s.c == c) {
                    continue;
                }
                out.push(Seg {
                    c,
                    vowel: false,
                    stress: 0,
                });
            }
            _ => return Err(format!("unknown phoneme '{c}' in {ascii}")),
        }
    }
    Ok(out)
}

/// ASCII phonemes to IPA, for the LPC concatenation synthesizer
/// ([`crate::phonemes::parse`]). Stress marks are dropped.
pub fn ascii_to_ipa(ascii: &str) -> String {
    let mut out = String::new();
    for c in ascii.chars() {
        let s = match c {
            'x' => "ə",
            'R' => "ɚ",
            'E' => "ɛ",
            '@' => "æ",
            'a' => "ɑ",
            'c' => "ɔ",
            'U' => "ʊ",
            '^' => "ʌ",
            'I' => "ɪ",
            'e' => "ɛɪ",
            'A' => "aɪ",
            'W' => "aw",
            'O' => "ɔɪ",
            'C' => "tʃ",
            'J' => "dʒ",
            'r' => "ɹ",
            'G' => "ŋ",
            'S' => "ʃ",
            'Z' => "ʒ",
            'T' => "θ",
            'D' => "ð",
            'y' => "j",
            'L' => "əl",
            'N' => "ən",
            '\'' | '"' => continue,
            _ => {
                out.push(c);
                continue;
            }
        };
        out.push_str(s);
    }
    out
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
    ("graphy", 2, SuffixStress::Before),
    ("graphers", 2, SuffixStress::Before),
    ("grapher", 2, SuffixStress::Before),
    ("logy", 2, SuffixStress::Before),
    ("logist", 2, SuffixStress::Before),
    ("logists", 2, SuffixStress::Before),
    ("nomy", 2, SuffixStress::Before),
    ("metry", 2, SuffixStress::Before),
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

/// Prefixes stripped to find a dictionary stem, with their phonemes. They are
/// left unstressed; the stem carries the primary stress.
const PREFIXES: &[(&str, &str)] = &[
    ("un", "^n"),
    ("non", "nan"),
    ("dis", "dIs"),
    ("mis", "mIs"),
    ("re", "ri"),
    ("pre", "pri"),
    ("in", "In"),
    ("im", "Im"),
    ("over", "ovR"),
    ("under", "^ndR"),
];

/// Suffixes stripped to find a dictionary stem. `None` means the phonemes
/// depends on the stem's last sound (-s, -es, -ed).
const SUFFIXES: &[(&str, Option<&str>)] = &[
    ("ly", Some("li")),
    ("ness", Some("nxs")),
    ("ments", Some("mxnts")),
    ("ment", Some("mxnt")),
    ("ful", Some("fxl")),
    ("less", Some("lxs")),
    ("ers", Some("Rz")),
    ("er", Some("R")),
    ("est", Some("xst")),
    ("ings", Some("IGz")),
    ("ing", Some("IG")),
    ("able", Some("xbxl")),
    ("ism", Some("Izxm")),
    ("ist", Some("Ist")),
    ("ize", Some("Az")),
    ("ship", Some("SIp")),
    ("hood", Some("hUd")),
    ("ed", None),
    ("es", None),
    ("s", None),
];

/// Phonemes of -s/-es or -ed after a stem whose pronunciation is `stem`.
fn inflection(stem: &str, suffix: &str) -> String {
    let last = stem.chars().rev().find(|c| !matches!(c, '\'' | '"'));
    let voiceless = matches!(last, Some('p' | 't' | 'k' | 'f' | 'T' | 's' | 'S' | 'C'));
    if suffix == "ed" {
        match last {
            Some('t' | 'd') => "Id",
            _ if voiceless => "t",
            _ => "d",
        }
    } else {
        match last {
            Some('s' | 'z' | 'S' | 'Z' | 'C' | 'J') => "Iz",
            _ if voiceless => "s",
            _ => "z",
        }
    }
    .to_string()
}

impl TextToPhoneme {
    /// Dictionary lookup, stripping affixes to find a stem.
    fn lookup(&self, word: &str) -> Option<String> {
        let r = self.lookup_inner(word);
        if self.trace
            && let Some(p) = &r
        {
            if self.dict.contains_key(word) {
                eprintln!("{word}: dictionary {p}");
            } else {
                eprintln!("{word}: via affixes {p}");
            }
        }
        r
    }

    fn lookup_inner(&self, word: &str) -> Option<String> {
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
            // Not loss → lose + s.
            if suf == "s" && stem.ends_with('s') {
                continue;
            }
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
                    let ipa = ipa.map_or_else(|| inflection(&x, suf), String::from);
                    return Some(format!("{x}{ipa}"));
                }
            }
        }
        None
    }

    /// Pronunciation of a word in ASCII phonemes: the dictionary (with affixes
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
            return format!("{x}xnt");
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
                if self.trace {
                    eprintln!("{word}: rules on stem \"{stem}{context}\" + -{suf}");
                }
                let x = self.translate(&format!(" {stem}{context} "));
                let ipa = if suf == "ed" { inflection(&x, suf) } else { "IG".into() };
                return format!("{x}{ipa}");
            }
        }
        if self.trace {
            eprintln!("{word}: rules");
        }
        self.translate(&format!(" {word} "))
    }

    /// Translate text to the ASCII phoneme format read by [`crate::ascii`],
    /// with word markers, stress and phrase boundaries.
    ///
    /// Stress comes from the dictionary when it has it; otherwise a content
    /// word is stressed on its first full (non-schwa) vowel. Function words
    /// come from a fixed list. The text is first normalized
    /// ([`crate::normalize`]): numbers, acronyms and abbreviations become
    /// words, and punctuation `,` (phrase break), `?` or `.`.
    pub fn to_ascii(&self, text: &str) -> Result<String, String> {
        use crate::normalize::{normalize, Token, LETTERS};
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
        for t in normalize(text) {
            let t = match t {
                Token::Punct(p) => {
                    toks.push(Tok::Punct(p.to_string()));
                    continue;
                }
                Token::Spell(letters, plural) => {
                    let n = letters.len();
                    for (k, l) in letters.bytes().enumerate() {
                        let mut name = LETTERS[(l - b'a') as usize].to_string();
                        // Stress on the last letter: F B I.
                        if k + 1 < n {
                            name = name.replace('\'', "\"");
                        } else if plural {
                            name.push_str(if b"sxh".contains(&l) { "Iz" } else { "z" });
                        }
                        let segs = ascii_to_segs(&name)?;
                        toks.push(Tok::Word(Word { function: false, segs }));
                    }
                    continue;
                }
                Token::Word(w) => w,
            };
            let ipa = self.pronounce(&t);
            let mut segs = ascii_to_segs(&ipa).map_err(|e| format!("{e} (\"{t}\")"))?;
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
                    let mut prev: Option<Seg> = None;
                    for seg in w.segs {
                        // Syllabic R after a tense vowel or diphthong
                        // (earlier, lawyer, higher) gets a syllable break so
                        // it isn't read as r-coloring. After a lax vowel it
                        // merges (aware: EXR). I is left out: rules give it for unstressed i
                        // (ameliorate, -ior).
                        if seg.c == 'R'
                            && seg.vowel
                            && prev.is_some_and(|p| p.vowel && !"EacU".contains(p.c))
                        {
                            s.push('-');
                        }
                        prev = Some(seg);
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
        ttp.dict.insert("fortunate", "f'crCxnxt");
        ttp.dict.insert("material", "mxt'Irixl");
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
            if let Err(e) = ascii_to_segs(out) {
                bad.push(format!("{rule}: {e}"));
            }
        }
        for (w, p) in DICTIONARY {
            if let Err(e) = ascii_to_segs(p) {
                bad.push(format!("{w}: {e}"));
            }
        }
        assert!(bad.is_empty(), "{bad:#?}");
    }
}
