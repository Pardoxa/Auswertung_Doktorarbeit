use crate::analyse::*;
use crate::*;
use std::{io::*, num::NonZeroUsize, sync::Mutex};
use std::fs::*;
use std::{env, cmp::Reverse};
use rand::prelude::SliceRandom;
use rand_pcg::Pcg64;
use rgsl::statistics::correlation;
use std::ops::*;
use rayon::prelude::*;
use lzma::LzmaWriter;
use lazy_static::*;

lazy_static! {
    static ref  LIMIT_RNG: Mutex<Pcg64> = 
    {
        Mutex::new(Pcg64::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7ac28fa16a64abf96))
    };
}


#[derive(Clone)]
pub struct Stats{
    mean: Vec<Vec<f64>>,
    iteration_count: Vec<Vec<usize>>,
    curve_count: Vec<usize>,
}

impl Stats {
    pub fn new(data: &Vec<Vec<Vec<f64>>>) -> Self
    {
        let length = data.len();
        let mean = vec![vec![f64::NAN; length]; length];
        let iteration_count = vec![vec![0; length]; length];
        let curve_count: Vec<_> = data.iter()
            .map(|entry| entry.len())
            .collect();
        Self{
            mean,
            curve_count,
            iteration_count,
        }
    }

    pub fn push_job_res_unchecked(&mut self, mat_res: JobRes)
    {
        self.push_unchecked(mat_res.i, mat_res.j, mat_res.mean, mat_res.iterations);
    }

    pub fn push_unchecked(&mut self, i: usize, j: usize, mean: f64, iteration_count: usize){
        self.mean[i][j] = mean;
        self.mean[j][i] = mean;
        
        self.iteration_count[i][j] = iteration_count;
        self.iteration_count[j][i] = iteration_count;
    }

    pub fn get_mean(&self) -> &Vec<Vec<f64>>{
        &self.mean
    } 

    pub fn get_iteration_count(&self) -> &Vec<Vec<usize>>
    {
        &self.iteration_count
    }

    pub fn get_curve_count(&self) -> &Vec<usize>
    {
        &self.curve_count
    }
}

pub fn get_cmd_args() -> String 
{
    // get cmd arguments
    let args: Vec<String> = env::args().collect();
    args.join(" ")
}

pub struct StatsWriter<W, W2>
{
    pub(crate) mean_writer: W,
    iteration_count_writer: W2,
    curve_count_writer: W2,
}

impl<W: Write, W2: Write> StatsWriter<W, W2>{

    fn write_mean(&mut self, mean: &[Vec<f64>])
    {
        for v in mean.iter(){
            let slice = &v[..v.len()-1];
            for v in slice.iter(){
                write!(self.mean_writer, "{:e} ", v).unwrap();
            }
            writeln!(self.mean_writer, "{:e}", v.last().unwrap()).unwrap();
        }
    }

    fn write_iteration_count(&mut self, iteration_count: &[Vec<usize>])
    {
        for v in iteration_count.iter(){
            let slice = &v[..v.len()-1];
            for v in slice.iter(){
                write!(self.iteration_count_writer, "{} ", v).unwrap();
            }
            writeln!(self.iteration_count_writer, "{}", v.last().unwrap()).unwrap();
        }
    }

    fn write_curve_count(&mut self, curve_count: &[usize])
    {
        for i in curve_count.iter(){
            writeln!(self.curve_count_writer, "{}", i).unwrap();
        }
    }

    pub fn write_stats(&mut self, stats: Stats)
    {
        self.write_mean(stats.get_mean());
        self.write_iteration_count(stats.get_iteration_count());
        self.write_curve_count(stats.get_curve_count());

    }
}

