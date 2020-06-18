use structopt::StructOpt;
use std::convert::*;
use std::process::exit;

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
        #[structopt(long)]
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
}

impl Mode {
    #[inline]
    pub fn get_fn(&self) -> fn(f64, f64) -> f64
    {
        match self {
            Mode::Abs => mode_abs,
            Mode::Sqrt => mode_sqrt,
            Mode::Cbrt => mode_cbrt,
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
}

impl HeatmapOpts{
    pub fn generate_filename<D: std::fmt::Display>(&self, suffix: D) -> String
    {
        format!("v{}_{:?}_N{}_b{}_e{}_{}.{}", env!("CARGO_PKG_VERSION"), self.mode, self.n, self.bins, self.every, self.save, suffix)
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
                }
            }
        }
    }
}