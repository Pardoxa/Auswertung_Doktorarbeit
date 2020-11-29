use structopt::StructOpt;
use std::io::{Read, BufRead, BufReader};
use std::fs::File;
use flate2::read::*;
use lzma::LzmaReader;

fn main() {
    let opt = Opt::from_args();
    let file = File::open(&opt.filename).unwrap();
    if opt.filename.ends_with(".gz"){
        let reader = GzDecoder::new(file);
        print_curve(reader, opt);
    } else if opt.filename.ends_with(".xz") {
        let reader = LzmaReader::new_decompressor(file).unwrap();
        print_curve(reader, opt);
    } else {
        print_curve(file, opt);
    }
}



#[derive(Debug, StructOpt)]
/// Prints line of .mes file for plotting
pub struct Opt{
    /// name of file
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
    pub normed: bool
}

pub(crate) fn parse_helper(slice: &str) -> Vec<usize>
{
    slice
        .split(" ")
        .skip(2)
        .map(|v| v.parse::<usize>().unwrap())
        .collect()
}

fn print_curve<R: Read>(reader: R, opt: Opt){
    let reader = BufReader::new(reader);
    //let index = if let Some(index) = opt.curve {
    //    index
    //} else {
    //    0
    //};
    let curves: Vec<_> = if let Some(search_energy) = opt.energy
    {
        reader.lines()
            .map(|v| v.unwrap())
            .filter(|line| 
                !line.trim_start().starts_with("#") // skip comments
                    && !line.is_empty()
            ).filter_map(|line|{
                let slice = line.trim();
                let mut it = slice.split(" ");
                let energy = it.next().unwrap();

                let energy = energy.parse::<usize>().unwrap();
                if energy == search_energy {
                    Some(parse_helper(slice))
                }else {
                    None
                }
            }).collect()
    } else {
        
        reader.lines()
            .map(|v| v.unwrap())
            .filter(|line| 
                !line.trim_start().starts_with("#") // skip comments
                    && !line.is_empty()
            ).map(|line|{
                let slice = line.trim();
                parse_helper(slice)
            }).collect()
    };

    let size = curves[0].len();
    if opt.normed {
        let normed_curves: Vec<_> = curves.iter().map(|v| norm(v)).collect();
        for j in 0..size {
            for i in 0..normed_curves.len(){
                print!("{} ", normed_curves[i][j]);
            }
            println!();
        }
    } else {
        for j in 0..size {
            for i in 0..curves.len(){
                print!("{} ", curves[i][j]);
            }
            println!()
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