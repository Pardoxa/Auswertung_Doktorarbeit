use crate::analyse::*;
use crate::*;
use std::io::*;
use std::fs::*;
use std::env;
use rgsl::statistics::correlation;
use std::ops::*;
use average::WeightedMean;
use average::Mean;

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
    pub fn calc_mean<F>(&self, mut i: usize, mut j: usize, mut k: usize, mut l: usize, reduction: F) -> f64
    where F: Fn(f64, f64) -> f64
    {

        let mut ex_i = self.data[i][k].len();
        let mut ex_j = self.data[j][l].len();
        
        // calculate weighted mean where both have values
        let mean: Mean = self.data[i][k].iter()
            .zip(self.data[j][l].iter())
            .map(|(a, b)| reduction(*a, *b))
            .collect();
        if ex_i == ex_j && ex_i == self.get_inside_len()
        {
            mean.mean()
        } else {
            
            let mut w_mean: WeightedMean = WeightedMean::new();
            let m_ex = ex_i.min(ex_j);
            w_mean.add(mean.mean(), m_ex as f64);
            if ex_i != ex_j {

                if ex_j < ex_i {
                    std::mem::swap(&mut ex_j, &mut ex_i);
                    std::mem::swap(&mut i, &mut j);
                    std::mem::swap(&mut k, &mut l);
                }

                // now ex_i is smaller than ex_j
                let a = self.data[i][k][ex_i - 1];
                let mean: Mean = (ex_i..ex_j).map(
                    |index| 
                    {
                        let b = self.data[j][l][index];
                        reduction(a, b)
                    }
                ).collect();
                w_mean.add(mean.mean(), (ex_j - ex_i) as f64);

            }
            // at last repeat the last value as long as needed:
            let i_last = self.data[i][k][ex_i - 1];
            let j_last = self.data[j][l][ex_j - 1];
            // use difference as weight
            let weight = self.inside_len - ex_j;
            w_mean.add(reduction(i_last, j_last), weight as f64);
            w_mean.mean()
        }

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