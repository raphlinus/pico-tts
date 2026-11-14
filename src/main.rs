use std::io::{self, BufRead};

use clap::Parser;

use crate::{
    klatt::KlattParams,
    phonemes::get_phoneme,
    synth::{Params, Synth},
};

mod klatt;
mod lpc;
mod phonemes;
mod phones;
mod phonet;
mod ascii;
mod f0;
mod prosod;
mod sequence;
mod synth;
mod text_to_phoneme;

#[cfg(feature = "rpoly")]
mod lpc_to_formants;

#[derive(Parser, Debug)]
enum Cmd {
    Clip(Clip),
    Lpc(Lpc),
    Synth(SynthCmd),
    Phoneme(PhonemeCmd),
    Say(SayCmd),
    Text(TextCmd),
    Klatt(KlattCmd),
    Phonet(PhonetCmd),
}

#[derive(Parser, Debug)]
struct Clip {
    audio_file: String,
    clip_file: String,
    #[arg(short, long)]
    out_file: String,
}

#[derive(Parser, Debug)]
struct Lpc {
    audio_file: String,
    start: f64,
    end: f64,
    #[arg(short, long)]
    #[clap(default_value = "0.9375")]
    preemph: f64,
    #[arg(short, long)]
    out_file: Option<String>,
    #[arg(short, long)]
    voiced: bool,
}

#[derive(Parser, Debug)]
struct SynthCmd {
    //#[arg(short, long)]
    out_file: String,
    coeffs: String,
}

#[derive(Parser, Debug)]
struct PhonemeCmd {
    //#[arg(short, long)]
    out_file: String,
    phoneme: String,
}

#[derive(Parser, Debug)]
struct SayCmd {
    //#[arg(short, long)]
    out_file: String,
    text: String,
}

#[derive(Parser, Debug)]
struct TextCmd {
    #[arg(short, long)]
    file: bool,
    /// Print the ASCII phoneme format (word markers, stress, boundaries)
    #[arg(long)]
    ascii: bool,
    text: String,
}

#[derive(Parser, Debug)]
struct KlattCmd {
    out_file: String,
    params: String,
}

#[derive(Parser, Debug)]
struct PhonetCmd {
    out_file: String,
    params: String,
    /// The input is ASCII phonemes (DECtalk one-character symbols)
    #[arg(long)]
    ascii: bool,
    /// The input is English text, converted with the letter-to-sound rules
    #[arg(long)]
    text: bool,
    /// Mapping of the av/avc table columns to AV/AVS
    #[arg(long, value_enum, default_value_t)]
    voicing: phonet::Voicing,
    /// Disable the nasal zero (FNZ stays at 250 Hz, cancelling the nasal pole)
    #[arg(long)]
    no_nasal_zero: bool,
    /// Disable a context rule (may be repeated)
    #[arg(long, value_enum)]
    disable: Vec<phonet::Rule>,
    /// Enable a context rule that is off by default (may be repeated)
    #[arg(long, value_enum)]
    enable: Vec<phonet::Rule>,
    /// Reading of the final ELSE rule in C.5
    #[arg(long, value_enum, default_value_t)]
    amp_else: phonet::AmpElse,
    /// Units of Tcdiph in table C-5
    #[arg(long, value_enum, default_value_t)]
    dip_units: phonet::DipUnits,
    /// Velar F2 locus rule in C.9.1
    #[arg(long, value_enum, default_value_t)]
    velar_locus: phonet::VelarLocus,
    /// Formant transition times as a percentage of the book's. At 100,
    /// short vowels between long transitions never reach their targets.
    #[arg(long, default_value_t = 70.0)]
    articulation: f32,
    /// Disable smoothing between segments (all transitions DISCON)
    #[arg(long)]
    no_smooth: bool,
    /// Coefficient updates per frame: formant frequencies and bandwidths
    /// are interpolated linearly from the previous frame in this many steps.
    #[arg(long, default_value_t = 1)]
    interp: usize,
    /// Smooth F0 with Fujisaki's accent filter, with this natural frequency
    /// in 1/s (Fujisaki's beta is typically about 20)
    #[arg(long)]
    f0_beta: Option<f32>,
    /// Which constants the F0 rules use where the book and the narrator
    /// differ
    #[arg(long, value_enum, default_value_t)]
    f0_consts: f0::F0Consts,
    /// Override the head peak (first accent) of each clause, in Hz
    #[arg(long)]
    f0_head: Option<f32>,
    /// Put the F0 peak of an unaccented phrase-final syllable at its left
    /// edge, as the narrator does (F0 steps down at the vowel onset)
    #[arg(long)]
    f0_left_final: bool,
    /// Print the synthesizer parameters of every frame to stderr
    #[arg(long)]
    dump: bool,
    /// Output gain in dB
    #[arg(long, default_value = "0")]
    gain: f32,
}

