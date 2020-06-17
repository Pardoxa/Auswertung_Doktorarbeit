use structopt::StructOpt;
use std::convert::*;

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
    },
}

#[derive(Clone)]
pub struct ReadOpts{
    pub n: usize,
    pub bins: usize,
    pub files: String,
    pub bin_size: usize,
}

impl From<Opt> for ReadOpts{
    fn from(opt: Opt) -> Self {
        match opt {
            Opt::Read{n, bins, files} => {
                Self{
                    n,
                    bins,
                    files,
                    bin_size: n / bins,
                }
            }
        }
    }
}