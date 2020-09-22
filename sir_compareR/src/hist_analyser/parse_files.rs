use crate::PercentOpts;
use lzma::LzmaReader;
use flate2::read::*;
use std::io::*;
use std::fs::*;
use net_ensembles::sampling::*;
use std::result::Result;
use std::path::Path;
use indicatif::*;
use crate::heatmap2::*;
use crate::hist_analyser::*;
use rayon::prelude::*;


fn parse_and_count<R>
(
    reader: R, 
    every: usize, 
    fun: FunctionChooser,
    hist_percent: &mut HistSampler<usize, HistUsize>
)
where
    R: Read,
{
    let buf_reader = BufReader::new(reader);


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
                
                parse_into_percent_res(slice, fun, hist_percent);
                
            }
        );
}

pub(crate) fn parse_into_percent_res
(
    slice: &str,
    fun: FunctionChooser,
    hist_percent: &mut HistSampler<usize, HistUsize>
)
{
    let mut it = slice.split(" ");
    let energy = it.next().unwrap();
    
    let energy = energy.parse::<usize>().unwrap();

    let iter = slice
        .split(" ")
        .skip(2)
        .map(|v| v.parse::<usize>().unwrap());
    
    let val = fun.usize_exec(iter);

    hist_percent.count(energy, val);
}

pub fn parse_and_count_all_files(opts: &PercentOpts) -> HistSampler<usize, HistUsize>
{
    
    let files: Vec<_> = glob::glob(&opts.files)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    
    let energy_hist = HistUsize::new(1, opts.n + 1, opts.bins)
        .expect("failed to create energy hist");

    let mut hist_percent = HistSampler::new(energy_hist);
    
    let hist_percent_vec: Vec<_> = files.par_iter()
        .progress()
        .map(|entry|
            {
                let mut tmp_hist_percent = hist_percent.clone();
                parse_and_count_file(entry, opts.every, opts.fun, &mut tmp_hist_percent);
                tmp_hist_percent
            }
        ).collect();
    
    for other in hist_percent_vec {
        hist_percent.dirty_add(&other)
    }
    
    hist_percent
}



pub fn parse_and_count_file<P>
(
    filename: P,
    every: usize,
    fun: FunctionChooser,
    hist_percent: &mut HistSampler<usize, HistUsize>
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
            parse_and_count(decoder, every, fun, hist_percent)
        },
        "xz" => {
            let decoder = LzmaReader::new_decompressor(file).unwrap();
            parse_and_count(decoder, every, fun, hist_percent)
        },
        _ => {
            parse_and_count(file, every, fun, hist_percent)
        }
    }

}