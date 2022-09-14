

use structopt::StructOpt;
mod rate_to_pdf;
mod pdf_to_rate;
use std::env;

fn main() {
    let options = Opt::from_args();
    print!("#");
    for a in env::args() {
        print!("{} ", a);
    }
    println!();
    match &options {
        Opt::RateToPdf{..} => {
            rate_to_pdf::rate_to_pdf(options.into())
        },
        Opt::PdfToRateOpt(o) => {
            pdf_to_rate::pdf_to_rate(o)
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
        
    },
    /// should be log10, output is ln as rate function requires
    PdfToRateOpt(PdfToRateOpt)
}

#[derive(StructOpt, Debug, Clone)]
pub struct PdfToRateOpt
{
    /// which rate function to load
    #[structopt(long, short)]
    pub load: String,
    
    /// use this system size
    #[structopt(long, short)]
    pub n: usize,

    #[structopt(long)]
    pub index_left: usize,

    #[structopt(long)]
    pub index_right: usize,

    #[structopt(long)]
    pub index_log: usize
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
            _ => unimplemented!()
        }
    }
}