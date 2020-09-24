use structopt::*;
mod parse_file;
use parse_file::*;
mod integral;
use integral::*;
use std::path::PathBuf;
use std::fs::*;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::env;

fn main() {
    let opts = NormOpts::from_args();
    let (mut files, mut curves) = parse_files(&opts.files);

    if curves.len() == 1 {
        let mut curve = curves.pop().unwrap();
        let path = files.pop().unwrap();
        norm(&mut curve, opts.n);
        write_curve(path, curve, opts.n as f64);
    } else {
        unimplemented!();
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "norm", about = "Norm probability density")]
pub struct NormOpts{
    /// which file(s) to load
    #[structopt(short, long)]
    files: String,

    /// Number of nodes
    #[structopt(short, long)]
    n: usize,
}


pub fn write_curve(path: PathBuf, curve: Vec<CurveEntry>, n: f64)
{
    let file = File::open(&path).unwrap();
    
    let str = path.to_str().unwrap();
    let mut s = str.to_owned();
    s.push_str(".normed");
    println!("new_file: {:?}", &s);
    let out = File::create(s).unwrap();
    let mut out = BufWriter::new(out);

    let buf = BufReader::new(file);
    
    for line in buf.lines(){
        let line = line.unwrap();
        let trim = line.trim_start();
        if trim.starts_with("#") || trim.is_empty() {
            writeln!(out, "{}", line).unwrap();
        } else {
            break;
        }
    }
    write!(out, "#").unwrap();
    for arg in env::args()
    {
        write!(out, " {}", arg).unwrap();
    }
    writeln!(out).unwrap();
    writeln!(out, "#left_div_n right_div_n normed left right").unwrap();
    
    for entry in curve.iter()
    {
        writeln!(
            out,
            "{:e} {:e} {:e} {} {}",
            entry.left as f64 / n,
            entry.right as f64 / n,
            entry.prob,
            entry.left,
            entry.right
        ).unwrap();
    }
}
