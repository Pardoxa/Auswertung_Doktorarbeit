use crate::Heatmap2Opts;
use lzma::LzmaReader;
use flate2::read::*;
use std::io::*;
use std::fs::*;
use crate::histogram::*;
use crate::parse_files::parse_helper;
use net_ensembles::sampling::*;
use std::result::Result;
use std::path::Path;
use indicatif::*;
use rayon::prelude::*;

fn parse_and_count<R>
(
    reader: R, 
    every: usize, 
    heatmap: &mut Heatmap<HistUsize, HistF64>,
    reduce: HistReduce,
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
                let mut it = slice.split(" ");
                let energy = it.next().unwrap();
                
                let energy = energy.parse::<usize>().unwrap();
                
                let vec: Vec<f64> = parse_helper(slice);
                let res = reduce.reduce(&vec);
                heatmap.count(energy, res)
                    .unwrap();
            }
        );
}

pub fn parse_and_count_all_files(opts: Heatmap2Opts) -> Heatmap<HistUsize, HistF64>
{
    let outer_hist = HistUsize::new(1, opts.n + 1, opts.bins_outer)
        .expect("failed to create outer hist");
    let inner_hist = HistF64::new(opts.left, opts.right, opts.bins_inner)
        .expect("failed to create inner hist");
    
    
    let files: Vec<_> = glob::glob(&opts.files)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    
    let mut heatmaps: Vec<_> = files.par_iter()
        .progress()
        .map(|entry|
            {
                let mut heatmap = Heatmap::new(outer_hist.clone(), inner_hist.clone());
                parse_and_count_file(entry, opts.every, &mut heatmap, opts.hist_reduce);
                heatmap
            }
        ).collect();
    let mut heatmap = heatmaps.pop().unwrap();
    for h in heatmaps {
        heatmap.combine(&h).unwrap();
    }
    heatmap
}



pub fn parse_and_count_file<P>
(
    filename: P,
    every: usize,
    data: &mut Heatmap<HistUsize, HistF64>,
    hist_reduce: HistReduce
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
            parse_and_count(decoder, every, data, hist_reduce)
        },
        "xz" => {
            let decoder = LzmaReader::new_decompressor(file).unwrap();
            parse_and_count(decoder, every, data, hist_reduce)
        },
        _ => {
            parse_and_count(file, every, data, hist_reduce)
        }
    }

}