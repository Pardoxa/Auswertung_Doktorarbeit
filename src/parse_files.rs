use std::io::*;
use std::fs::*;
use flate2::read::*;
use std::path::Path;
use glob;
use crate::parse_cmd::*;
use std::result::Result;

fn parse_file_helper<R: Read>(reader: R, every: usize) -> Vec<Vec<usize>>
{
    let buf_reader = BufReader::new(reader);
    
    buf_reader.lines()
        .map(|v| v.unwrap())
        .filter(|line| 
            {
                !line.trim_start().starts_with("#") // skip comments
            }
        ).step_by(every)
        .map( |line|
        {
            let slice = line.trim();
            
            slice.split(" ")
                .map(|v| v.parse::<usize>().unwrap())
                .collect()
        }
    ).collect()
}

pub fn parse_file<P: AsRef<Path>>(filename: P, every: usize) -> Vec<Vec<usize>>
{
    let is_gz = filename.as_ref()
        .to_str()
        .unwrap()
        .ends_with("gz");
    let file = File::open(filename).unwrap();
    if is_gz {
        let decoder = GzDecoder::new(file);
        parse_file_helper(decoder, every)
    } else 
    {
        parse_file_helper(file, every)
    }
}


pub fn parse_all_files(opts: HeatmapOpts) -> Vec<Vec<usize>>
{
    let mut res = Vec::new();
    for entry in glob::glob(&opts.files).unwrap().filter_map(Result::ok) {
        res.extend(parse_file(entry, opts.every));
    }
    res
}