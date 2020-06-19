use crate::analyse::*;
use crate::*;
use std::io::*;
use std::fs::*;
use std::env;
use rgsl::statistics::correlation;
use std::ops::*;

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

pub struct StatsWriter<W>
{
    mean_writer: W,
    iteration_count_writer: W,
    curve_count_writer: W,
}

impl<W: Write> StatsWriter<W>{

    fn write_mean(&mut self, mean: &Vec<Vec<f64>>)
    {
        for v in mean.iter(){
            let len = v.len();
            for i in 0..len - 1{
                write!(self.mean_writer, "{:e} ", v[i]).unwrap();
            }
            writeln!(self.mean_writer, "{:e}", v.last().unwrap()).unwrap();
        }
    }

    fn write_iteration_count(&mut self, iteration_count: &Vec<Vec<usize>>)
    {
        for v in iteration_count.iter(){
            let len = v.len();
            for i in 0..len - 1{
                write!(self.iteration_count_writer, "{} ", v[i]).unwrap();
            }
            writeln!(self.iteration_count_writer, "{}", v.last().unwrap()).unwrap();
        }
    }

    fn write_curve_count(&mut self, curve_count: &Vec<usize>)
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

impl StatsWriter<File>{
    pub fn new_from_heatmap_opts(opts: HeatmapOpts) -> StatsWriter<BufWriter<File>>
    {
        let mean_name = opts.generate_filename("stats.mean");
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

        let mut stats: StatsWriter<BufWriter<File>> = stats.into();
        writeln!(stats, "#{}", get_cmd_args()).unwrap();
        writeln!(stats, "#{}", env::current_dir().unwrap().to_str().unwrap()).unwrap();
        stats
    }
}

impl<W: Write> Write for StatsWriter<W>{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.mean_writer.write(buf)?;
        self.iteration_count_writer.write(buf)?;
        self.curve_count_writer.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.mean_writer.flush()?;
        self.iteration_count_writer.flush()?;
        self.curve_count_writer.flush()
    }
}

impl<W: Write> From<StatsWriter<W>> for StatsWriter<BufWriter<W>>{
    fn from(origin: StatsWriter<W>) -> Self {
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

    pub fn get_inside_len(&self) -> usize
    {
        self.inside_len
    }

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

    pub fn get_len_at_index(&self, index: usize) -> usize {
        self.data[index].len()
    }


    /// calculates mean of (itemwise) reduction of two curves 
    /// cuve1: data[i][k]
    /// curve2: data[j][l] 
    pub fn calc_mean<F>(&self, i: usize, j: usize, k: usize, l: usize, reduction: F) -> f64
    where F: Fn(f64, f64) -> f64
    {
        reduce(&self.data[i][k], &self.data[j][l], self.get_inside_len(), reduction)
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
}

/// calculates mean of (itemwise) reduction of two curves 
/// cuve1: data[i][k]
/// curve2: data[j][l] 
pub fn reduce<F>(arr1: &[f64], arr2: &[f64], len: usize, reduction: F) -> f64
where F: Fn(f64, f64) -> f64
{
    let ex_1 = arr1.len();
    let ex_2 = arr2.len();
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
                for i in ex_2..ex_1 {
                    sum += reduction(a, arr1[i]);
                }
            }else {
                // now ex_1 is smaller than ex_2
                let a = arr1[ex_1 - 1];
                for i in ex_1..ex_2 {
                    sum += reduction(a, arr2[i]);
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