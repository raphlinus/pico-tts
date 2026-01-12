//! Data for phones.

use bitflags::bitflags;

use crate::klatt::KlattParams;

/// Phones
///
/// Note that these are called "phonetic segments" in the book, and also commonly
/// "allophones".
///
/// The list taken from Appendix B of the book, in alphabetical order. However, qq has
/// been added, as it's referenced elsewhere but is missing from that table.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Phone {
    Aa,
    Ae,
    Ah,
    Ao,
    Aw,
    Ax,
    Axp,
    Axr,
    Ay,
    Bb,
    Ch,
    Dd,
    Dh,
    Dx,
    Eh,
    El,
    Em,
    En,
    Er,
    Exr,
    Ey,
    Ff,
    Gg,
    Gp,
    Hh,
    Hx,
    Ih,
    Ix,
    Ixr,
    Iy,
    Jj,
    Kk,
    Kp,
    Ll,
    Lx,
    Mm,
    Ng,
    Nn,
    Ow,
    Oxr,
    Oy,
    Pp,
    Qq,
    Rr,
    Rx,
    Sh,
    Sil,
    Ss,
    Th,
    Tq,
    Tt,
    Uh,
    Uw,
    Uxr,
    Vv,
    Wh,
    Ww,
    Yu,
    Yy,
    Zh,
    Zz,
}

const N_PHONE: usize = 61;

pub struct ParamTarget {
    // Actually not 100% sure what the distinction is between av and avc,
    // it doesn't seem to be explained in the book.
    #[allow(unused)]
    av: u8,
    avc: u8,
    asp: u8,
    af: u8,
    a2: u8,
    a3: u8,
    a4: u8,
    a5: u8,
    a6: u8,
    ab: u8,
    f1: u16,
    f2: u16,
    f3: u16,
    f4: u16,
    b1: u16,
    b2: u16,
    b3: u16,
}

pub struct ParamTargetGlide {
    f1: u16,
    f2: u16,
    f3: u16,
}

#[derive(Clone, Copy)]
pub struct PhoneDuration {
    #[expect(unused, reason = "will be used in phonet rules")]
    pub min_duration: u16,
    pub inherent_duration: u16,
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Flags: u32 {
        const AFFRICATE = 1;
        const ALVEOLAR = 2;
        const ASPSEG = 4;
        const DENTAL = 8;
        const DIPHTHONG = 0x10;
        const F2BACK = 0x20;
        const FRICATIVE = 0x40;
        const FRONT = 0x80;
        const GLOTTAL = 0x100;
        const HIGH = 0x200;
        const LABIAL = 0x400;
        const LATERAL = 0x800;
        const LAX = 0x1000;
        const LIQGLIDE = 0x2000;
        const LOW = 0x4000;
        const NASAL = 0x8000;
        const PALATAL = 0x1_0000;
        const PALVEL = 0x2_0000;
        const PLOSIVE = 0x4_0000;
        const RETRO = 0x8_0000;
        const RGLIDE = 0x10_0000;
        const ROUND = 0x20_0000;
        const SCHWA = 0x40_0000;
        const SONORANT = 0x80_0000;
        const STOP = 0x100_0000;
        const SYLLABIC = 0x200_0000;
        const VELAR = 0x400_0000;
        const VOICED = 0x800_0000;
        const VOWEL = 0x1000_0000;
        const WGLIDE = 0x2000_0000;
        const YGLIDE = 0x4000_0000;
    }
}

const fn inv_map(phones: &[Phone]) -> [u8; N_PHONE] {
    let mut map = [255; N_PHONE];
    let mut i = 0;
    while i < phones.len() {
        let phone = phones[i];
        map[phone as u8 as usize] = i as u8;
        i += 1;
    }
    map
}

