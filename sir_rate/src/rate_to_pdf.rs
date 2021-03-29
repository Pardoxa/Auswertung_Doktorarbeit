

use std::{fs::File, io::BufRead, ops::Mul};
use std::path::Path;
use std::convert::AsRef;
use std::io::{BufReader, Write};
use crate::RateToPdfOpt;

pub fn rate_to_pdf(opt: RateToPdfOpt)
{
    let mut rate = parse_file(&opt.load);
    calc_pdf(opt, &mut rate);
    print_rate(&rate)
}

#[derive(Debug, Clone)]
pub struct RateEntry
{
    pub left: f64,
    pub right: f64,
    pub val: f64
}

impl RateEntry
{
    pub fn val(&mut self) -> &mut f64
    {
        &mut self.val
    }
}


pub fn parse_entry(slice: &str) -> Option<RateEntry>
{
    let mut it = slice.split(" ");
    let left = it.next()?.parse::<f64>().ok()?;
    
    let right = it.next()?.parse::<f64>().ok()?;

    let val = it.next()?.parse::<f64>().ok()?;

    Some(
        RateEntry{
            left,
            right,
            val
        }
    )
}

pub fn parse_file<F: AsRef<Path>>(file: F) -> Vec<RateEntry>
{
    let f = File::open(file).expect("unable to open file");
    let reader = BufReader::new(f);

    reader.lines()
        .map(|l| l.unwrap())
        .filter(
            |line|
            {
                let t = line.trim_start();
                !t.starts_with("#") // skip comments
                && !t.is_empty()
            }
        ).map(
            |l|
            {
                let t = l.trim_start();
                parse_entry(t).unwrap()
            }
        ).collect()
}

pub fn calc_pdf(opt: RateToPdfOpt, rate: &mut [RateEntry])
{
    let n = opt.n as f64 * -1.0;

    rate.iter_mut()
        .map(RateEntry::val)
        .for_each(
            |v|
            {
                *v = v.mul(n);
                if opt.no_exp
                {
                    *v *= std::f64::consts::LOG10_E;
                }else {
                    *v = v.exp();
                }
            }
        );
}

pub fn print_rate(rate: &[RateEntry])
{
    println!("#left right val");
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    rate.iter()
        .for_each(
            |r|
            {
                write!(&mut out, "{} {} ", r.left, r.right).unwrap();
                dtoa::write(&mut out, r.val).unwrap();
                writeln!(&mut out).unwrap()
            }
        )
}