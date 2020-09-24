use std::io::{Read, BufReader, BufRead};
use std::fs::*;
use std::path::PathBuf;

pub fn parse_files(files: &str) -> (Vec<PathBuf>, Vec<Vec<CurveEntry>>)
{
    let files: Vec<_> = glob::glob(files)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    
    let curve_vec: Vec<_> = files.iter()
        .map(|path| {
            let file = File::open(path).unwrap();
            parse_file(file)
        }).collect();
    (files, curve_vec)
}

pub fn parse_file<R: Read>(file: R) -> Vec<CurveEntry>
{
    let buf = BufReader::new(file);
    
    buf.lines()
        .filter_map(|v| {
            match v.ok() {
                None => None,
                Some(s) => {
                    if s.trim_start().starts_with("#") || s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                }
            } 
        }).map(|s| {
            let slice = s.trim_start();
            let mut it = slice.split(" ");
            let left = 1 + it.next().unwrap().parse::<usize>().unwrap();
            let right = 1 +  it.next().unwrap().parse::<usize>().unwrap();
            let prob = it.next().unwrap().parse::<f64>().unwrap();
            CurveEntry{
                left,
                right,
                prob
            }
        }).collect()
}

#[derive(Clone)]
pub struct Curve{
    pub left: Vec<usize>,
    pub right: Vec<usize>,
    pub probability: Vec<f64>
}

#[derive(Clone)]
pub struct CurveEntry{
    pub left: usize,
    pub right: usize,
    pub prob: f64
}

impl CurveEntry{
    pub fn delta(&self) -> f64 {
        (self.right - self.left) as f64
    }
}