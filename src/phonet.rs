//! Phoneme to Klatt parameters.

use crate::{
    klatt::KlattParams,
    phones::{Flags, Phone, nonvocalic_target, vocalic_target, vocalic_target_glide},
};

/// A state machine to convert a sequence of phonemes into frames for
/// Klatt synthesis.
#[derive(Default)]
pub struct Phonet {
    // Note: this will become a window
    cur_phone: Option<Phone>,
    phone_dur: u16,
    /// Time in ms since phone start
    time_rel: u16,
}

/// Frame time in ms
const FRAME_TIME: u16 = 5;

impl Phonet {
    pub fn inp_ready(&self) -> bool {
        self.cur_phone.is_none()
    }

    pub fn push_phone(&mut self, phone: Phone) {
        if let Some(dur) = crate::phones::phone_duration(phone) {
            self.cur_phone = Some(phone);
            self.phone_dur = dur.inherent_duration;
            self.time_rel = 0;
        }
    }

    pub fn get_frame(&mut self, out: &mut KlattParams) {
        if let Some(phone) = self.cur_phone {
            let target = vocalic_target(phone).or_else(|| nonvocalic_target(phone));
            if let Some(target) = target {
                target.update(out);
                if let Some(glide) = vocalic_target_glide(phone) {
                    let t = self.time_rel as f32 / self.phone_dur as f32;
                    glide.lerp(target, out, t);
                }
                if phone.is(Flags::VOICED) {
                    out.f0 = 160.;
                    out.av += 40.;
                } else {
                    out.f0 = 0.;
                }
            }
            self.time_rel += FRAME_TIME;
            if self.time_rel >= self.phone_dur {
                self.cur_phone = None;
            }
        }
    }
}
