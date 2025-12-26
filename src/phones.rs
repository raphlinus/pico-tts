//! Data for phones.

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
