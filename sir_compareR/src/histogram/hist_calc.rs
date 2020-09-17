
 
use rand_pcg::Pcg64;
use rand::SeedableRng;
use crate::histogram::*;
use rayon::iter::*;
use indicatif::*;
use average::Mean;
use net_ensembles::sampling::bootstrap;

pub fn histogramm_parallel(hist_data: Histogram, num_threds: usize, p_bar: bool) -> Vec<(f64, f64)>
{
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threds)
        .build()
        .unwrap();
    
        let len = hist_data.hist().len();
    
    let mut results = Vec::with_capacity(len);
    let bar = if p_bar{
        let b = ProgressBar::new(len as u64);
        b.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise} - {eta_precise}] {wide_bar}"));
        Some(b)
    }else{
        None
    };
    pool.install(||
        {
            
            let v: Vec<_> = (0..len).collect();
            v.par_iter()
                .map(
                    |&index| 
                    {
                        let mean = |data: &Vec<&f64>| {
                            let mean: Mean = data.iter().copied().collect();
                            mean.mean()
                        };
                        let rng = Pcg64::seed_from_u64(index as u64);
                        let result = bootstrap(rng, 200, &hist_data.hist()[index],  mean);
                        for b in bar.iter(){
                            b.inc(1);
                        }
                        (index, result)
                    }
                    
                ).collect_into_vec(&mut results);
        }
    );

    into_reduced_hist(results)
}

pub fn into_reduced_hist(mut vec: Vec<(usize, (f64, f64))>) -> Vec<(f64, f64)>
{
    vec.sort_unstable_by_key(|entry| entry.0);
    vec.into_iter()
        .map(|(_, (mean, error))| (mean, error))
        .collect()
}

