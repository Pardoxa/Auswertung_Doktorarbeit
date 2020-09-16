use std::io::*;
use std::fs::*;
use flate2::read::*;
use std::path::Path;
use glob;
use crate::parse_cmd::*;
use crate::stats::Data;
use std::result::Result;
use lzma::LzmaReader;

#[derive(Debug, Copy, Clone)]
pub enum DataMode{
    Naive,
    Sparse,
}

fn norm_vec(vec: &mut Vec<f64>){
    let mut max = vec[0];
    for i in 1..vec.len() {
        if max < vec[i] {
            max = vec[i];
        }
    }
    let inverse = 1.0 / max;
    for i in 0..vec.len() {
        vec[i] *= inverse;
    }
}

pub(crate) fn parse_helper(slice: &str) -> Vec<f64>
{
    slice
        .split(" ")
        .skip(2)
        .map(|v| v.parse::<f64>().unwrap())
        .collect()
}

fn parse_and_group<R, F>
(
    reader: R, 
    every: usize,
    data: &mut Data,
    index_func: F,
    norm: bool,
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
                let extinction_index = it.next().unwrap();
                let energy = energy.parse::<usize>().unwrap();
                let extinction_index = extinction_index.parse::<usize>().unwrap();

                let mut vec: Vec<f64> = if data.is_inside_len_set() {
                    slice
                        .split(" ")
                        .skip(2)
                        .take(extinction_index + 1)
                        .map(|v| v.parse::<f64>().unwrap())
                        .collect()
                } else {
                    let mut vec: Vec<_> = parse_helper(slice);
                    data.set_inside_len(vec.len());
                    vec.truncate(extinction_index + 1);
                    vec
                };
                
                if norm {
                    norm_vec(&mut vec);
                }
                
                // append to correct bin
                data.push(index_func(energy), vec);
            }
        );
}

fn parse_and_group_naive<R, F>
(
    reader: R, 
    every: usize, 
    data: &mut Data,
    index_func: F,
    norm: bool,
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
                
                let mut vec: Vec<f64> = parse_helper(slice);
                
                if norm {
                    norm_vec(&mut vec);
                }

                // append to correct bin
                data.push(index_func(energy), vec);
            }
        );
}

pub fn parse_and_group_all_files(opts: HeatmapOpts) -> Data
{
    let mut data = Data::new_from_heatmap_options(&opts);
    let index = |energy| {
        (energy - 1) / opts.bin_size
    };
    for entry in glob::glob(&opts.files).unwrap().filter_map(Result::ok) {
        dbg!(&entry);
        parse_and_group_file(entry, opts.every, &mut data, index, opts.data_mode, opts.norm);
    }
    data
}



pub fn parse_and_group_file<P, F>
(
    filename: P,
    every: usize,
    data: &mut Data,
    index_func: F,
    data_mode: DataMode,
    norm: bool
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
            match data_mode {
               DataMode::Sparse => parse_and_group(decoder, every, data, index_func, norm),
               DataMode::Naive => parse_and_group_naive(decoder, every, data, index_func, norm),
            };
        },
        "xz" => {
            let decoder = LzmaReader::new_decompressor(file).unwrap();
            match data_mode {
                DataMode::Sparse => parse_and_group(decoder, every, data, index_func, norm),
                DataMode::Naive => parse_and_group_naive(decoder, every, data, index_func, norm),
            };
        },
        _ => {
            match data_mode {
                DataMode::Sparse => parse_and_group(file, every, data, index_func, norm),
                DataMode::Naive => parse_and_group_naive(file, every, data, index_func, norm),
            };
        }
    }

}