impl StatsWriter<File, File>{
    pub fn new_from_heatmap_opts(opts: HeatmapOpts) -> StatsWriter<LzmaWriter<BufWriter<File>>, BufWriter<File>>
    {
        let mean_name = opts.generate_filename("stats.mean.xz");
        let iteration_name = opts.generate_filename("stats.iterations");
        let curve_count_name = opts.generate_filename("stats.curve_count");
        println!("Generated:\n{}\n{}\n{}", &mean_name, &iteration_name, &curve_count_name);

        let mean_writer = File::create(mean_name).unwrap();
        let iteration_count_writer = File::create(iteration_name).unwrap();
        let curve_count_writer = File::create(curve_count_name).unwrap();

        let stats = Self{
            mean_writer,
            iteration_count_writer,
            curve_count_writer,
        };

        let stats: StatsWriter<BufWriter<File>, BufWriter<File>> = stats.into();

        let mut stats = StatsWriter{
            mean_writer: LzmaWriter::new_compressor(stats.mean_writer, 4).unwrap(),
            iteration_count_writer: stats.iteration_count_writer,
            curve_count_writer: stats.curve_count_writer
        };
        writeln!(stats, "#{}", get_cmd_args()).unwrap();
        writeln!(stats, "#{}", env::current_dir().unwrap().to_str().unwrap()).unwrap();
        stats
    }
}

impl<W: Write, W2: Write> Write for StatsWriter<W, W2>{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.mean_writer.write_all(buf)?;
        self.iteration_count_writer.write_all(buf)?;
        self.curve_count_writer.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        self.mean_writer.flush()?;
        self.iteration_count_writer.flush()?;
        self.curve_count_writer.flush()
    }
}

impl<W: Write> From<StatsWriter<W, W>> for StatsWriter<BufWriter<W>, BufWriter<W>>{
    fn from(origin: StatsWriter<W, W>) -> Self {
        Self{
            curve_count_writer: BufWriter::new(origin.curve_count_writer),
            iteration_count_writer: BufWriter::new(origin.iteration_count_writer),
            mean_writer: BufWriter::new(origin.mean_writer),
        }
    }
}


pub struct Data
{
    pub data: Vec<Vec<Vec<f64>>>,
    pub inside_len: usize,
    pub inside_len_set: bool,
}

impl Data{
    pub fn new_from_heatmap_options(opts: &HeatmapOpts) -> Self {
        let data = vec![Vec::new(); opts.bins];
        Self{
            data,
            inside_len: 0,
            inside_len_set: false,
        }
    }

    pub fn is_inside_len_set(&self) -> bool
    {
        self.inside_len_set
    }

    pub fn make_same_len(&mut self)
    {
        let max_len = self.data.iter().flat_map(
            |vec|
            {
                vec.iter().map(
                    |vec|
                    vec.len()
                )
            }
        ).max().unwrap();

        self.data.iter_mut()
            .for_each(
                |bin|
                {
                    bin.iter_mut()
                        .for_each(
                            |curve|
                            {
                                let dif = max_len - curve.len();
                                if dif > 0 
                                {
                                    let last = *curve.last().unwrap();
                                    curve.extend(
                                        std::iter::repeat(last).take(dif)
                                    );
                                }
                            }
                        )    
                }
            )

    }

    #[inline(always)]
    pub fn set_inside_len(&mut self, len: usize){
        self.inside_len = len;
        self.inside_len_set = true;
    }

    pub fn push(&mut self, index: usize, v: Vec<f64>)
    {
        self.data[index].push(v);
    }

    pub fn data(&self) -> &Vec<Vec<Vec<f64>>>
    {
        &self.data
    }

    pub fn range_iter(&self) -> Range<usize>
    {
        0..self.data.len()
    }

    #[inline(always)]
    pub fn get_len_at_index(&self, index: usize) -> usize {
        self.data[index].len()
    }


    /// calculates mean of (itemwise) reduction of two curves 
    /// cuve1: data[i][k]
    /// curve2: data[j][l]
    #[inline(always)]
    pub fn calc_mean<F>(&self, i: usize, j: usize, k: usize, l: usize, reduction: F) -> f64
    where F: Fn(f64, f64) -> f64
    {
        reduce(&self.data[i][k], &self.data[j][l], reduction)
    }


    pub fn calc_correlation(&self, i: usize, j: usize, k: usize, l: usize) -> f64
    {
        
        correlation(
            &self.data[i][k],
            1,
            &self.data[j][l],
            1,
            self.data[i][k].len()
        )

    }

    pub fn average_entries(&self) -> usize
    {
        self.data
            .iter()
            .map(|e| e.len())
            .sum::<usize>() / self.data.len()
    }

    pub fn max_n_entries(&self, n: usize) -> Vec<usize>
    {
        let mut lens: Vec<_> = self.data
            .iter()
            .map(|e| e.len())
            .collect();
        lens.sort_unstable_by_key(|&e| Reverse(e));
        lens.truncate(n);
        lens.shrink_to_fit();
        lens
    }

