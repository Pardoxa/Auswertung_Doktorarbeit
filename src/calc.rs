use crate::ReadOpts;
use average::Mean;
use std::path::Path;
use std::fs::*;
use std::io::*;
use std::env;
use indicatif::*;
use std::convert::*;

pub fn get_cmd_args() -> String 
{
    // get cmd arguments
    let args: Vec<String> = env::args().collect();
    args.join(" ")
}

pub fn group_data(data: Vec<Vec<usize>>, opts: ReadOpts) -> Vec<Vec<Vec<f64>>>
{
    let index = |energy| (energy - 1) / opts.bin_size;
    let mut vec = vec![Vec::new(); opts.bins];
    for curve in data {
        let energy = curve[0];
        // find max
        let max = *curve.iter().skip(2).max().unwrap();
        let inverse = 1.0 / max as f64;
        let normed: Vec<_> = curve.into_iter()
            .skip(2)    // skip energy and extinction time
            .map(|val|
                {
                    val as f64 * inverse
                }
            ).collect();
        vec[index(energy)].push(normed);
    }
    vec
}

pub fn compare_curves(data: Vec<Vec<Vec<f64>>>) -> Vec<Vec<f64>>
{
    let mut tmp_vec = Vec::new();
    let mut diff_helper = Vec::new();
    let mut matr = Vec::with_capacity(data.len());
    for i in 0..data.len(){
        matr.push(Vec::with_capacity(data.len()));
        for _ in 0..data.len(){
            matr[i].push(0.0);
        }
    }

    let mut workload = 0u64;
    for i in 0..data.len(){
        for j in 0..data.len(){
            if i < j{
                continue;
            }
            workload += u64::try_from(data[i].len()).unwrap() * u64::try_from(data[j].len()).unwrap();
        }
    }
    let bar = ProgressBar::new(workload);
    bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise} - {eta_precise}] {wide_bar}"));

    for i in 0..data.len(){
        for j in 0..data.len(){
            if i < j {
                continue;
            }
            diff_helper.clear();
            for (index_c1, c1) in data[i].iter().enumerate(){
                for (index_c2, c2) in data[j].iter().enumerate(){
                    // do not compare curve with itself
                    if i == j && index_c1 == index_c2 {
                        continue;
                    }
                    tmp_vec.clear();
                    tmp_vec.extend(
                        c1.iter()
                            .zip(c2.iter())
                            .map(|(val1, val2)| (val1 - val2).abs())
                    );
                    let mean: Mean = tmp_vec.iter().collect();
                    diff_helper.push(mean.mean());
                }
            }
            let res: Mean = diff_helper.iter().collect();
            matr[i][j] = res.mean();
            matr[j][i] = matr[i][j];
            bar.inc(u64::try_from(diff_helper.len()).unwrap());
        }
    }
    matr
    
}

pub fn write_matr<P: AsRef<Path>>(matr: Vec<Vec<f64>>, path: P)
{
    let mut writer = File::create(path).unwrap();
    writeln!(writer, "#{}", get_cmd_args()).unwrap();
    for line in matr{
        for (index, val) in line.iter().enumerate(){
            if index == 0 {
                write!(writer, "{:e}", val).unwrap();
            } else {
                write!(writer, " {:e}", val).unwrap();
            }
        }
        writeln!(writer).unwrap();
    }
}