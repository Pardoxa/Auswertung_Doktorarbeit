
use std::{str::FromStr, cmp::Ordering};

#[derive(Debug, Clone, Copy)]
pub enum HistReduce{
    IndexMax,
    ValMax,
    Median,
}

impl HistReduce {
    pub fn reduce(&self, arr: &[f64]) -> f64
    {
        match self {
            HistReduce::IndexMax => max_index(arr) as f64,
            HistReduce::ValMax => max_val(arr),
            HistReduce::Median => {
                let mid = arr.len() / 2;
                let mut clone = arr.to_vec();
                clone.sort_unstable_by(
                    |a,b| a.partial_cmp(b).unwrap()
                );
                clone[mid]
            }
        }
    }
}

impl FromStr for HistReduce {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        
        match s.to_lowercase().as_str() {
            "indexmax" | "index_max" => Ok(HistReduce::IndexMax),
            "val_max" | "valmax" => Ok(HistReduce::ValMax),
            "median" => Ok(HistReduce::Median),
            _ => Err("Invalid HistReduce requested")
            }
    }
}


fn max_val(arr: &[f64]) -> f64
{
    max_helper(arr).1
}

fn max_index(arr: &[f64]) -> usize
{
    max_helper(arr).0
}

fn max_helper(arr: &[f64]) -> (usize, f64)
{
    let mut max_index = 0;
    let mut max_val = f64::NEG_INFINITY;
    for (i, val) in arr.iter().copied().enumerate()
    {
        if val > max_val
        {
            max_val = val;
            max_index = i;
        }
    }
    (max_index, max_val)
}

#[derive(Clone)]
pub struct Histogram{
    hist: Vec<Vec<f64>>
}

impl Histogram{
    pub fn new(capacity: usize) -> Self
    {
        Self{
            hist: vec![Vec::new(); capacity]
        }
    }

    pub fn hist(&self) -> &Vec<Vec<f64>>
    {
        &self.hist
    }

    pub fn push(&mut self, index: usize, val: f64)
    {
        self.hist[index].push(val);
    }

    #[allow(dead_code)]
    pub fn into_vec(self) -> Vec<Vec<f64>>
    {
        self.hist
    }

    #[allow(dead_code)]
    pub fn append(&mut self, mut other: Histogram)
    {
        match self.hist.len().cmp(&other.hist.len())
        {
            Ordering::Equal => {
                for (index, mut bin) in other.hist.into_iter().enumerate(){
                    self.hist[index].append(&mut bin);
                }
            },
            Ordering::Less => {
                for index in 0..self.hist.len()
                {
                    self.hist[index].append(&mut other.hist[index]);
                }
                for vec in other.hist.into_iter().skip(self.hist.len())
                {
                    self.hist.push(vec);
                }
            },
            Ordering::Greater => {
                unreachable!("reached in append")
            }
        }
    }
}
