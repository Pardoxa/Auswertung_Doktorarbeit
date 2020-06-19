use structopt::StructOpt;
use std::convert::*;
use std::process::exit;
use std::collections::*;
use crate::parse_files::*;

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
        /// number of nodes
        #[structopt(long,short)]
        n: usize,

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

        /// choose compare mode, default: 0
        /// * 0: Abs
        /// * 1: Sqrt
        /// * 2: Cbrt
        /// * 3: Corr
        #[structopt(long, default_value = "0")]
        mode: usize,

    },
}

#[derive(Copy, Clone, Debug)]
pub enum Mode
{
    Abs,
    Sqrt,
    Cbrt,
    Corr
}

impl Mode {
    #[inline]
    pub fn get_fn(&self) -> fn(f64, f64) -> f64
    {
        match self {
            Mode::Abs => mode_abs,
            Mode::Sqrt => mode_sqrt,
            Mode::Cbrt => mode_cbrt,
            Mode::Corr => panic!("INVALID MODE!"),
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

impl From<usize> for Mode{
    fn from(num: usize) -> Self {
        match num {
            0 => Mode::Abs,
            1 => Mode::Sqrt,
            2 => Mode::Cbrt,
            3 => Mode::Corr,
            _ => panic!("invalid mode!"),
        }
    }
}

#[derive(Clone)]
pub struct HeatmapOpts{
    pub n: usize,
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
}

impl HeatmapOpts{
    pub fn generate_filename<D: std::fmt::Display>(&self, extension: D) -> String
    {
        format!(
            "v{}_{:?}_N{}_b{}_e{}_{}.{}.{}", 
            env!("CARGO_PKG_VERSION"),
            self.mode,
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
                    .filter(|&suf| suf != "gz")
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
                    data_mode
                }
            }
        }
    }
}