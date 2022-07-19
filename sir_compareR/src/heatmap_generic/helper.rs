use std::{convert::TryFrom, str::FromStr, num::*};
use crate::parse_cmd::{GnuPalett, Opt};
use sampling::*;

#[derive(Debug, Clone)]
pub struct HeatmapGenericOpts
{
    pub hist_x: HistBuilder,
    pub hist_y: HistBuilder,
    pub x_index: usize,
    pub y_index: usize,
    pub files: String,
    pub every: NonZeroUsize,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub non_normalized: bool,
    pub gnuplot_name: String,
    pub gnuplot_output_name: String,
    pub supress_hist_error: bool,
    pub gnuplot_exec: bool,
    pub palett: GnuPalett
}

impl TryFrom<Opt> for HeatmapGenericOpts
{
    type Error = &'static str;
    fn try_from(opt: Opt) -> Result<Self, Self::Error> 
    {
        match opt
        {
            Opt::GenericHeatmap{
                hist_x,
                hist_y,
                x_index,
                y_index,
                files,
                every,
                x_label,
                y_label,
                non_normalized,
                mut name,
                gnuplot_output_name,
                supress_hist_error,
                gnuplot,
                palett
            } => {
                if x_index == y_index {
                    Err("Indizes are not allowed to be identical")
                } else {
                    let output = match gnuplot_output_name
                    {
                        Some(name) => name,
                        None => {
                            let mut name = name.as_str();
                            if name.ends_with(".gp")
                            {
                                name = &name[..name.len()-3];
                            }
                            format!("{}.pdf", name)
                        }
                    };
                    if !name.ends_with(".gp")
                    {
                        name = format!("{}.gp", name);
                    }
                    Ok(
                        Self{
                            hist_x,
                            hist_y,
                            y_index,
                            x_index,
                            files,
                            every,
                            y_label,
                            x_label,
                            non_normalized,
                            gnuplot_name: name,
                            gnuplot_output_name: output,
                            supress_hist_error,
                            gnuplot_exec: gnuplot,
                            palett,
                        }
                    )
                }  
            },
            _ => Err("wrong arm")
        }    
    }
}

#[derive(Debug, Clone)]
pub enum HistBuilder{
    F64Hist {
        bins: usize,
        left: f64,
        right: f64,
    },
    IsizeHist {
        bins: usize,
        left: isize,
        right: isize,
    }
}

#[derive(Debug, Clone)]
pub enum HistWrapper{
    Isize {hist: HistIsize},
    F64 {hist: HistF64}
}

impl HistBuilder 
{

    pub fn build(&self) -> Option<HistWrapper>
    {
        match self{
            HistBuilder::F64Hist {bins, left, right} => {
                Some(
                    HistWrapper::F64{
                        hist: HistF64::new(*left, *right, *bins).ok()?
                    }
                )
            },
            HistBuilder::IsizeHist { bins, left, right } => {
                Some(
                    HistWrapper::Isize{
                        hist: HistIsize::new(*left, *right, *bins).ok()?
                    }
                )
            }
        }
    }
}

impl FromStr for HistBuilder {
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

        if lower.starts_with('i')
        {
            let left = left.parse::<isize>().unwrap();
            let right = right.parse::<isize>().unwrap();
            Ok(
                Self::IsizeHist{
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
                Self::F64Hist{
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