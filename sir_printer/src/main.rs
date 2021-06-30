use structopt::StructOpt;
use std::{io::{BufRead, BufReader, Read, Write}, num::NonZeroUsize};
use std::fs::File;
use std::env;
use flate2::read::*;
use lzma::LzmaReader;
use glob;

fn main() {
    let opt = Opt::from_args();

    println!("#v{}", env!("CARGO_PKG_VERSION"));
    print!("#");
    for arg in env::args() {
        print!(" {}", arg);
    }
    println!();
    println!("# {:#?}", env::current_dir().unwrap().as_os_str());
    let mut curves = Vec::with_capacity(200000);
    for filename in glob::glob(&opt.filename)
        .unwrap()
        .filter_map(Result::ok)
    {
        let extension = filename.extension()
            .unwrap()
            .to_str()
            .unwrap();
        let file = match File::open(filename.as_path())
        {
            Ok(f) => f,
            Err(e) => 
            {
                eprintln!("IO ERROR: {} -> skipping {}", e, filename.display());
                continue
            }
        };
        let opt = opt.clone();
        
        match extension 
        {
            "gz" => {
                let reader = GzDecoder::new(file);
                parse_curve(reader, opt, &mut curves);
            },
            "xz" => {
                let reader = LzmaReader::new_decompressor(file).unwrap();
                parse_curve(reader, opt, &mut curves);
            },
            _ => {
                parse_curve(file, opt, &mut curves);
            }
        }
    }
    let len = curves.len();
    println!("# curves: {}", len);
    print_curves(curves, opt);
    println!("# curves: {}", len);
}



#[derive(Debug, StructOpt, Clone)]
/// Prints line of .mes file for plotting
pub struct Opt{
    /// name/glob of file(s)
    #[structopt(short, long)]
    pub filename: String,

    /// Which curve (index) to print? 
    /// If no index is given, first curve will be printed
    #[structopt(short, long)]
    pub curve: Option<usize>,

    /// print curve with specific energy
    /// Can be combined with "curve"
    #[structopt(short, long)]
    pub energy: Option<usize>,

    /// Norm the curves?
    #[structopt(short, long)]
    pub normed: bool,

    /// every nth curve
    #[structopt(long, default_value="1")]
    pub every: NonZeroUsize
}

pub(crate) fn parse_helper(slice: &str) -> Vec<usize>
{
    slice
        .split(" ")
        .skip(2)
        .map(|v| v.parse::<usize>().unwrap())
        .collect()
}

fn parse_curve<R: Read>(reader: R, opt: Opt, curves: &mut Vec<Vec<usize>>){
    let reader = BufReader::new(reader);
    if let Some(search_energy) = opt.energy
    {
        curves.extend(
            reader.lines()
            .map(|v| v.unwrap())
            .filter(|line| {
                    let t = line.trim_start();
                    !t.starts_with("#") // skip comments
                    && !t.is_empty()
                }
            ).step_by(opt.every.get())
            .filter_map(|line|{
                let slice = line.trim();
                let mut it = slice.split(" ");
                let energy = it.next().unwrap();

                let energy = energy.parse::<usize>().unwrap();
                if energy == search_energy {
                    Some(parse_helper(slice))
                }else {
                    None
                }
            })     
        )

    } else {
        
        curves.extend(
            reader.lines()
            .map(|v| v.unwrap())
            .filter(|line| {
                    let t = line.trim_start();
                    !t.starts_with("#") // skip comments
                    && !t.is_empty()
                }
            ).step_by(opt.every.get())
            .map(|line|{
                let slice = line.trim();
                parse_helper(slice)
            })
        )

    };
}

fn print_curves(curves: Vec<Vec<usize>>, opt: Opt)
{
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    match opt.curve
    {
        Some(index) => {
            match curves.get(index)
            {
                Some(c) => {
                    if opt.normed {
                        let normed = norm(c);
                        for val in normed 
                        {
                            dtoa::write(&mut out, val).unwrap();
                            writeln!(&mut out).unwrap();
                        }
                    }else {
                        for &val in c 
                        {
                            itoa::write(&mut out, val).unwrap();
                            writeln!(&mut out).unwrap();
                        }
                    }
                },
                None => {
                    eprintln!("curve {} not found! :/", index);
                }
            }
        },
        None => {
            if let Some(c) = curves.get(0) {
                let size = c.len();
                if opt.normed {
                    let normed_curves: Vec<_> = curves
                        .into_iter()
                        .map(|v| norm(&v))
                        .collect();
                    for j in 0..size {
                        for i in 0..normed_curves.len(){
                            dtoa::write(&mut out, normed_curves[i][j]).unwrap();
                            write!(&mut out, " ").unwrap();
                        }
                        writeln!(&mut out).unwrap();
                    }
                } else {
                    for j in 0..size {
                        for i in 0..curves.len(){
                            itoa::write(&mut out, curves[i][j]).unwrap();
                            write!(&mut out, " ").unwrap();
                        }
                        writeln!(&mut out).unwrap();
                    }
                }
            }else {
                eprintln!("NO curves found :(");
            }
        }
    }
}

fn norm(curve: &[usize]) -> Vec<f64>
{
    let max = *curve.iter().max().unwrap() as f64;
    let mut res = Vec::with_capacity(curve.len());
    res.extend(
        curve.into_iter().map(|&v| v as f64 / max)
    );
    res
}