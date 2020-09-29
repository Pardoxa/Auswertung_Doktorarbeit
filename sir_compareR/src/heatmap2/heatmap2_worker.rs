use ord_subset::{OrdSubset, OrdSubsetIterExt};
use std::str::FromStr;
use crate::heatmap2::*;
use net_ensembles::sampling::*;
use either::*;
use std::fmt;
use fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum FunctionChooser{
    ValMax,
    ValMin,
    IndexMax,
    IndexMin,
    LastIndexNotZero,
    FromXToY(f64, f64),
}

impl Display for FunctionChooser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FromXToY(x, y) => {
                write!(f, "from_{}_to_{}", x, y)
            }
            _ => {
                write!(f, "{:?}", self)
            }
        }
    }
}

impl FunctionChooser{
    pub fn f64_exec<I>(&self, iter: I) -> f64
    where I: Iterator<Item=f64>
    {
        match self {
            FunctionChooser::ValMax => max_val(iter),
            _ => unimplemented!()
        }
    }

    pub fn usize_exec<I>(&self, iter: I) -> usize
    where I: Iterator<Item=usize> + Clone
    {
        match self {
            FunctionChooser::ValMax => max_val(iter),
            FunctionChooser::IndexMax => max_index(iter),
            FunctionChooser::IndexMin => min_index(iter),
            FunctionChooser::ValMin => min_val(iter),
            FunctionChooser::LastIndexNotZero => {
                let mut index = 0;
                for (id, val) in iter.enumerate() {
                    if val != 0 {
                        index = id; 
                    }
                }
                index
            },
            FunctionChooser::FromXToY(x, y) => {
                let max_val = max_val(iter.clone());
                let p_y = max_val as f64 * y;
                let p_x = max_val as f64 * x;
                let mut index_p_x = None;
                let mut val_p_x = 0.0;
                let mut index_p_y = None;
                for (index, val) in iter.clone().enumerate()
                {
                    if val as f64 >= p_x {
                        index_p_x = Some(index);
                        val_p_x = val as f64;
                        break;
                    }
                }
                for (index, val) in iter.enumerate()
                {
                    if val as f64 >= p_y {
                        index_p_y = Some(index);
                        break;
                    }
                }
                if index_p_y.is_none() {
                    dbg!(index_p_x);
                    dbg!(max_val);
                    dbg!(val_p_x);
                    dbg!(p_y);
                }
                index_p_y.unwrap() - index_p_x.unwrap()
            }
        }
    }
}


impl FromStr for FunctionChooser {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        
        match s.to_lowercase().as_str() {
            "indexmax" | "index_max" => Ok(FunctionChooser::IndexMax),
            "val_max" | "valmax" => Ok(FunctionChooser::ValMax),
            "indexmin" | "index_min" => Ok(FunctionChooser::IndexMin),
            "val_min" | "valmin" => Ok(FunctionChooser::ValMin),
            "lastindexnotzero" | "last_index_not_zero" | "last-index-not-zero" | "last" => Ok(FunctionChooser::LastIndexNotZero),
            _ => {
                if s.contains("to") {
                    let mut iter = s.split("_");
                    let x = iter.next()
                        .ok_or("Invalid FunctionChooser requested - invalid first (x) number")?;
                    if let Some(n) = iter.next(){
                        assert_eq!(n, "to", "Invalid_request - no to?");
                    }
                    let y = iter.next()
                        .ok_or("Invalid FunctionChooser requested - invalid second (y) number")?;
                    
                    let x = x.parse::<f64>()
                        .map_err(|_| "Invalid FunctionChooser requested - unable to parse first (x) number")?;
                    let y = y.parse::<f64>()
                        .map_err(|_| "Invalid FunctionChooser requested - unable to parse second (y) number")?;
                    return Ok(Self::FromXToY(x, y));
                }
                Err("Invalid FunctionChooser requested")
            }
        }
    }
}



pub fn max_val<T, I>(iter: I) -> T
where I: Iterator<Item=T>,
    T: Copy + OrdSubset,
{
    iter.ord_subset_max()
        .unwrap()
}

pub fn min_val<T, I>(iter: I) -> T
where I: Iterator<Item=T>,
    T: Copy + OrdSubset,
{
    iter.ord_subset_min()
        .unwrap()
}

fn max_index<T, I>(mut iter: I) -> usize 
where I: Iterator<Item=T> + Clone,
    T: Copy + OrdSubset + Eq,
{
    let max = max_val(iter.clone());
    let index = iter.position(|v| v == max).unwrap();
    index
}

fn min_index<T, I>(mut iter: I) -> usize 
where I: Iterator<Item=T> + Clone,
    T: Copy + OrdSubset + Eq,
{
    let max = min_val(iter.clone());
    let index = iter.position(|v| v == max).unwrap();
    index
}


#[derive(Debug, Clone)]
pub enum HeatmapBuilder{
    F64Heatmap {
        bins: usize,
        left: f64,
        right: f64,
    },
    UsizeHeatmap {
        bins: usize,
        left: usize,
        right: usize,
    }
}

impl fmt::Display for HeatmapBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HeatmapBuilder::UsizeHeatmap{bins, left, right} => 
            {
                write!(f, "Husize")?;
                write!(f, "bins{}left{}right{}", bins, left, right)
            },
            HeatmapBuilder::F64Heatmap{bins, left, right} =>{
                write!(f, "Hf64")?;
                write!(f, "bins{}left{}right{}", bins, left, right)
            }
        }
        
    }
}

impl FromStr for HeatmapBuilder {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let lower = s.to_lowercase();
        let mut iter = lower.split(" ").skip(1);
        
        let bins = iter.next().expect("Not enough arguments, bins missing");
        let bins = bins.parse::<usize>().unwrap();
        let left = iter.next().expect("Not enough arguments, left missing");
        let right = iter.next().expect("Not enough arguments, right missing");

        assert_eq!(iter.next(), None, "HeatmapBuilder: To many arguments");

        if lower.starts_with("u")
        {
            let left = left.parse::<usize>().unwrap();
            let right = right.parse::<usize>().unwrap();
            Ok(
                Self::UsizeHeatmap{
                    bins,
                    left,
                    right
                }
            )
        } 
        else if lower.starts_with("f")
        {
            let left = left.parse::<f64>().unwrap();
            let right = right.parse::<f64>().unwrap();
            Ok(
                Self::F64Heatmap{
                    bins,
                    left,
                    right
                }
            )
        } else {
            Err("Usage: 'f bins left right'. Choose either u for unsigned (integer) or f for floats. E.g. 'u 100 0 100' ")
        }
    }
}

impl HeatmapBuilder{
    pub fn build(&self, n: usize, energy_bins: usize) -> EitherH
    {
        let energy_hist = HistUsize::new(1, n + 1, energy_bins)
            .expect("failed to create outer hist");

        match self{
            Self::F64Heatmap{bins, left, right} => 
            {
                let fun_hist = HistF64::new(*left, *right, *bins)
                    .expect("failed to create fun hist");
                let heatmap = HeatmapU::new(energy_hist, fun_hist);
                Left(heatmap)
            },
            Self::UsizeHeatmap{bins, left, right} => {
                let fun_hist = HistUsize::new(*left, *right, *bins)
                    .expect("failed to create fun hist");
                let heatmap = HeatmapU::new(energy_hist, fun_hist);
                Right(heatmap)
            }
        }
        
    }
}