use crate::HistogramOpts;
use lzma::LzmaReader;
use flate2::read::*;
use std::io::*;
use std::fs::*;
use crate::histogram::*;
use crate::parse_files::parse_helper;
use std::result::Result;
use std::path::Path;

fn parse_and_group_naive<R, F>
(
    reader: R, 
    every: usize, 
    data: &mut Histogram,
    index_func: F,
    reduce: HistReduce,
)
where
    F: Fn(usize) -> usize,
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

                // append to correct bin
                data.push(index_func(energy), res);
            }
        );
}

pub fn parse_and_group_all_files(opts: HistogramOpts) -> Histogram
{
    let mut hist_data = Histogram::new(opts.bins);
    let index = |energy| (energy - 1) / opts.bin_size;
    let files: Vec<_> = glob::glob(&opts.files)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    
    files.iter()
        .for_each(|entry|
            {
                parse_and_group_file(entry, opts.every, &mut hist_data, index, opts.hist_reduce);
            }
        );
    hist_data
}



pub fn parse_and_group_file<P, F>
(
    filename: P,
    every: usize,
    data: &mut Histogram,
    index_func: F,
    hist_reduce: HistReduce
)
where P: AsRef<Path>,
    F: Fn(usize) -> usize,
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
            parse_and_group_naive(decoder, every, data, index_func, hist_reduce)
        },
        "xz" => {
            let decoder = LzmaReader::new_decompressor(file).unwrap();
            parse_and_group_naive(decoder, every, data, index_func, hist_reduce)
        },
        _ => {
            parse_and_group_naive(file, every, data, index_func, hist_reduce)
        }
    }

}