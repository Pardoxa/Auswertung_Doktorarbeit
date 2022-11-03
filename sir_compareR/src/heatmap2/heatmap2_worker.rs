use num_traits::{Zero, AsPrimitive};
use ord_subset::{OrdSubset, OrdSubsetIterExt};
use std::str::FromStr;
use crate::heatmap2::*;
use sampling::*;
use either::*;
use std::fmt;
use fmt::Display;
use std::any::{TypeId, Any};

#[derive(Debug, Clone, Copy)]
pub enum FunctionChooser{
    ValMax,
    ValMin,
    IndexMax,
    IndexMin,
    LastIndexNotZero,
    FromXToY(f64, f64),
    /// sum for usize, otherwise sum * energy (fraction)
    Sum
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

// this will be optimized out: https://godbolt.org/z/jdvdxjYhn
#[inline]
pub fn cast_or_panic<T, U>(val: U) -> T
where U: Any, 
T: Any
{
    if TypeId::of::<T>() == TypeId::of::<U>()
    {
        let b: Box<dyn Any> = Box::new(val);
        *b.downcast().unwrap()
    } else {
        panic!("Issue with casting - you probably did not mean to normalize the trajectories")
    }
}

impl FunctionChooser{
    pub fn f64_exec<I>(&self, iter: I, energy: usize) -> f64
    where I: Iterator<Item=f64>
    {
        match self {
            FunctionChooser::ValMax => max_val(iter),
            FunctionChooser::Sum => iter.sum::<f64>() / energy as f64,
            _ => unimplemented!()
        }
    }

    pub fn usize_exec<I, T>(&self, iter: I) -> usize
    where I: Iterator<Item=T> + Clone,
     T: Copy + OrdSubset + PartialEq + std::fmt::Debug + Any + 'static + Send + Sync + Zero + AsPrimitive<f64>,
    {
        match self {
            FunctionChooser::ValMax => {
                cast_or_panic(max_val::<T, _>(iter))
            },
            FunctionChooser::IndexMax => max_index::<T, _>(iter),
            FunctionChooser::IndexMin => min_index::<T, _>(iter),
            FunctionChooser::ValMin => {
                cast_or_panic(min_val::<T, _>(iter))
            },
            FunctionChooser::Sum => {
                let iter = iter.map(|val| cast_or_panic::<usize, _>(val));
                iter.sum()
            },
            FunctionChooser::LastIndexNotZero => {
                let mut index = 0;
                for (id, val) in iter.enumerate() {
                    if val != T::zero() {
                        index = id; 
                    }
                }
                index
            },
            FunctionChooser::FromXToY(x, y) => {
                let max_val = max_val(iter.clone());
                let p_y = max_val.as_() * y;
                let p_x = max_val.as_() * x;
                let mut index_p_x = None;
                let mut val_p_x = 0.0;
                let mut index_p_y = None;
                for (index, val) in iter.clone().enumerate()
                {
                    let val = val.as_();
                    if val >= p_x {
                        index_p_x = Some(index);
                        val_p_x = val as f64;
                        break;
                    }
                }
                for (index, val) in iter.enumerate()
                {
                    if val.as_() >= p_y {
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
            "sum" => Ok(FunctionChooser::Sum),
            _ => {
                if s.contains("to") {
                    let mut iter = s.split_whitespace();
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



pub fn max_val<T, I>(mut iter: I) -> T
where I: Iterator<Item=T>,
    T: Copy  + PartialOrd,
{
    let mut m = iter.next().unwrap();
    for val in iter {
        if val > m {
            m = val;
        }
    }
    m
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
    T: Copy + PartialOrd,
{
    let mut cur_max = iter.next().unwrap();
    let mut pos = 0;
    for (index, val) in iter.enumerate()
    {
        if cur_max < val {
            cur_max = val;
            pos = index + 1;
        }
    }
    pos
}

fn min_index<T, I>(mut iter: I) -> usize 
where I: Iterator<Item=T> + Clone,
    T: Copy + PartialOrd,
{
    let mut cur_min = iter.next().unwrap();
    let mut pos = 0;
    for (index, val) in iter.enumerate()
    {
        if cur_min > val {
            cur_min = val;
            pos = index + 1;
        }
    }
    pos
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
        let mut iter = lower.split_whitespace().skip(1);
        
        let bins = iter.next().expect("Not enough arguments, bins missing");
        let bins = bins.parse::<usize>().unwrap();
        let left = iter.next().expect("Not enough arguments, left missing");
        let right = iter.next().expect("Not enough arguments, right missing");

        assert_eq!(iter.next(), None, "HeatmapBuilder: To many arguments");

        if lower.starts_with('u')
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
        else if lower.starts_with('f')
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