impl Phone {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "aa" => Some(Self::Aa),
            "ae" => Some(Self::Ae),
            "ah" => Some(Self::Ah),
            "ao" => Some(Self::Ao),
            "aw" => Some(Self::Aw),
            "ax" => Some(Self::Ax),
            "axp" => Some(Self::Axp),
            "axr" => Some(Self::Axr),
            "ay" => Some(Self::Ay),
            "bb" => Some(Self::Bb),
            "ch" => Some(Self::Ch),
            "dd" => Some(Self::Dd),
            "dh" => Some(Self::Dh),
            "dx" => Some(Self::Dx),
            "eh" => Some(Self::Eh),
            "el" => Some(Self::El),
            "em" => Some(Self::Em),
            "en" => Some(Self::En),
            "er" => Some(Self::Er),
            "exr" => Some(Self::Exr),
            "ey" => Some(Self::Ey),
            "ff" => Some(Self::Ff),
            "gg" => Some(Self::Gg),
            "gp" => Some(Self::Gp),
            "hh" => Some(Self::Hh),
            "hx" => Some(Self::Hx),
            "ih" => Some(Self::Ih),
            "ix" => Some(Self::Ix),
            "ixr" => Some(Self::Ixr),
            "iy" => Some(Self::Iy),
            "jj" => Some(Self::Jj),
            "kk" => Some(Self::Kk),
            "kp" => Some(Self::Kp),
            "ll" => Some(Self::Ll),
            "lx" => Some(Self::Lx),
            "mm" => Some(Self::Mm),
            "ng" => Some(Self::Ng),
            "nn" => Some(Self::Nn),
            "ow" => Some(Self::Ow),
            "oxr" => Some(Self::Oxr),
            "oy" => Some(Self::Oy),
            "pp" => Some(Self::Pp),
            "qq" => Some(Self::Qq),
            "rr" => Some(Self::Rr),
            "rx" => Some(Self::Rx),
            "sil" => Some(Self::Sil),
            "sh" => Some(Self::Sh),
            "ss" => Some(Self::Ss),
            "th" => Some(Self::Th),
            "tq" => Some(Self::Tq),
            "tt" => Some(Self::Tt),
            "uh" => Some(Self::Uh),
            "uw" => Some(Self::Uw),
            "uxr" => Some(Self::Uxr),
            "vv" => Some(Self::Vv),
            "wh" => Some(Self::Wh),
            "ww" => Some(Self::Ww),
            "yu" => Some(Self::Yu),
            "yy" => Some(Self::Yy),
            "zh" => Some(Self::Zh),
            "zz" => Some(Self::Zz),
            _ => None,
        }
    }

    pub fn flags(self) -> Flags {
        PHONE_FLAGS[self as u8 as usize]
    }

    // True if all flags are set
    pub fn is(self, f: Flags) -> bool {
        self.flags().contains(f)
    }

    #[expect(unused, reason = "will be used in phonet rules")]
    // True if not all flags are set (any are not)
    pub fn is_not(self, f: Flags) -> bool {
        !self.flags().contains(f)
    }
}

impl ParamTarget {
    const fn new(
        av: u8,
        avc: u8,
        asp: u8,
        af: u8,
        a2: u8,
        a3: u8,
        a4: u8,
        a5: u8,
        a6: u8,
        ab: u8,
        f1: u16,
        f2: u16,
        f3: u16,
        f4: u16,
        b1: u16,
        b2: u16,
        b3: u16,
    ) -> Self {
        Self {
            av,
            avc,
            asp,
            af,
            a2,
            a3,
            a4,
            a5,
            a6,
            ab,
            f1,
            f2,
            f3,
            f4,
            b1,
            b2,
            b3,
        }
    }

    pub fn update(&self, params: &mut KlattParams) {
        params.av = self.avc as f32;
        params.ah = self.asp as f32;
        params.af = self.af as f32;
        params.a2 = self.a2 as f32;
        params.a3 = self.a3 as f32;
        params.a4 = self.a4 as f32;
        params.a5 = self.a5 as f32;
        params.a6 = self.a6 as f32;
        params.ab = self.ab as f32;
        params.f1 = self.f1 as f32;
        params.f2 = self.f2 as f32;
        params.f3 = self.f3 as f32;
        params.f4 = self.f4 as f32;
        params.b1 = self.b1 as f32;
        params.b2 = self.b2 as f32;
        params.b3 = self.b3 as f32;
    }
}

impl ParamTargetGlide {
    const fn new(f1: u16, f2: u16, f3: u16) -> Self {
        Self { f1, f2, f3 }
    }

    pub fn lerp(&self, target: &ParamTarget, params: &mut KlattParams, t: f32) {
        params.f1 = target.f1 as f32 * (1.0 - t) + self.f1 as f32 * t;
        params.f2 = target.f2 as f32 * (1.0 - t) + self.f2 as f32 * t;
        params.f3 = target.f3 as f32 * (1.0 - t) + self.f3 as f32 * t;
    }
}

