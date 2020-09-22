use structopt::StructOpt;
use std::convert::*;
use std::process::exit;
use std::collections::*;
use crate::parse_files::*;
use crate::histogram::*;
use crate::heatmap2::*;

const COMPRESSION_SUFFIX: [&str; 2]= ["gz", "xz"];

pub fn get_cmd_opts() -> Opt
{
    Opt::from_args()
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Execute the sir models!")]
pub enum Opt
{
    /// TODO description
    Heatmap {
        /// number of nodes that are reachable (minus 1)
        #[structopt(long,short)]
        n: usize,

        /// actual number of nodes
        #[structopt(long)]
        n_real: Option<usize>,

        /// number of samples
        #[structopt(long, short)]
        bins: usize,

        /// number of samples
        #[structopt(long, short)]
        files: String,

        /// save file to create
        #[structopt(long, default_value= "")]
        save: String,

        #[structopt(short)]
        /// number of threads to use
        j: usize,

        #[structopt(long)]
        /// hide progress bar
        no_p_bar: bool,

        #[structopt(long, short)]
        /// use every nth step
        every: usize,

        #[structopt(long, default_value = "2")]
        /// min number of curves to be used in calculation
        cutoff: usize,

        #[structopt(long)]
        /// do not norm curves
        no_norm: bool,

        /// choose compare mode, default: 0
        /// * 0: Abs
        /// * 1: Sqrt
        /// * 2: Cbrt
        /// * 3: Corr
        #[structopt(long, default_value = "0")]
        mode: usize,
    },
    Histogram {
        /// number of nodes
        #[structopt(long,short)]
        n: usize,

        /// number of samples
        #[structopt(long, short)]
        bins: usize,

        /// filenames
        #[structopt(long, short)]
        files: String,

        /// save file to create
        #[structopt(long, default_value= "")]
        save: String,

        #[structopt(short)]
        /// number of threads to use
        j: usize,

        #[structopt(long)]
        /// hide progress bar
        no_p_bar: bool,

        #[structopt(long, short)]
        /// use every nth step
        every: usize,

        /// What function to use
        /// valid: indexmax, valmax
        #[structopt(long)]
        hist_reduce: HistReduce
    },
    Heatmap2 {
        /// number of nodes
        #[structopt(long,short)]
        n: usize,

        /// number of bins for energy
        #[structopt(long)]
        bins: usize,

        /// For requesting the heatmap
        /// e.g. 'f 100 0 100' for float, 100 bins, left 0, right 100
        /// e.g. 'u 100 0 100' for usize, 100 bins, left 0, right 100
        #[structopt(long)]
        heatmap: HeatmapBuilder,

        /// filenames
        #[structopt(long, short)]
        files: String,

        /// save file to create
        #[structopt(long, default_value= "")]
        save: String,

        #[structopt(long)]
        /// hide progress bar
        no_p_bar: bool,

        #[structopt(long, short)]
        /// use every nth step
        every: usize,

        #[structopt(long)]
        /// norm curves before calculation
        normed: bool,

        /// What function to use
        /// valid: indexmax, valmax
        #[structopt(long)]
        fun: FunctionChooser,
        
    },
    Percent {
        /// number of nodes
        #[structopt(long,short)]
        n: usize,

        /// number of bins for energy
        #[structopt(long)]
        bins: usize,

        /// filenames
        #[structopt(long, short)]
        files: String,

        /// What function to use
        /// valid: indexmax, valmax
        #[structopt(long)]
        fun: FunctionChooser,

        #[structopt(long, short)]
        /// use every nth step
        every: usize,

        #[structopt(long, short)]
        /// Percent val
        percent: f64,
    }
}

#[derive(Debug, Clone)]
pub struct PercentOpts{
    pub n: usize,
    pub bins: usize,
    pub files: String,
    pub fun: FunctionChooser,
    pub every: usize,
    pub percent: f64,
    pub suffix: String
}

impl PercentOpts{
    pub fn generate_filename<D: std::fmt::Display>(&self, extension: D) -> String
    {
        format!(
            "v{}_{:?}_N{}_b{}_e{}_p{}.{}.{}", 
            env!("CARGO_PKG_VERSION"),
            self.fun,
            self.n,
            self.bins,
            self.every,
            self.percent,
            &self.suffix,
            extension
        )
    }
}

impl From<Opt> for PercentOpts{
    fn from(opt: Opt) -> Self {
        match opt {
            Opt::Percent {
                n,
                files,
                every,
                fun,
                bins,
                percent,
            } => {
                let suffix = match get_suffix(&files){
                    Ok(suf) => suf,
                    Err(set) => {
                        eprintln!("WARNING: Sufix do not match! Found {:?}", set);
                        set.into_iter()
                            .collect::<Vec<String>>()
                            .join("_")
                    }
                };
                PercentOpts{
                    n,
                    files,
                    fun,
                    every,
                    bins,
                    percent,
                    suffix
                }
            },
            _ => unreachable!()
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Mode
{
    Abs,
    Sqrt,
    Cbrt,
    Corr,
    IndexMaxAbs,
    MaxValAbs
}


impl From<usize> for Mode{
    fn from(num: usize) -> Self {
        match num {
            0 => Mode::Abs,
            1 => Mode::Sqrt,
            2 => Mode::Cbrt,
            3 => Mode::Corr,
            4 => Mode::IndexMaxAbs,
            5 => Mode::MaxValAbs,
            _ => panic!("invalid mode!"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Heatmap2Opts
{
    pub n: usize,
    pub bins: usize,
    pub files: String,
    pub save: String,
    pub no_p_bar: bool,
    pub every: usize,
    pub suffix: String,
    pub fun: FunctionChooser,
    pub normed: bool,
    pub heatmap_builder: HeatmapBuilder,
}

impl Heatmap2Opts{
    pub fn generate_filename<D: std::fmt::Display>(&self, extension: D) -> String
    {
        format!(
            "v{}_{:?}_N{}_b{}_{}_e{}_{}.{}.{}", 
            env!("CARGO_PKG_VERSION"),
            self.fun,
            self.n,
            self.bins,
            self.heatmap_builder,
            self.every,
            self.save,
            &self.suffix,
            extension
        )
    }
}


impl From<Opt> for Heatmap2Opts{
    fn from(opt: Opt) -> Self {
        match opt {
            Opt::Heatmap2 {
                n,
                files,
                save,
                no_p_bar,
                every,
                fun,
                normed,
                heatmap,
                bins,
            } => {
                if n % bins != 0 {
                    eprintln!("ERROR: {} does nt divide by {} - rest is {}", n, bins, n % bins);
                    exit(-1);
                }
                let suffix = match get_suffix(&files){
                    Ok(suf) => suf,
                    Err(set) => {
                        eprintln!("WARNING: Sufix do not match! Found {:?}", set);
                        set.into_iter()
                            .collect::<Vec<String>>()
                            .join("_")
                    }
                };
                
                Heatmap2Opts{
                    n,
                    bins,
                    heatmap_builder: heatmap,
                    files,
                    save,
                    no_p_bar,
                    every,
                    fun,
                    suffix,
                    normed
                }
            },
            _ => unreachable!()
        }
    }
}

#[derive(Clone, Debug)]
pub struct HistogramOpts{
    pub n: usize,
    pub bins: usize,
    pub files: String,
    pub bin_size: usize,
    pub save: String,
    pub j: usize,
    pub no_p_bar: bool,
    pub every: usize,
    pub suffix: String,
    pub hist_reduce: HistReduce,
}

impl HistogramOpts{
    pub fn generate_filename<D: std::fmt::Display>(&self, extension: D) -> String
    {
        format!(
            "v{}_{:?}_N{}_b{}_e{}_{}.{}.{}", 
            env!("CARGO_PKG_VERSION"),
            self.hist_reduce,
            self.n,
            self.bins,
            self.every,
            self.save,
            &self.suffix,
            extension
        )
    }
}

impl From<Opt> for HistogramOpts{
    fn from(opt: Opt) -> Self {
        match opt {
            Opt::Histogram {
                n, 
                bins,
                files,
                save,
                j,
                no_p_bar,
                every,
                hist_reduce
            } => {
                if n % bins != 0 {
                    eprintln!("ERROR: {} does nt divide by {} - rest is {}", n, bins, n % bins);
                    exit(-1);
                }
                let suffix = match get_suffix(&files){
                    Ok(suf) => suf,
                    Err(set) => {
                        eprintln!("WARNING: Sufix do not match! Found {:?}", set);
                        set.into_iter()
                            .collect::<Vec<String>>()
                            .join("_")
                    }
                };
                
                HistogramOpts{
                    n,
                    bins,
                    files,
                    bin_size: n / bins,
                    save,
                    j,
                    no_p_bar,
                    every,
                    hist_reduce,
                    suffix
                }
            },
            _ => unreachable!()
        }
    }
}

#[derive(Clone)]
pub struct HeatmapOpts{
    pub n: usize,
    pub n_real: Option<usize>,
    pub bins: usize,
    pub files: String,
    pub bin_size: usize,
    pub save: String,
    pub j: usize,
    pub no_p_bar: bool,
    pub every: usize,
    pub cutoff: usize,
    pub mode: Mode,
    pub suffix: String,
    pub data_mode: DataMode,
    pub norm: bool
}

impl HeatmapOpts{
    pub fn generate_filename<D: std::fmt::Display>(&self, extension: D) -> String
    {
        let norm = if self.norm {
            "norm"
        } else {
            "NoNorm"
        };
        let n_actual = if let Some(val) = self.n_real {
            val
        } else {
            self.n
        };
        format!(
            "v{}_{:?}_{}_N{}_Reach{}_b{}_e{}_{}.{}.{}", 
            env!("CARGO_PKG_VERSION"),
            self.mode,
            norm,
            n_actual,
            self.n,
            self.bins,
            self.every,
            self.save,
            &self.suffix,
            extension
        )
    }
}

pub fn get_suffix<S: AsRef<str>>(pattern: S) -> Result<String, HashSet<String>>
{
    let list: HashSet<_> = glob::glob(pattern.as_ref())
        .unwrap()
        .filter_map(Result::ok)
        .map(
            |item| 
            {
                let s = item.into_os_string()
                        .into_string()
                        .unwrap();
                s.rsplit(".")
                    .filter(|suf| !COMPRESSION_SUFFIX.contains(suf))
                    .next()
                    .unwrap()
                    .to_owned()
                    
            }
        )
        .collect();
    if list.len() == 1 {
        Ok(
            list.into_iter()
                .next()
                .unwrap()
        )
    }else{
        Err(list)
    }
}

impl From<Opt> for HeatmapOpts{
    fn from(opt: Opt) -> Self {
        match opt {
            Opt::Heatmap{
                n, 
                bins,
                files,
                save,
                j,
                no_p_bar,
                every,
                cutoff,
                mode,
                no_norm,
                n_real,
            } => {
                if n % bins != 0 {
                    eprintln!("ERROR: {} does nt divide by {} - rest is {}", n, bins, n % bins);
                    exit(-1);
                }
                let suffix = match get_suffix(&files){
                    Ok(suf) => suf,
                    Err(set) => {
                        eprintln!("WARNING: Sufix do not match! Found {:?}", set);
                        set.into_iter()
                            .collect::<Vec<String>>()
                            .join("_")
                    }
                };
                let data_mode = match mode{
                    0..=2 => DataMode::Sparse,
                    _ => DataMode::Naive,
                };
                Self{
                    n,
                    bins,
                    files,
                    bin_size: n / bins,
                    save,
                    j,
                    no_p_bar,
                    every,
                    cutoff,
                    mode: mode.into(),
                    suffix,
                    data_mode,
                    norm: !no_norm,
                    n_real,
                }
            },
            _ => unreachable!()
        }
    }
}