fn read_wav(filename: String) -> (hound::WavSpec, Vec<i16>) {
    let mut reader = hound::WavReader::open(&filename).expect("error opening input file");
    let spec = reader.spec();

    let samples = reader.samples().map(|x| x.unwrap()).collect::<Vec<i16>>();
    (spec, samples)
}

fn main_clip(args: Clip) {
    let (spec, samples) = read_wav(args.audio_file);
    let mut writer = hound::WavWriter::create(args.out_file, spec).unwrap();
    let clip = std::fs::read_to_string(&args.clip_file).expect("error reading clip file");
    for line in clip.lines() {
        let words = line.split_ascii_whitespace().collect::<Vec<_>>();
        if words.len() >= 2 {
            if let Ok(start) = words[1].parse::<f64>() {
                if let Ok(end) = words[2].parse::<f64>() {
                    let istart = (spec.sample_rate as f64 * start).round() as usize;
                    let iend = (spec.sample_rate as f64 * end).round() as usize;
                    for sample in &samples[istart..iend] {
                        writer.write_sample(*sample).unwrap();
                    }
                }
            }
        }
    }
    writer.finalize().unwrap();
}

fn main_lpc(args: Lpc) {
    let (spec, samples) = read_wav(args.audio_file);
    let samples_f64 = samples.iter().map(|x| *x as f64).collect::<Vec<_>>();
    let preemph = preemph(&samples_f64, args.preemph);
    let after_preemph = if args.voiced { &preemph } else { &samples_f64 };
    let istart = (spec.sample_rate as f64 * args.start).round() as usize;
    let iend = (spec.sample_rate as f64 * args.end).round() as usize;
    const FRAME_SIZE: usize = 400;
    const WINDOW_SIZE: usize = 800;
    let n_chunks = (iend - istart) / FRAME_SIZE - 1;
    let mut out = None;
    if let Some(out_file) = &args.out_file {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        out = Some(hound::WavWriter::create(out_file, spec).unwrap());
    }
    const LEN: usize = 8000;

    for i in 0..n_chunks {
        let window = &after_preemph[istart + i * FRAME_SIZE..][..WINDOW_SIZE];
        let coeffs = lpc::Reflector::new(&window);
        println!("{:.3?} {:.3}", coeffs.ks(), coeffs.rms());
        #[cfg(feature = "rpoly")]
        {
            lpc_to_formants::lpc_to_formants(coeffs.ks());
        }
        if let Some(writer) = &mut out {
            let period = if args.voiced { 140 } else { 0 };
            let mut synth = Synth::new(coeffs.ks().len());
            let params = Params {
                k: coeffs.ks().into(),
                period,
                rms: 1.0,
            };
            for j in 0..LEN {
                let y = synth.get_sample(&params);
                let yi = (y * 16384. * simple_env(j, 7000)).clamp(-32768.0, 32767.) as i16;
                writer.write_sample(yi).unwrap();
            }
        }
    }
    if let Some(writer) = out {
        writer.finalize().unwrap()
    }
}

fn simple_env(i: usize, len: usize) -> f64 {
    const FADE: usize = 700;
    if i < FADE {
        i as f64 * (1. / FADE as f64)
    } else if i < len - FADE {
        1.0
    } else if i < len {
        (len - i) as f64 * (1. / FADE as f64)
    } else {
        0.0
    }
}

fn preemph(inp: &[f64], a: f64) -> Vec<f64> {
    (0..inp.len())
        .map(|i| inp[i] - inp.get(i.wrapping_sub(1)).cloned().unwrap_or_default() * a)
        .collect()
}

fn main_synth(args: SynthCmd) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(args.out_file, spec).unwrap();
    let k: Vec<f64> = args
        .coeffs
        .split(',')
        .map(|arg| arg.trim().parse().unwrap())
        .collect();
    let mut synth = Synth::new(k.len());
    let params = Params {
        k,
        period: 140,
        rms: 1.0,
    };
    for _ in 0..16_000 {
        let y = synth.get_sample(&params);
        let yi = (y * 16384.).clamp(-32768.0, 32767.) as i16;
        writer.write_sample(yi).unwrap();
    }
    writer.finalize().unwrap();
}