impl PhoneDuration {
    const fn new(min: u16, inh: u16) -> Self {
        Self {
            min_duration: min,
            inherent_duration: inh,
        }
    }
}

macro_rules! phone_map {
    (
        const $map:ident: [u8; N_PHONE] = _;
        const $vals:ident: &[$ty:ident] = &[
            $( $el:ident: ($($arg:expr),+), )*
        ];
    ) => {
        const $map: [u8; N_PHONE] = inv_map(&[
            $( Phone::$el ),*
        ]);
        const $vals: &[$ty] = &[
            $( $ty::new( $( $arg ),+ ), )*
        ];
    };
}

// This is table C-1 from the book.
//
// One thing to note: the af value is set to 0 in this table for stops,
// as the frication amplitude is set in C.8.3.1
phone_map! {
    const NONVOCALIC_TARGET_IX: [u8; N_PHONE] = _;
    const NONVOCALIC_TARGET: &[ParamTarget] = &[
        Axp: (57, 60, 0, 0, 60, 60, 60, 60, 60, 0, 430, 1500, 2500, 3300, 120, 60, 120),
        Bb: (0, 54, 0, 0, 0, 0, 0, 0, 0, 72, 200, 900, 2100, 3300, 65, 90, 125),
        Ch: (0, 0, 0, 0, 0, 60, 75, 70, 70, 0, 300, 1700, 2400, 3300, 200, 110, 270),
        Dd: (0, 54, 0, 0, 0, 0, 0, 50, 82, 0, 200, 900, 2100, 3300, 65, 90, 125),
        Dh: (36, 54, 0, 60, 0, 0, 0, 0, 30, 54, 300, 1150, 2700, 3300, 60, 95, 185),
        Dx: (44, 60, 0, 0, 60, 60, 60, 60, 60, 0, 200, 1600, 2700, 3300, 60, 95, 185),
        El: (57, 57, 0, 0, 60, 60, 60, 60, 60, 0, 450, 800, 2850, 3000, 65, 60, 80),
        Em: (51, 57, 0, 0, 60, 60, 60, 60, 60, 0, 200, 900, 2100, 3300, 120, 60, 70),
        En: (51, 57, 0, 0, 60, 60, 60, 60, 60, 0, 200, 1600, 2700, 3300, 120, 70, 110),
        Ff: (0, 0, 31, 60, 0, 0, 0, 0, 0, 64, 400, 1130, 2100, 3300, 225, 120, 175),
        Gg: (0, 54, 0, 0, 70, 30, 30, 60, 10, 0, 250, 1600, 1900, 3300, 70, 1450, 190),
        Gp: (0, 54, 0, 0, 30, 70, 60, 62, 62, 0, 200, 1950, 2800, 3300, 120, 140, 250),
        Hh: (0, 0, 60, 0, 60, 60, 60, 60, 60, 0, 450, 1450, 2450, 3300, 300, 160, 300),
        Hx: (44, 60, 57, 0, 60, 60, 60, 60, 60, 0, 450, 1450, 2450, 3300, 200, 120, 200),
        Jj: (0, 54, 0, 0, 0, 60, 75, 70, 70, 0, 200, 1700, 2400, 3300, 50, 110, 270),
        Kk: (0, 0, 0, 0, 73, 30, 30, 60, 10, 0, 350, 1600, 1900, 3300, 280, 220, 250),
        Kp: (0, 0, 0, 0, 30, 70, 60, 62, 62, 0, 300, 1950, 2800, 3300, 150, 140, 250),
        Ll: (50, 57, 0, 0, 60, 60, 60, 60, 60, 0, 330, 1050, 2800, 3300, 50, 100, 280),
        Lx: (57, 57, 0, 0, 60, 60, 60, 60, 60, 0, 450, 800, 2850, 3300, 65, 60, 80),
        Mm: (51, 57, 0, 0, 60, 60, 60, 60, 60, 0, 480, 1050, 2100, 3300, 40, 175, 120),
        Ng: (51, 57, 0, 0, 60, 60, 60, 60, 60, 0, 480, 1600, 2050, 3300, 160, 150, 100),
        Nn: (51, 57, 0, 0, 60, 60, 60, 60, 60, 0, 480, 1400, 2700, 3300, 40, 300, 260),
        Pp: (0, 0, 0, 0, 0, 0, 0, 0, 0, 72, 300, 900, 2100, 3300, 300, 190, 185),
        Qq: (0, 0, 0, 0, 60, 60, 60, 60, 60, 0, 400, 1400, 2450, 3300, 120, 140, 250),
        Rr: (50, 57, 0, 0, 60, 60, 60, 60, 60, 0, 330, 1060, 1380, 3300, 70, 100, 120),
        Rx: (50, 57, 0, 0, 60, 60, 60, 60, 60, 0, 460, 1260, 1560, 3300, 60, 60, 70),
        Sh: (0, 0, 31, 60, 0, 60, 75, 70, 70, 0, 400, 1650, 2400, 3300, 200, 110, 280),
        Sil: (0, 0, 0, 0, 60, 60, 60, 60, 60, 0, 400, 1400, 2400, 3300, 120, 140, 250),
        Ss: (0, 0, 31, 60, 0, 0, 0, 50, 82, 0, 400, 1400, 2700, 3300, 200, 95, 220),
        Th: (0, 0, 31, 60, 0, 0, 0, 0, 30, 54, 400, 1400, 2700, 3300, 225, 95, 200),
        Tq: (0, 0, 0, 0, 0, 0, 0, 50, 82, 0, 200, 1400, 2700, 3300, 120, 140, 250),
        Tt: (0, 0, 0, 0, 0, 0, 0, 50, 82, 0, 200, 1400, 2700, 3300, 300, 120, 220),
        Vv: (40, 54, 0, 60, 0, 0, 0, 0, 0, 64, 300, 1130, 2100, 3300, 55, 95, 125),
        Wh: (0, 57, 51, 0, 60, 60, 60, 60, 60, 0, 330, 600, 2100, 3300, 150, 60, 60),
        Ww: (50, 57, 0, 0, 60, 60, 60, 60, 60, 0, 285, 610, 2150, 3300, 50, 80, 60),
        Yy: (50, 57, 0, 0, 60, 60, 60, 60, 60, 0, 240, 2070, 3020, 3300, 220, 140, 250),
        Zh: (40, 54, 0, 60, 0, 60, 75, 70, 70, 0, 300, 1650, 2400, 3300, 220, 140, 250),
        Zz: (40, 54, 0, 60, 0, 0, 0, 50, 82, 0, 300, 1400, 2700, 3300, 70, 85, 190),
    ];
}

