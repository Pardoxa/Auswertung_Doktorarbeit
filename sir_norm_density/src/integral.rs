use crate::parse_file::*;

pub fn integrate_block(curve: &[CurveEntry]) -> f64 {
    
    curve.iter()
        .map(|c_e| {
            block(c_e)
        }).sum()
    
}

fn block(curve_entry: &CurveEntry) -> f64 {
    curve_entry.delta() * (10_f64.powf(curve_entry.prob))
}

fn sum_check(curve: &[CurveEntry]) -> f64
{
    let mut sum = 0.0;
    for c_e in curve.iter()
    {
        sum += 10_f64.powf(c_e.prob);
    }
    sum
}

pub fn norm(curve: &mut [CurveEntry], n: usize)
{
    println!("sum_check: {}", sum_check(curve));
    let integral = integrate_block(&curve) / n as f64;
    println!("integral_before: {}", integral);
    let sub = integral.log10();
    for val in curve.iter_mut()
    {
        val.prob -= sub;
    }
    println!("integral_after: {}", integrate_block(&curve) / n as f64);
}