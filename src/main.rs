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
    text: String,
}

#[derive(Parser, Debug)]
struct KlattCmd {
    out_file: String,
    params: String,
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
        klatt_params.av += 40.;
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
    }
}