// This is table C-2 from the book.
phone_map! {
    const VOCALIC_TARGET_IX: [u8; N_PHONE] = _;
    const VOCALIC_TARGET: &[ParamTarget] = &[
        Aa: (57, 57, 0, 0, 60, 60, 60, 60, 60, 0, 700, 1220, 2600, 3300, 130, 70, 160),
        Ae: (57, 57, 0, 0, 60, 60, 60, 60, 60, 0, 620, 1660, 2430, 3300, 70, 130, 300),
        Ah: (59, 59, 0, 0, 60, 60, 60, 60, 60, 0, 620, 1220, 2550, 3300, 80, 50, 140),
        Ao: (58, 58, 0, 0, 60, 60, 60, 60, 60, 0, 600, 990, 2570, 3300, 90, 100, 80),
        Aw: (57, 57, 0, 0, 60, 60, 60, 60, 60, 0, 640, 1230, 2550, 3300, 70, 70, 110),
        Ax: (60, 60, 0, 0, 60, 60, 60, 60, 60, 0, 550, 1260, 2470, 3300, 80, 50, 140),
        Axr: (60, 60, 0, 0, 60, 60, 60, 60, 60, 0, 680, 1170, 2380, 3300, 60, 60, 110),
        Ay: (58, 58, 0, 0, 60, 60, 60, 60, 60, 0, 660, 1200, 2550, 3300, 100, 120, 200),
        Eh: (61, 61, 0, 0, 60, 60, 60, 60, 60, 0, 530, 1680, 2500, 3300, 60, 90, 200),
        Er: (62, 62, 0, 0, 60, 60, 60, 60, 60, 0, 470, 1270, 1540, 3300, 100, 60, 110),
        Exr: (60, 60, 0, 0, 60, 60, 60, 60, 60, 0, 460, 1650, 2400, 3300, 60, 80, 140),
        Ey: (59, 59, 0, 0, 60, 60, 60, 60, 60, 0, 480, 1720, 2520, 3300, 70, 100, 200),
        Ih: (60, 60, 0, 0, 60, 60, 60, 60, 60, 0, 400, 1800, 2670, 3300, 50, 100, 140),
        Ix: (60, 60, 0, 0, 60, 60, 60, 60, 60, 0, 420, 1680, 2520, 3300, 50, 100, 140),
        Ixr: (60, 60, 0, 0, 60, 60, 60, 60, 60, 0, 320, 1900, 2900, 3300, 70, 80, 120),
        Iy: (60, 60, 0, 0, 60, 60, 60, 60, 60, 0, 310, 2200, 2960, 3300, 50, 200, 400),
        Ow: (60, 60, 0, 0, 60, 60, 60, 60, 60, 0, 540, 1100, 2300, 3300, 80, 70, 70),
        Oxr: (60, 60, 0, 0, 60, 60, 60, 60, 60, 0, 550, 820, 2200, 3300, 60, 60, 60),
        Oy: (62, 62, 0, 0, 60, 60, 60, 60, 60, 0, 550, 960, 2400, 3300, 80, 120, 160),
        Uh: (63, 63, 0, 0, 60, 60, 60, 60, 60, 0, 450, 1100, 2350, 3300, 80, 100, 80),
        Uw: (64, 64, 0, 0, 60, 60, 60, 60, 60, 0, 350, 1250, 2200, 3300, 65, 110, 140),
        Uxr: (60, 60, 0, 0, 60, 60, 60, 60, 60, 0, 360, 800, 2000, 3300, 60, 60, 80),
        Yu: (64, 64, 0, 0, 60, 60, 60, 60, 60, 0, 290, 1900, 2600, 3300, 70, 160, 220),
    ];
}