fn main_phoneme(args: PhonemeCmd) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(args.out_file, spec).unwrap();
    let phoneme = get_phoneme(&args.phoneme).expect("phoneme not found");
    let mut synth = Synth::new(phoneme.ks.len());
    let k = phoneme.ks.to_vec();
    println!("{k:?} {}", phoneme.ks.len());
    let period = if phoneme.voiced { 140 } else { 0 };
    let rms = phoneme.rms * 1e-3;
    let params = Params { k, period, rms };
    for j in 0..16_000 {
        let y = synth.get_sample(&params);
        let env = simple_env(j, 16_000);
        let yi = (y * 16384. * env).clamp(-32768.0, 32767.) as i16;
        writer.write_sample(yi).unwrap();
    }
    writer.finalize().unwrap();
}

fn main_say(args: SayCmd) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(args.out_file, spec).unwrap();
    let phoneme_seq;
    if args.text.starts_with('/') {
        phoneme_seq = crate::phonemes::parse(&args.text[1..]);
    } else {
        let ttp = crate::text_to_phoneme::TextToPhoneme::new();
        let phonemes = ttp.translate(&format!(" {} ", args.text));
        phoneme_seq = crate::phonemes::parse(&phonemes);
    }
    let mut seq = crate::sequence::Sequence::new(phoneme_seq);
    while let Some(y) = seq.get() {
        let yi = (y * 16384.).clamp(-32768.0, 32767.) as i16;
        writer.write_sample(yi).unwrap();
    }
    writer.finalize().unwrap();
}

fn main_text(args: TextCmd) {
    let ttp = crate::text_to_phoneme::TextToPhoneme::new();
    if args.file {
        let file = std::fs::File::open(args.text).unwrap();
        let reader = io::BufReader::new(file);
        for word in reader.lines() {
            let w = word.unwrap();
            println!("{w}: {}", ttp.translate(&format!(" {w} ")));
        }
    } else if args.ascii {
        match ttp.to_ascii(&args.text) {
            Ok(a) => println!("{a}"),
            Err(e) => eprintln!("error: {e}"),
        }
    } else {
        println!("{}", ttp.translate(&format!(" {} ", args.text)));
    }
}

fn main_klatt(args: KlattCmd) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 10_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(args.out_file, spec).unwrap();
    let mut klatt_params = KlattParams::default();
    let mut target = None;
    let mut glide = None;
    if let Some(phone) = phones::Phone::parse(&args.params) {
        target = phones::vocalic_target(phone).or_else(|| phones::nonvocalic_target(phone));
        glide = phones::vocalic_target_glide(phone);
        if let Some(target) = target {
            target.update(&mut klatt_params);
        }
        klatt_params.f0 = 160.;
    } else {
        let params: Vec<f32> = args
            .params
            .split(',')
            .map(|arg| arg.trim().parse().unwrap())
            .collect();
        klatt_params.av = params[0];
        klatt_params.f0 = params[1];
        klatt_params.f1 = params[2];
        klatt_params.f2 = params[3];
        klatt_params.f3 = params[4];
        klatt_params.b1 = params[5];
        klatt_params.b2 = params[6];
        klatt_params.b3 = params[7];
    }
    let mut klatt = crate::klatt::Klatt::default();
    klatt.set(&klatt_params);
    const N: usize = 3_000;
    for i in 0..N {
        let t = i as f32 * (1.0 / N as f32);
        if let Some(glide) = glide {
            glide.lerp(target.unwrap(), &mut klatt_params, t);
            klatt.set(&klatt_params);
        }
        let y = klatt.process();
        let yi = (y * 16384.).clamp(-32768.0, 32767.) as i16;
        writer.write_sample(yi).unwrap();
    }
    writer.finalize().unwrap();
}