    pub fn print_lens(&self) {
        print!("lens:");
        self.data.iter()
            .for_each(|e| print!(" {}", e.len()));
        println!()
    }

    pub fn min_n_entries(&self, n: usize) -> Vec<usize>
    {
        let mut lens: Vec<_> = self.data
            .iter()
            .map(|e| e.len())
            .collect();
        
        lens.sort_unstable();
        lens.truncate(n);
        lens.shrink_to_fit();
        lens
    }

    pub fn limit_entries(&mut self, maximum: NonZeroUsize)
    {
        let mut rng_lock = LIMIT_RNG.lock()
            .unwrap();
        self.data
            .iter_mut()
            .filter(|v| v.len() > maximum.get())
            .for_each(
                |v|
                {
                    v.shuffle(rng_lock.deref_mut());
                    v.truncate(maximum.get());
                    v.shrink_to_fit();
                    v.iter_mut()
                        .for_each(|v| v.shrink_to_fit());
                }
            );
    }

}

pub struct IndexData{
    pub index_data: Vec<Vec<isize>>,
    pub val_max_data: Vec<Vec<f64>>
}

impl IndexData {
    pub fn to_index_max(data: Data) -> Self
    {
        let index_data: Vec<_> = data.data
            .par_iter()
            .map(|vec_vec| {
                let max_index_vec: Vec<_> = vec_vec.iter()
                    .map(|v|{
                        let max = v.iter()
                            .copied()
                            .max_by(|a,b| {
                            a.partial_cmp(b)
                                .expect("NAN ENCOUNTERED!")
                        }).expect("Max Index error");
                        let pos = v.iter()
                            .position(|&val| val == max)
                            .unwrap();
                        pos as isize
                    }
                    ).collect();
                max_index_vec
            }).collect();

        let val_max_data: Vec<_> = index_data.iter()
            .zip(data.data)
            .map(|(ind_vec, cur_vec)| {
                ind_vec.iter().zip(cur_vec.iter())
                    .map(|(i, cur)|
                    {
                        let index = *i as usize;
                        cur[index]
                    }
                ).collect()
            }).collect();
        Self{
            index_data,
            val_max_data
        }
    }

    #[inline(always)]
    pub fn abs(&self,  i: usize, j: usize, k: usize, l: usize) -> isize
    {
        (self.index_data[i][k] - self.index_data[j][l]).abs()
    }

    #[inline(always)]
    pub fn abs_val(&self,  i: usize, j: usize, k: usize, l: usize) -> f64
    {
        (self.val_max_data[i][k] - self.val_max_data[j][l]).abs()
    }

    #[inline(always)]
    pub fn get_len_at_index(&self, index: usize) -> usize {
        self.index_data[index].len()
    }
}

/// calculates mean of (itemwise) reduction of two curves 
/// cuve1: data[i][k]
/// curve2: data[j][l] 
pub fn reduce<F>(arr1: &[f64], arr2: &[f64], reduction: F) -> f64
where F: Fn(f64, f64) -> f64
{
    let ex_1 = arr1.len();
    let ex_2 = arr2.len();
    let len = ex_1.max(ex_2);
    let counter = ex_1.min(ex_2);
    
    // calculate weighted mean where both have values
    let mut sum = 0.0;
    for i in 0..counter{
        sum += reduction(arr1[i], arr2[i]);
    }
    
    if ex_1 == ex_2 && ex_1 == len
    {
        sum / counter as f64
    } else {
        
        if ex_1 != ex_2 {
            if ex_2 < ex_1 {
                // now ex_2 is smaller than ex_1
                let a = arr2[ex_2 - 1];
                let slice = &arr1[ex_2..ex_1];
                for &a1 in slice {
                    sum += reduction(a, a1);
                }
            }else {
                // now ex_1 is smaller than ex_2
                let a = arr1[ex_1 - 1];
                let slice = &arr2[ex_1..ex_2];
                for &a2 in slice {
                    sum += reduction(a, a2);
                }
            }
            
        }
        // at last repeat the last value as long as needed:
        let last_1 = arr1[ex_1 - 1];
        let last_2 = arr2[ex_2 - 1];
        // use difference as weight
        let weight = len - ex_2.max(ex_1);
        sum += reduction(last_1, last_2) * weight as f64;
        sum / len as f64
    }
}