phone_map! {
    const VOCALIC_TARGET_GLIDE_IX: [u8; N_PHONE] = _;
    const VOCALIC_TARGET_GLIDE: &[ParamTargetGlide] = &[
        Ae: (650, 1490, 2470),
        Ao: (630, 1040, 2600),
        Aw: (420, 940, 2350),
        Axr: (520, 1400, 1650),
        Ay: (400, 1880, 2500),
        Eh: (620, 1530, 2530),
        Exr: (450, 1500, 1700),
        Ey: (330, 2200, 2600),
        Ih: (470, 1600, 2600),
        Ixr: (420, 1550, 1750),
        Iy: (290, 2070, 2980),
        Ow: (450, 900, 2300),
        Oxr: (490, 1300, 1500),
        Oy: (360, 1820, 2450),
        Uh: (500, 1180, 2390),
        Uw: (320, 900, 2200),
        Uxr: (390, 1150, 1500),
        Yu: (330, 1200, 2100),
    ];
}

pub fn nonvocalic_target(phone: Phone) -> Option<&'static ParamTarget> {
    let ix = NONVOCALIC_TARGET_IX[phone as u8 as usize] as usize;
    if ix < N_PHONE {
        Some(&NONVOCALIC_TARGET[ix])
    } else {
        None
    }
}

pub fn vocalic_target(phone: Phone) -> Option<&'static ParamTarget> {
    let ix = VOCALIC_TARGET_IX[phone as u8 as usize] as usize;
    if ix < N_PHONE {
        Some(&VOCALIC_TARGET[ix])
    } else {
        None
    }
}

pub fn vocalic_target_glide(phone: Phone) -> Option<&'static ParamTargetGlide> {
    let ix = VOCALIC_TARGET_GLIDE_IX[phone as u8 as usize] as usize;
    if ix < N_PHONE {
        Some(&VOCALIC_TARGET_GLIDE[ix])
    } else {
        None
    }
}

macro_rules! set_flag {
    ( $r:ident[$el:ident] = $( $arg:ident),+ ) => {
        $r[Phone::$el as u8 as usize] = Flags::empty()
            $( .union(Flags::$arg) )* ;
    };
}