fn main_phonet(args: PhonetCmd) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 10_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    // The input is either a file of timed segments or a list of phones.
    let mut text = std::fs::read_to_string(&args.params).unwrap_or_else(|_| args.params.clone());
    if args.text {
        let ttp = crate::text_to_phoneme::TextToPhoneme::new();
        text = match ttp.to_ascii(&text) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        };
        eprintln!("{text}");
    }
    if args.ascii || args.text {
        text = match ascii::ascii_to_phono2(&text) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        };
        eprintln!("{text}");
    }
    // PHONO2 input (with C:/F: word markers) gets durations from the
    // chapter 9 rules; otherwise the input gives them directly.
    let segs = if text.split_whitespace().any(|t| t == "C:" || t == "F:") {
        match prosod::prosod(&text, args.f0_consts, args.f0_head, args.f0_left_final) {
            Ok(ann) => {
                for a in &ann {
                    let f0: Vec<String> = a
                        .seg
                        .f0
                        .iter()
                        .map(|(p, f)| format!("{p:.2}:{f:.0}"))
                        .collect();
                    eprintln!(
                        "{:>4} {:>4} ms  {:<28} {}",
                        format!("{:?}", a.seg.phone).to_uppercase(),
                        a.seg.dur,
                        a.note,
                        f0.join(" ")
                    );
                }
                ann.into_iter().map(|a| a.seg).collect()
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    } else {
        match phonet::parse_segments(&text) {
            Ok(segs) => segs,
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    };
    let mut writer = hound::WavWriter::create(args.out_file, spec).unwrap();
    let mut klatt = crate::klatt::Klatt::default();
    let samples_per_frame = (spec.sample_rate * phonet::FRAME_TIME / 1000) as usize;
    let gain = 16384. * 10f32.powf(args.gain / 20.);
    let mut peak = 0f32;
    let mut n_clip = 0;
    let mut frame_t = 0;
    let mut prev_params: Option<klatt::KlattParams> = None;
    for klatt_params in phonet::render(
        &segs,
        &phonet::Options {
            disabled: args.disable.clone(),
            enabled: args.enable.clone(),
            amp_else: args.amp_else,
            dip_units: args.dip_units,
            velar_locus: args.velar_locus,
            articulation: args.articulation,
            voicing: args.voicing,
            nasal_zero: !args.no_nasal_zero,
            smooth: !args.no_smooth,
            f0_beta: args.f0_beta,
        },
    ) {
        if args.dump {
            let p = &klatt_params;
            eprintln!(
                "{:>6} f0 {:4.0} av {:3.0} avs {:3.0} ah {:3.0} af {:3.0} a2-6 {:3.0} {:3.0} {:3.0} {:3.0} {:3.0} ab {:3.0} f1-4 {:4.0} {:4.0} {:4.0} {:4.0} b1-3 {:3.0} {:3.0} {:3.0} fnz {:3.0}",
                frame_t, p.f0, p.av, p.avs, p.ah, p.af, p.a2, p.a3, p.a4, p.a5, p.a6, p.ab, p.f1, p.f2, p.f3, p.f4, p.b1, p.b2, p.b3, p.fnz
            );
            frame_t += phonet::FRAME_TIME;
        }
        let n_steps = args.interp.max(1);
        let prev = prev_params.replace(klatt_params.clone());
        for i in 0..samples_per_frame {
            if i * n_steps % samples_per_frame == 0 {
                let k = i * n_steps / samples_per_frame + 1;
                let t = k as f32 / n_steps as f32;
                let mut p = klatt_params.clone();
                if let Some(q) = &prev {
                    let lerp = |a: f32, b: f32| a + (b - a) * t;
                    p.f1 = lerp(q.f1, p.f1);
                    p.f2 = lerp(q.f2, p.f2);
                    p.f3 = lerp(q.f3, p.f3);
                    p.f4 = lerp(q.f4, p.f4);
                    p.b1 = lerp(q.b1, p.b1);
                    p.b2 = lerp(q.b2, p.b2);
                    p.b3 = lerp(q.b3, p.b3);
                    p.fnz = lerp(q.fnz, p.fnz);
                }
                klatt.set(&p);
            }
            let y = klatt.process() * gain;
            peak = peak.max(y.abs());
            if y.abs() > 32767. {
                n_clip += 1;
            }
            writer
                .write_sample(y.clamp(-32768.0, 32767.) as i16)
                .unwrap();
        }
    }
    eprintln!(
        "peak {:.1} dBFS, {n_clip} samples clipped",
        20. * (peak / 32768.).log10()
    );

    writer.finalize().unwrap();
}

fn main() {
    let cmd = Cmd::parse();
    //println!("{cmd:?}");
    match cmd {
        Cmd::Clip(args) => main_clip(args),
        Cmd::Lpc(lpc) => main_lpc(lpc),
        Cmd::Synth(synth) => main_synth(synth),
        Cmd::Phoneme(phoneme) => main_phoneme(phoneme),
        Cmd::Say(seq) => main_say(seq),
        Cmd::Text(text) => main_text(text),
        Cmd::Klatt(klatt) => main_klatt(klatt),
        Cmd::Phonet(phonet) => main_phonet(phonet),
    }
}
