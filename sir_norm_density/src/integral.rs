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
    curve.iter()
        .map(|v| 10_f64.powf(v.prob))
        .sum()
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


pub fn merge(curve_vec: &[Vec<CurveEntry>]) -> Vec<CurveEntry>
{
    // assert that they were normed before
    for curve in curve_vec.iter()
    {
        assert!((sum_check(curve) - 1.0).abs() < 1e-5);
    }
    let first = curve_vec[0].as_slice();
    for next in curve_vec[1..].iter()
    {
        for (c_e1, c_e2) in first.iter().zip(next.iter())
        {
            assert_eq!(c_e1.left, c_e2.left, "Dimension Error");
            assert_eq!(c_e2.right, c_e1.right, "Dimension Error");
        }
    }
    let mut res = first.to_vec();

    for curve in curve_vec[1..].iter()
    {
        res.iter_mut()
            .zip(curve.iter())
            .for_each(|(this, other)| this.prob += other.prob);
    }
    let denominator = curve_vec.len() as f64;
    res.iter_mut()
        .for_each(|entry| entry.prob /= denominator);

    res
}