const fn mk_flags() -> [Flags; N_PHONE] {
    let mut r = [Flags::empty(); N_PHONE];
    set_flag!(r[Aa] = LOW, SONORANT, SYLLABIC, VOICED, VOWEL);
    set_flag!(
        r[Ae] = DIPHTHONG,
        FRONT,
        LOW,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL
    );
    set_flag!(r[Ah] = SONORANT, SYLLABIC, VOICED, VOWEL);
    set_flag!(
        r[Ao] = DIPHTHONG,
        LOW,
        ROUND,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL
    );
    set_flag!(
        r[Aw] = DIPHTHONG,
        LOW,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL,
        WGLIDE
    );
    set_flag!(r[Ax] = SCHWA, SONORANT, SYLLABIC, VOICED, VOWEL);
    set_flag!(
        r[Axr] = DIPHTHONG,
        LOW,
        RGLIDE,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL
    );
    set_flag!(
        r[Ay] = DIPHTHONG,
        LOW,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL,
        YGLIDE
    );
    set_flag!(r[Bb] = LABIAL, PLOSIVE, STOP, VOICED);
    set_flag!(r[Ch] = AFFRICATE, PALATAL, PLOSIVE, STOP);
    set_flag!(r[Dd] = ALVEOLAR, PLOSIVE, STOP, VOICED);
    set_flag!(r[Dh] = DENTAL, FRICATIVE, STOP, VOICED);
    set_flag!(r[Dx] = ALVEOLAR, VOICED);
    set_flag!(r[Eh] = DIPHTHONG, FRONT, SONORANT, SYLLABIC, VOICED, VOWEL);
    set_flag!(r[El] = LATERAL, LIQGLIDE, SONORANT, SYLLABIC, VOICED);
    set_flag!(r[Em] = LABIAL, NASAL, SONORANT, STOP, SYLLABIC, VOICED);
    set_flag!(r[En] = ALVEOLAR, NASAL, SONORANT, STOP, SYLLABIC, VOICED);
    set_flag!(r[Er] = RETRO, SONORANT, SYLLABIC, VOICED);
    set_flag!(
        r[Exr] = DIPHTHONG,
        FRONT,
        RGLIDE,
        SONORANT,
        SYLLABIC,
        VOICED
    );
    set_flag!(r[Ey] = DIPHTHONG, FRONT, SONORANT, SYLLABIC, VOICED, YGLIDE);
    set_flag!(r[Ff] = FRICATIVE, LABIAL);
    set_flag!(r[Gg] = PLOSIVE, STOP, VELAR, VOICED);
    set_flag!(r[Gp] = PALVEL, PLOSIVE, STOP, VOICED);
    set_flag!(r[Hh] = ASPSEG, GLOTTAL, SONORANT);
    set_flag!(r[Hx] = ASPSEG, GLOTTAL, SONORANT, VOICED);
    set_flag!(
        r[Ih] = DIPHTHONG,
        FRONT,
        HIGH,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL
    );
    set_flag!(
        r[Ix] = FRONT,
        HIGH,
        SCHWA,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL
    );
    set_flag!(
        r[Ixr] = DIPHTHONG,
        FRONT,
        HIGH,
        RGLIDE,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL
    );
    set_flag!(
        r[Iy] = DIPHTHONG,
        F2BACK,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL,
        YGLIDE
    );
    set_flag!(r[Jj] = AFFRICATE, PALATAL, PLOSIVE, STOP, VOICED);
    set_flag!(r[Kk] = PLOSIVE, STOP, VELAR);
    set_flag!(r[Kp] = PALVEL, PLOSIVE, STOP);
    set_flag!(r[Ll] = LATERAL, LIQGLIDE, SONORANT, VOICED);
    set_flag!(r[Lx] = LATERAL, LIQGLIDE, SONORANT, VOICED);
    set_flag!(r[Mm] = NASAL, SONORANT, STOP, VOICED);
    set_flag!(r[Ng] = NASAL, SONORANT, STOP, VELAR, VOICED);
    set_flag!(r[Nn] = ALVEOLAR, NASAL, SONORANT, STOP, VOICED);
    set_flag!(r[Ow] = DIPHTHONG, ROUND, SONORANT, SYLLABIC, VOICED, WGLIDE);
    set_flag!(
        r[Oxr] = DIPHTHONG,
        RGLIDE,
        ROUND,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL
    );
    set_flag!(
        r[Oy] = DIPHTHONG,
        ROUND,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL,
        YGLIDE
    );
    set_flag!(r[Pp] = LABIAL, PLOSIVE, STOP);
    set_flag!(r[Qq] = GLOTTAL, STOP, VOICED);
    set_flag!(r[Rr] = LIQGLIDE, RETRO, SONORANT, VOICED);
    set_flag!(r[Rx] = LIQGLIDE, RETRO, SONORANT, VOICED);
    set_flag!(r[Sh] = FRICATIVE, PALATAL);
    set_flag!(r[Sil] = GLOTTAL);
    set_flag!(r[Ss] = ALVEOLAR, FRICATIVE);
    set_flag!(r[Th] = DENTAL, FRICATIVE);
    set_flag!(r[Tq] = ALVEOLAR, PLOSIVE, STOP, VOICED);
    set_flag!(r[Tt] = ALVEOLAR, PLOSIVE, STOP);
    set_flag!(
        r[Uh] = DIPHTHONG,
        HIGH,
        ROUND,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL
    );
    set_flag!(r[Uw] = DIPHTHONG, ROUND, SYLLABIC, VOICED, VOWEL, WGLIDE);
    // Question: why is Uxr not DIPHTHONG?
    set_flag!(r[Uxr] = HIGH, RGLIDE, SONORANT, SYLLABIC, VOICED, VOWEL);
    set_flag!(r[Vv] = FRICATIVE, LABIAL, VOICED);
    set_flag!(
        r[Wh] = ASPSEG,
        HIGH,
        LABIAL,
        LIQGLIDE,
        ROUND,
        SONORANT,
        VOICED
    );
    set_flag!(r[Ww] = HIGH, LABIAL, LIQGLIDE, ROUND, SONORANT, VOICED);
    set_flag!(
        r[Yu] = DIPHTHONG,
        F2BACK,
        HIGH,
        ROUND,
        SONORANT,
        SYLLABIC,
        VOICED,
        VOWEL,
        WGLIDE
    );
    set_flag!(r[Yy] = F2BACK, HIGH, LIQGLIDE, PALATAL, SONORANT, VOICED);
    set_flag!(r[Zh] = FRICATIVE, PALATAL, VOICED);
    set_flag!(r[Zz] = ALVEOLAR, FRICATIVE, VOICED);
    r
}

