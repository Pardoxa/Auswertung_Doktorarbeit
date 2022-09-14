

use std::{fs::File, io::BufRead};
use std::path::Path;
use std::convert::AsRef;
use std::io::{BufReader, Write};
use crate::PdfToRateOpt;

pub fn pdf_to_rate(opt: &PdfToRateOpt)
{
    let mut rate = parse_file(&opt.load, opt.index_left, opt.index_right, opt.index_log);
    calc_rate(opt, &mut rate);
    print_rate(&rate)
}

#[derive(Debug, Clone)]
pub struct RateEntry
{
    pub left: f64,
    pub right: f64,
    pub val: f64
}



pub fn parse_entry(slice: &str, index_left: usize, index_right: usize, index_log: usize) -> Option<RateEntry>
{
    let mut it = slice.split_whitespace();
    let left = it.nth(index_left)?.parse::<f64>().ok()?;
    let mut it = slice.split_whitespace();
    let right = it.nth(index_right)?.parse::<f64>().ok()?;
    let mut it = slice.split_whitespace();
    let val = it.nth(index_log)?.parse::<f64>().ok()?;

    Some(
        RateEntry{
            left,
            right,
            val
        }
    )
}

pub fn parse_file<F: AsRef<Path>>(file: F, index_left: usize, index_right: usize, index_log: usize) -> Vec<RateEntry>
{
    let f = File::open(file).expect("unable to open file");
    let reader = BufReader::new(f);

    reader.lines()
        .map(|l| l.unwrap())
        .filter(
            |line|
            {
                let t = line.trim_start();
                !t.starts_with('#') // skip comments
                && !t.is_empty()
            }
        ).map(
            |l|
            {
                let t = l.trim_start();
                parse_entry(t, index_left, index_right, index_log).unwrap()
            }
        ).collect()
}

pub fn calc_rate(opt: &PdfToRateOpt, rate: &mut [RateEntry])
{
    let n = opt.n as f64;

    let mut min = f64::INFINITY;
    // switch base to e
    let mut left = 0.0;
    rate.iter_mut()
        .for_each(
            |v|
            {
                v.val /= std::f64::consts::LOG10_E;
                v.val = -v.val/n;
                if v.val < min{
                    min = v.val;
                    left = v.left;
                }
            }
        );
    
    rate.iter_mut()
        .for_each(
            |v| v.val -= min
        );
    eprintln!("left: {left}")
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