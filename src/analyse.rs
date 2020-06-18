use crate::HeatmapOpts;
use average::Mean;
use std::path::Path;
use std::fs::*;
use std::io::*;
use std::env;
use std::iter;
use indicatif::*;
use std::convert::*;
use rayon::prelude::*;
use rayon;


pub fn get_cmd_args() -> String 
{
    // get cmd arguments
    let args: Vec<String> = env::args().collect();
    args.join(" ")
}

pub fn group_data(data: Vec<(usize, Vec<f64>)>, opts: HeatmapOpts) -> Vec<Vec<Vec<f64>>>
{
    let index = |energy| (energy - 1) / opts.bin_size;
    let mut vec = vec![Vec::new(); opts.bins];
    for (energy, mut curve) in data {
        
        // find max
        let mut max = curve[0];
        for i in 1..curve.len(){
            if max < curve[i]{
                max = curve[i];
            }
        }
        let inverse = 1.0 / max;
        for i in 0..curve.len(){
            curve[i] *= inverse;
        }
        vec[index(energy)].push(curve);
    }
    vec
}

pub fn compare_curves(data: Vec<Vec<Vec<f64>>>, p_bar: bool, cutoff: usize) -> Vec<Vec<f64>>
{
    let mut diff_helper = Vec::new();
    let mut matr = vec![ vec![f64::NAN; data.len()]; data.len()];

    let mut workload = 0u64;
    for i in 0..data.len(){
        for j in 0..data.len(){
            if i < j{
                continue;
            }
            workload += u64::try_from(data[i].len()).unwrap() * u64::try_from(data[j].len()).unwrap();
        }
    }
    let bar = if p_bar{
        let b = ProgressBar::new(workload);
        b.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise} - {eta_precise}] {wide_bar}"));
        Some(b)
    }else{
        None
    };
     
    

    for i in 0..data.len(){
        if data[i].len() < cutoff {
            continue;
        }
        for j in 0..data.len(){
            if i < j {
                continue;
            } else if data[j].len() < cutoff { // check that there is actually at least 2 curves available
                continue;
            }
            diff_helper.clear();
            for (index_c1, c1) in data[i].iter().enumerate(){
                for (index_c2, c2) in data[j].iter().enumerate(){
                    // do not compare curve with itself
                    if i == j && index_c1 == index_c2 {
                        continue;
                    }
                    let mean: Mean =
                        c1.iter()
                            .zip(c2.iter())
                            .map(
                                |(val1, val2)| (val1 - val2).abs()
                            ).collect();
                    diff_helper.push(mean.mean());
                }
            }
            let res: Mean = diff_helper.iter().collect();
            matr[i][j] = res.mean();
            matr[j][i] = matr[i][j];
            for b in bar.iter(){
                b.inc(u64::try_from(diff_helper.len()).unwrap());
            }
        }
    }
    matr
    
}

pub struct MatRes{
    pub mean: f64,
    pub i: usize,
    pub j: usize,
}

pub fn compare_curves_parallel(data: Vec<Vec<Vec<f64>>>, num_threds: usize, p_bar: bool, cutoff: usize) -> Vec<Vec<f64>>
{
    let mut matr = vec![ vec![f64::NAN; data.len()]; data.len()];
    

    // calculating workload, create jobs
    let mut workload = 0u64;
    let mut jobs = Vec::new();
    for i in 0..data.len(){
        if data[i].len() < cutoff{
            continue;
        }
        for j in 0..data.len() {
            if i < j {
                continue;
            } else if data[j].len() < cutoff { // check that there is actually at least 2 curves available
                continue;
            }
            let work = u64::try_from(data[i].len()).unwrap() * u64::try_from(data[j].len()).unwrap();
            workload += work;
            jobs.push((i, j));
        }

    }

    let bar = if p_bar{
        let b = ProgressBar::new(workload);
        b.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise} - {eta_precise}] {wide_bar}"));
        Some(b)
    }else{
        None
    };
  
    let pool = rayon::ThreadPoolBuilder::new().num_threads(num_threds).build().unwrap();

    let mut results = Vec::new();
    pool.install(||
    {
        jobs.into_par_iter().map(
            |(i, j)|
            {
                
                let len = data[i][0].len();
                let res: Mean = 
                if i == j {
                    (0..data[i].len())
                        .flat_map(|k| iter::repeat(k).zip(0..data[j].len()))
                        .filter(|&(k,l)| k != l)
                        .map(
                            |(k,l)| 
                            {
                                let mean: Mean = (0..len)
                                    .map(|index|  (data[i][k][index] - data[j][l][index]).abs() )
                                    .collect();
                                mean.mean()
                            }
                        ).collect()
                } else {
                    (0..data[i].len())
                        .flat_map(|k| iter::repeat(k).zip(0..data[j].len()))
                        .map(
                            |(k,l)| 
                            {
                                let mean: Mean = (0..len)
                                    .map(|index|  (data[i][k][index] - data[j][l][index]).abs() )
                                    .collect();
                                mean.mean()
                            }
                        ).collect()
                };
                
                   
                for b in bar.iter(){
                    b.inc(u64::try_from(data[i].len() * data[j].len()).unwrap());
                }
                
                MatRes{
                    mean: res.mean(),
                    i,
                    j,
                }
            }
        ).collect_into_vec(&mut results);
    });
    

    for r in results {
        
            matr[r.i][r.j] = r.mean;
            matr[r.j][r.i] = r.mean;
        
    }
    matr
    
}

pub fn write_matr<P: AsRef<Path>>(matr: Vec<Vec<f64>>, path: P)
{
    println!("Creating: {}", path.as_ref().to_str().unwrap());
    let writer = File::create(path).unwrap();
    let mut writer = BufWriter::new(writer);
    writeln!(writer, "#{}", get_cmd_args()).unwrap();
    writeln!(writer, "#{}", env::current_dir().unwrap().to_str().unwrap()).unwrap();
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