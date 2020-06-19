use crate::HeatmapOpts;
use crate::stats::*;
use crate::parse_cmd::*;
use average::Mean;
use std::iter;
use indicatif::*;
use std::convert::*;
use rayon::prelude::*;
use rayon;


pub fn compare_curves(data: Data, p_bar: bool, cutoff: usize, mode: Mode) -> Stats
{
    let mut diff_helper = Vec::new();
    let mut stats = Stats::new(data.data());

    let mut workload = 0u64;
    for i in data.range_iter(){
        for j in data.range_iter(){
            if i < j{
                continue;
            }
            workload += u64::try_from(data.get_len_at_index(i)).unwrap() * u64::try_from(data.get_len_at_index(j)).unwrap();
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
     
    
    
    for i in data.range_iter(){
        if data.get_len_at_index(i) < cutoff {
            continue;
        }
        for j in data.range_iter() {
            if i < j {
                continue;
            } else if data.get_len_at_index(j) < cutoff { // check that there is actually at least 2 curves available
                continue;
            }
            diff_helper.clear();
            for k in 0..data.get_len_at_index(i){
                for l in 0..data.get_len_at_index(j){
                    // do not compare curve with itself
                    if i == j && k == l {
                        continue;
                    }
                    let reduced = match mode {
                        Mode::Abs => data.calc_mean(i, j, k, l, mode_abs),
                        Mode::Sqrt => data.calc_mean(i, j, k, l, mode_sqrt),
                        Mode::Cbrt => data.calc_mean(i, j, k, l, mode_cbrt),
                        Mode::Corr => data.calc_correlation(i, j, k, l),
                    };
                    diff_helper.push(reduced);
                }
            }
            let iteration_count = diff_helper.len();
            let res: Mean = diff_helper.iter().collect();
            stats.push_unchecked(i, j, res.mean(), iteration_count);
            for b in bar.iter(){
                b.inc(u64::try_from(data.get_len_at_index(i) * data.get_len_at_index(j)).unwrap());
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

#[inline]
fn mode_abs(a: f64, b: f64) -> f64 {
    (a - b).abs()
}

#[inline]
fn mode_sqrt(a: f64, b: f64) -> f64 {
    mode_abs(a,b).sqrt()
}

#[inline]
fn mode_cbrt(a: f64, b: f64) -> f64 {
    mode_abs(a,b).cbrt()
}

pub fn compare_curves_parallel(data: Data, num_threds: usize, p_bar: bool, cutoff: usize, mode: Mode) -> Stats
{
    let mut stats = Stats::new(data.data());
    

    // calculating workload, create jobs
    let mut workload = 0u64;
    let mut jobs = Vec::new();
    for i in data.range_iter(){
        if data.get_len_at_index(i) < cutoff{
            continue;
        }
        for j in data.range_iter() {
            if i < j {
                continue;
            } else if data.get_len_at_index(j) < cutoff { // check that there is actually at least 2 curves available
                continue;
            }
            let work = u64::try_from(data.get_len_at_index(i)).unwrap() * u64::try_from(data.get_len_at_index(j)).unwrap();
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
                
                //let len = data.get_inside_len();
                let mut iteration_count = 0;
                let res: Mean = 
                if i == j {
                    (0..data.get_len_at_index(i))
                        .flat_map(|k| iter::repeat(k).zip(0..data.get_len_at_index(j)))
                        .filter(|&(k,l)| k != l)
                        .inspect(|_| iteration_count += 1)
                        .map(
                            |(k,l)| 
                            {
                                match mode {
                                    Mode::Abs => data.calc_mean(i, j, k, l, mode_abs),
                                    Mode::Sqrt => data.calc_mean(i, j, k, l, mode_sqrt),
                                    Mode::Cbrt => data.calc_mean(i, j, k, l, mode_cbrt),
                                    Mode::Corr => data.calc_correlation(i, j, k, l),
                                }
                            }
                        ).collect()
                } else {
                    (0..data.get_len_at_index(i))
                        .flat_map(|k| iter::repeat(k).zip(0..data.get_len_at_index(j)))
                        .inspect(|_| iteration_count += 1)
                        .map(
                            |(k,l)| 
                            {
                                match mode {
                                    Mode::Abs => data.calc_mean(i, j, k, l, mode_abs),
                                    Mode::Sqrt => data.calc_mean(i, j, k, l, mode_sqrt),
                                    Mode::Cbrt => data.calc_mean(i, j, k, l, mode_cbrt),
                                    Mode::Corr => data.calc_correlation(i, j, k, l),
                                }
                            }
                        ).collect()
                };
                
                   
                for b in bar.iter(){
                    b.inc(u64::try_from(data.get_len_at_index(i) * data.get_len_at_index(j)).unwrap());
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