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
    },
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
}

impl HeatmapOpts{
    pub fn generate_filename<D: std::fmt::Display>(&self, suffix: D) -> String
    {
        format!("v{}N{}_b{}_e{}_{}.{}", env!("CARGO_PKG_VERSION"), self.n, self.bins, self.every, self.save, suffix)
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
                }
            }
        }
    }
}