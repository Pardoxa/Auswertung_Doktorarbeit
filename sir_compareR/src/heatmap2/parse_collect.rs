use crate::Heatmap2Opts;
use lzma::LzmaReader;
use flate2::read::*;
use std::io::*;
use std::fs::*;
use net_ensembles::sampling::*;
use std::result::Result;
use std::path::Path;
use indicatif::*;
use rayon::prelude::*;
use crate::heatmap2::*;
use either::*;

pub type HeatmapUF = HeatmapU<HistUsize, HistF64>;
pub type HeatmapUU = HeatmapU<HistUsize, HistUsize>;
pub type EitherH = Either<HeatmapUF, HeatmapUU>;

fn parse_and_count<R>
(
    reader: R, 
    every: usize, 
    either_heatmap: &mut EitherH,
    reduce: FunctionChooser,
    normed: bool,
)
where
    R: Read,
{
    let buf_reader = BufReader::new(reader);
    
    let f64_heatmap_fun = |slice: &str, heatmap: &mut HeatmapUF| parse_into_heatmap_f64(slice, heatmap, reduce, normed);
    let usize_heatmap_fun = |slice: &str, heatmap: &mut HeatmapUU| parse_into_heatmap_usize(slice, heatmap, reduce, normed);


    buf_reader.lines()
        .map(|v| v.unwrap())
        .filter(|line| 
            {
                !line.trim_start().starts_with("#") // skip comments
                && !line.is_empty()
            }
        ).step_by(every)
        .for_each( |line|
            {
                let slice = line.trim();
                
                either_heatmap.as_mut()
                    .either_with
                    (
                        slice,
                        f64_heatmap_fun,
                        usize_heatmap_fun
                    );
                
                
            }
        );
}

pub(crate) fn parse_into_heatmap_f64
(
    slice: &str,
    heatmap: &mut HeatmapUF,
    fun: FunctionChooser,
    normed: bool
)
{
    let mut it = slice.split(" ");
    let energy = it.next().unwrap();
    
    let energy = energy.parse::<usize>().unwrap();

    let iter = slice
        .split(" ")
        .skip(2)
        .map(|v| v.parse::<f64>().unwrap());

    let val = if normed {
        let max = max_val(iter);
        let iter = slice
            .split(" ")
            .skip(2)
            .map(|v| v.parse::<f64>().unwrap() / max);
        fun.f64_exec(iter, energy)
    } else {
        fun.f64_exec(iter, energy)
    };

    let _ = heatmap.count(energy, val);
}


pub(crate) fn parse_into_heatmap_usize
(
    slice: &str,
    heatmap: &mut HeatmapUU,
    fun: FunctionChooser,
    normed: bool
)
{
    let mut it = slice.split(" ");
    let energy = it.next().unwrap();
    
    let energy = energy.parse::<usize>().unwrap();

    let iter = slice
        .split(" ")
        .skip(2)
        .map(|v| v.parse::<usize>().unwrap());

    let val = if normed {
        let max = max_val(iter);
        let iter = slice
            .split(" ")
            .skip(2)
            .map(|v| v.parse::<usize>().unwrap() / max);
        fun.usize_exec(iter)
    } else {
        fun.usize_exec(iter)
    };

    let _ = heatmap.count(energy, val);
}

pub fn parse_and_count_all_files(opts: &Heatmap2Opts) -> EitherH
{
    
    let files: Vec<_> = glob::glob(&opts.files)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    
    let mut heatmap_origin = opts.heatmap_builder
        .build(opts.n, opts.bins);
    
    let heatmaps: Vec<_> = files.par_iter()
        .progress()
        .map(|entry|
            {
                let mut heatmap = heatmap_origin.clone();
                parse_and_count_file(entry, opts.every, &mut heatmap, opts.fun, opts.normed);
                heatmap
            }
        ).collect();
    
    for h in heatmaps {
        match heatmap_origin.as_mut()
        {
            Left(acc) => {
                let other = h.unwrap_left();
                acc.combine(&other).unwrap();
            },
            Right(acc) => {
                let other = h.unwrap_right();
                acc.combine(&other).unwrap();
            }
        }
    }
    heatmap_origin
}



pub fn parse_and_count_file<P>
(
    filename: P,
    every: usize,
    heatmap: &mut EitherH,
    hist_reduce: FunctionChooser,
    normed: bool
)
where P: AsRef<Path>,
{
    let ending = filename.as_ref()
        .extension()
        .unwrap()
        .to_str()
        .unwrap();
    let file = File::open(&filename).unwrap();

    match ending {
        "gz" => {
            let decoder = GzDecoder::new(file);
            parse_and_count(decoder, every, heatmap, hist_reduce, normed)
        },
        "xz" => {
            let decoder = LzmaReader::new_decompressor(file).unwrap();
            parse_and_count(decoder, every, heatmap, hist_reduce, normed)
        },
        _ => {
            parse_and_count(file, every, heatmap, hist_reduce, normed)
        }
    }

}