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
    Read {
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
        save: String
    },
}

#[derive(Clone)]
pub struct ReadOpts{
    pub n: usize,
    pub bins: usize,
    pub files: String,
    pub bin_size: usize,
    pub save: String,
}

impl ReadOpts{
    pub fn generate_filename(&self) -> String
    {
        format!("v{}N{}_b{}_{}.dat", env!("CARGO_PKG_VERSION"), self.n, self.bins, self.save)
    }
}

impl From<Opt> for ReadOpts{
    fn from(opt: Opt) -> Self {
        match opt {
            Opt::Read{n, bins, files, save} => {
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
                }
            }
        }
    }
}