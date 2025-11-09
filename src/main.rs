use clap::Parser;

use crate::{
    phonemes::get_phoneme,
    synth::{Params, Synth},
};

mod lpc;
mod phonemes;
mod synth;

#[derive(Parser, Debug)]
enum Cmd {
    Clip(Clip),
    Lpc(Lpc),
    Synth(SynthCmd),
    Phoneme(PhonemeCmd),
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
        let window = &preemph[istart + i * FRAME_SIZE..][..WINDOW_SIZE];
        let coeffs = lpc::Reflector::new(&window);
        println!("{:.3?} {:.3}", coeffs.ks(), coeffs.rms());
        if let Some(writer) = &mut out {
            let mut synth = Synth::new(coeffs.ks().len());
            let params = Params {
                k: coeffs.ks().into(),
                period: 140,
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
    let params = Params { k, period: 140 };
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
    let params = Params { k, period: 140 };
    for j in 0..16_000 {
        let y = synth.get_sample(&params);
        let env = simple_env(j, 16_000);
        let yi = (y * 16384. * env).clamp(-32768.0, 32767.) as i16;
        writer.write_sample(yi).unwrap();
    }
    writer.finalize().unwrap();
}

fn main() {
    let cmd = Cmd::parse();
    println!("{cmd:?}");
    match cmd {
        Cmd::Clip(args) => main_clip(args),
        Cmd::Lpc(lpc) => main_lpc(lpc),
        Cmd::Synth(synth) => main_synth(synth),
        Cmd::Phoneme(phoneme) => main_phoneme(phoneme),
    }
}