pub const PHONE_FLAGS: [Flags; N_PHONE] = mk_flags();

// This is table 9-1 from the book.
phone_map! {
    const PHONE_DURATION_IX: [u8; N_PHONE] = _;
    const PHONE_DURATION: &[PhoneDuration] = &[
        Aa: (100, 240),
        Ae: (80, 230),
        Ah: (60, 140),
        Ao: (100, 240),
        Aw: (100, 260),
        Ax: (60, 120),
        Axr: (120, 260),
        Ay: (150, 250),
        Eh: (70, 150),
        Er: (80, 180),
        Exr: (130, 270),
        Ey: (100, 190),
        Ih: (40, 135),
        Ix: (60, 110),
        Ixr: (100, 230),
        Iy: (55, 155),
        Ow: (80, 220),
        Oxr: (130, 240),
        Oy: (150, 280),
        Uh: (60, 160),
        Uw: (70, 210),
        Uxr: (110, 230),
        Yu: (150, 230),

        El: (110, 260),
        Hh: (20, 80),
        Hx: (25, 70),
        Ll: (40, 80),
        Lx: (70, 90),
        Rr: (30, 80),
        Rx: (70, 80),
        Ww: (60, 80),
        Wh: (60, 70),
        Yy: (40, 80),

        Em: (110, 170),
        En: (100, 170),
        Mm: (60, 70),
        Nn: (50, 60),
        Ng: (60, 95),

        Dh: (30, 50),
        Ff: (80, 100),
        Ss: (60, 105),
        Sh: (80, 105),
        Th: (60, 90),
        Vv: (40, 60),
        Zz: (40, 75),
        Zh: (40, 70),

        Bb: (60, 85),
        Dd: (50, 75),
        Dx: (20, 20),
        Gg: (60, 80),
        Gp: (40, 80),
        Kk: (60, 80),
        Kp: (40, 80),
        Pp: (50, 90),
        Tt: (50, 75),
        Tq: (50, 75),

        Ch: (50, 70),
        Jj: (50, 70),

        Axp: (70, 70),
    ];
}

pub fn phone_duration(phone: Phone) -> Option<PhoneDuration> {
    let ix = PHONE_DURATION_IX[phone as u8 as usize] as usize;
    if ix < N_PHONE {
        Some(PHONE_DURATION[ix])
    } else {
        None
    }
}
