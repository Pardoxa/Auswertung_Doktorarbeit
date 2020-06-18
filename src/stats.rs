use crate::analyse::*;
use crate::*;
use std::io::*;
use std::fs::*;
use std::env;
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