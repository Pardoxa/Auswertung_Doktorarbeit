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
    let index = if let Some(index) = opt.curve {
        index
    } else {
        0
    };
    let curve = if let Some(search_energy) = opt.energy
    {
        let curve = reader.lines()
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
            }).nth(index)
            .unwrap();
        curve
    } else {
        
        let curve = reader.lines()
            .map(|v| v.unwrap())
            .filter(|line| 
                !line.trim_start().starts_with("#") // skip comments
                    && !line.is_empty()
            ).map(|line|{
                let slice = line.trim();
                parse_helper(slice)
            }).nth(index)
            .unwrap();
        curve
    };

    if opt.normed {
        let max = *curve.iter().max().unwrap();
        let max = max as f64;
        for val in curve {
            let v = val as f64 / max;
            println!("{}", v);
        }
    } else {
        for val in curve {
            println!("{}", val);
        }
    }
    

}