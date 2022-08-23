use crate::HeatmapOpts;
use crate::stats::*;
use crate::parse_cmd::*;
use average::Mean;
use indicatif::*;
use std::convert::*;
use rayon::prelude::*;

pub fn compare_curves(mut data: Data, p_bar: bool, cutoff: usize, mode: Mode) -> Stats
{
    let mut diff_helper = Vec::new();
    let mut stats = Stats::new(data.data());
    println!("called");
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
     
    match mode {
        Mode::Corr => {
            println!("make same len");
            data.make_same_len();
        },
        _ => ()
    };
    
    for i in data.range_iter(){
        if data.get_len_at_index(i) < cutoff {
            continue;
        }
        for j in data.range_iter() {
            if i < j || data.get_len_at_index(j) < cutoff {// check that there is actually at least 2 curves available
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
                        _ => unreachable!()
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

pub fn compare_curves_parallel(mut data: Data, num_threds: usize, p_bar: bool, cutoff: usize, mode: Mode) -> Stats
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
            if i < j || data.get_len_at_index(j) < cutoff {// check that there is actually at least 2 curves available
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

    match mode {
        Mode::Corr => {
            println!("make same len");
            data.make_same_len();
        },
        _ => ()
    };


    let mut results = Vec::new();
    pool.install(||
    {
        match mode {
            Mode::IndexMaxAbs | Mode::MaxValAbs => {
                let data = IndexData::to_index_max(data);
                jobs.into_par_iter().map(
                    |(i, j)|
                    {
                        //let len = data.get_inside_len();
                        let mut iteration_count = 0;
                        let mut sum = 0_isize;
                        let mut val_sum = 0.0;
                        if i == j {
                            for k in 0..data.get_len_at_index(i) {
                                for l in 0..data.get_len_at_index(j){
                                    if k != l {
                                        iteration_count += 1;
                                        match mode {
                                            Mode::IndexMaxAbs => {
                                                sum += data.abs(i, j, k, l);
                                            },
                                            Mode::MaxValAbs => {
                                                val_sum += data.abs_val(i, j, k, l);
                                            },
                                            _ => unreachable!()
                                        };
                                    }
                                }
                            }
                        } else {
                            for k in 0..data.get_len_at_index(i) {
                                iteration_count += data.get_len_at_index(j);
                                for l in 0..data.get_len_at_index(j){

                                    match mode {
                                        Mode::IndexMaxAbs => {
                                            sum += data.abs(i, j, k, l);
                                        },
                                        Mode::MaxValAbs => {
                                            val_sum += data.abs_val(i, j, k, l);
                                        },
                                        _ => unreachable!()
                                    };

                                }
                            }
                        };
                
                   
                        for b in bar.iter(){
                            b.inc(u64::try_from(data.get_len_at_index(i) * data.get_len_at_index(j)).unwrap());
                        }
                        let mean = match mode {
                            Mode::IndexMaxAbs => {
                                sum as f64 / iteration_count as f64
                            },
                            Mode::MaxValAbs => {
                                val_sum / iteration_count as f64
                            },
                            _ => unreachable!()
                        };
                        JobRes{
                            mean,
                            i,
                            j,
                            iterations: iteration_count
                        }
                }
                ).collect_into_vec(&mut results);
            },
            _ => {
                jobs.into_par_iter().map(
                    |(i, j)|
                    {
                        
                        //let len = data.get_inside_len();
                        let mut iteration_count = 0;
                        let mut sum = 0.0;
                        if i == j {
                            for k in 0..data.get_len_at_index(i) {
                                for l in 0..data.get_len_at_index(j){
                                    if k != l {
                                        iteration_count += 1;
                                        sum += match mode {
                                            Mode::Abs => data.calc_mean(i, j, k, l, mode_abs),
                                            Mode::Sqrt => data.calc_mean(i, j, k, l, mode_sqrt),
                                            Mode::Cbrt => data.calc_mean(i, j, k, l, mode_cbrt),
                                            Mode::Corr => data.calc_correlation(i, j, k, l),
                                            _ => unreachable!()
                                        };
                                    }
                                }
                            }
                        } else {
                            for k in 0..data.get_len_at_index(i) {
                                iteration_count += data.get_len_at_index(j);
                                for l in 0..data.get_len_at_index(j){
                                    
                                    sum += match mode {
                                        Mode::Abs => data.calc_mean(i, j, k, l, mode_abs),
                                        Mode::Sqrt => data.calc_mean(i, j, k, l, mode_sqrt),
                                        Mode::Cbrt => data.calc_mean(i, j, k, l, mode_cbrt),
                                        Mode::Corr => data.calc_correlation(i, j, k, l),
                                        _ => unreachable!()
                                    };
                                    
                                }
                            }
                        };
                        
                           
                        for b in bar.iter(){
                            b.inc(u64::try_from(data.get_len_at_index(i) * data.get_len_at_index(j)).unwrap());
                        }
                        
                        JobRes{
                            mean: sum / iteration_count as f64,
                            i,
                            j,
                            iterations: iteration_count
                        }
                    }
                ).collect_into_vec(&mut results);
            }
        };
        
    });

    for r in results {
        stats.push_job_res_unchecked(r);
        
    }
    stats
    
}

pub fn write_matr(stats: Stats, opts: HeatmapOpts)
{
    let mut stats_writer = StatsWriter::new_from_heatmap_opts(opts);
    stats_writer.write_stats(stats);
    stats_writer.mean_writer.finish().unwrap();
}