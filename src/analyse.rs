use crate::HeatmapOpts;
use crate::stats::*;
use average::Mean;
use std::iter;
use indicatif::*;
use std::convert::*;
use rayon::prelude::*;
use rayon;


pub fn compare_curves(data: Vec<Vec<Vec<f64>>>, p_bar: bool, cutoff: usize) -> Stats
{
    let mut diff_helper = Vec::new();
    let mut stats = Stats::new(&data);

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
            let iteration_count = diff_helper.len();
            let res: Mean = diff_helper.iter().collect();
            stats.push_unchecked(i, j, res.mean(), iteration_count);
            for b in bar.iter(){
                b.inc(u64::try_from(data[i].len() * data[j].len()).unwrap());
            }
            
        }
    }
    stats
    
}

pub struct JobRes{
    pub mean: f64,
    pub iterations: usize,
    pub i: usize,
    pub j: usize,
}


#[derive(Copy, Clone)]
pub struct CompareRes{
    pub mean: f64,
    pub iterations: usize,
}

impl From<JobRes> for CompareRes{
    fn from(mat_res: JobRes) -> Self {
        Self{
            mean: mat_res.mean,
            iterations: mat_res.iterations,
        }
    }
}

impl Default for CompareRes{
    fn default() -> Self {
        Self{
            mean: f64::NAN,
            iterations: 0,
        }
    }
}

pub fn compare_curves_parallel(data: Vec<Vec<Vec<f64>>>, num_threds: usize, p_bar: bool, cutoff: usize) -> Stats
{
    let mut stats = Stats::new(&data);
    

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
                let mut iteration_count = 0;
                let res: Mean = 
                if i == j {
                    (0..data[i].len())
                        .flat_map(|k| iter::repeat(k).zip(0..data[j].len()))
                        .filter(|&(k,l)| k != l)
                        .inspect(|_| iteration_count += 1)
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
                        .inspect(|_| iteration_count += 1)
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
                
                JobRes{
                    mean: res.mean(),
                    i,
                    j,
                    iterations: iteration_count
                }
            }
        ).collect_into_vec(&mut results);
    });
    

    for r in results {
        stats.push_job_res_unchecked(r);
        
    }
    stats
    
}

pub fn write_matr(stats: Stats, opts: HeatmapOpts)
{
    let mut stats_writer = StatsWriter::new_from_heatmap_opts(opts.clone());
    stats_writer.write_stats(stats);
}