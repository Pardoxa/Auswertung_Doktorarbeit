use crate::parse_file::*;
use ord_subset::*;
use std::f64::consts::*;

pub fn to_rate_fun(curve: &mut [CurveEntry], n: f64){
    let factor = -LN_10 / n;
    // -N^(-1)*ln(P(E))
    curve.iter_mut()
        .for_each(|c_e| c_e.prob *= factor);
    // calculate min for norming
    let min = curve.iter()
        .ord_subset_min_by_key(|&v| v.prob)
        .unwrap().prob;
    // norm such that the min val is 0
    curve.iter_mut()
        .for_each(|c_e| c_e.prob -= min);
    
}