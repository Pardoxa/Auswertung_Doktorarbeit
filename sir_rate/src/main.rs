

use structopt::StructOpt;
mod rate_to_pdf;
use std::env;

fn main() {
    let options = Opt::from_args();
    print!("#");
    for a in env::args() {
        print!("{} ", a);
    }
    println!();
    match &options {
        _ => {
            rate_to_pdf::rate_to_pdf(options.into())
        }
    }
}


#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "Stuff with rate functions")]
pub enum Opt
{
    /// Calculate the pdf from a rate function
    RateToPdf{
        /// which rate function to load
        #[structopt(long, short)]
        load: String,

        /// use this system size
        #[structopt(long, short)]
        n: usize,

        /// logarithmic result
        #[structopt(long)]
        no_exp: bool,
    }
}

pub struct RateToPdfOpt
{
    pub load: String,
    pub n: usize,
    pub no_exp: bool,
}

impl From<Opt> for RateToPdfOpt{
    fn from(opt: Opt) -> Self {
        match opt {
            Opt::RateToPdf {
                load,
                n,
                no_exp,
            } => {
                
                Self{
                    n,
                    load,
                    no_exp,
                }
            },
        }
    }
}