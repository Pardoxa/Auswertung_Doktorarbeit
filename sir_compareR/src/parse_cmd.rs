use structopt::StructOpt;
use std::ops::Deref;
use std::str::FromStr;
use std::{convert::*, num::NonZeroUsize};
use std::process::exit;
use std::collections::*;
use crate::parse_files::*;
use crate::histogram::*;
use crate::heatmap2::*;
use crate::heatmap_generic::HistBuilder;
use sampling::heatmap::{GnuplotPalette, CubeHelixParameter};

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

        /// max number of bin entries 
        #[structopt(long)]
        max_entries: Option<NonZeroUsize>,

        /// print number of entries in each bin
        #[structopt(long)]
        print_bin_lens: bool,

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

        /// Do not subtract 1 from the energy value
        #[structopt(long)]
        no_subtract: bool,

        /// Use this option when C=0 is allowed
        #[structopt(long)]
        c0: bool
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

        /// automatically call gnuplot to plot the resulting heatmap
        #[structopt(long, short)]
        gnuplot: bool,

        #[structopt(long)]
        /// norm curves before calculation
        normed: bool,

        /// What function to use
        /// valid: 'indexmax' bzw. 'index_max',
        /// 'valmax' bzw. 'val_max',
        /// 'index_min' bzw. 'indexmin',
        /// 'val_min' bzw. 'valmin',
        /// 'lastindexnotzero' bzw. 'last_index_not_zero' bzw. 'last-index-not-zero' bzw. 'last',
        /// 'x_to_y' where x and y are f64
        #[structopt(long)]
        fun: FunctionChooser,

        /// Use rgb color palett
        #[structopt(long)]
        rgb: bool,

        /// Use this option when C=0 is allowed
        #[structopt(long)]
        c0: bool
        
    },
    GenericHeatmap
    {
        /// For requesting the heatmap
        /// e.g. 'f 100 0 100' for float, 100 bins, left 0, right 100
        /// e.g. 'i 100 0 100' for isize, 100 bins, left 0, right 100
        #[structopt(long)]
        hist_x: HistBuilder,

        /// For requesting the heatmap
        /// e.g. 'f 100 0 100' for float, 100 bins, left 0, right 100
        /// e.g. 'i 100 0 100' for isize, 100 bins, left 0, right 100
        #[structopt(long)]
        hist_y: HistBuilder,

        /// Index of the x value. Default 0
        #[structopt(long,short, default_value="0")]
        x_index: usize,

        /// Index of the y value. Default 1
        #[structopt(long,short, default_value="1")]
        y_index: usize,

        /// filenames (globbing pattern)
        #[structopt(long, short)]
        files: String,

        /// use every "every"th data point
        #[structopt(long, short, default_value = "1")]
        every: NonZeroUsize,

        /// X label for heatmap
        #[structopt(long)]
        x_label: Option<String>,

        /// Y label for heatmap
        #[structopt(long)]
        y_label: Option<String>,

        /// Do not normalize heatmap
        #[structopt(long, short)]
        non_normalized: bool,

        /// How the resulting gnuplot file will be called
        #[structopt(long)]
        name: String,

        /// How the file generated by the gnuplot file will be called
        #[structopt(long)]
        gnuplot_output_name: Option<String>,

        /// automatically call gnuplot to plot the resulting heatmap
        #[structopt(long, short)]
        gnuplot: bool,

        /// Do not output hist errors
        #[structopt(long, short)]
        supress_hist_error: bool,

        /// Which palett to use. "r" for rgb, "h" for hsv and "c" for cubehelix. Use "c,r" for cubehelix reversed
        #[structopt(long, short, default_value="r")]
        palett: GnuPalett
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
        /// valid: 'indexmax' bzw. 'index_max',
        /// 'valmax' bzw. 'val_max',
        /// 'index_min' bzw. 'indexmin',
        /// 'val_min' bzw. 'valmin',
        /// 'lastindexnotzero' bzw. 'last_index_not_zero' bzw. 'last-index-not-zero' bzw. 'last',
        /// 'x_to_y' where x and y are f64
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
            "v{}_{}_N{}_b{}_e{}_p{}.{}.{}", 
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
    pub gnuplot_exec: bool,
    pub rgb: bool,
    pub c0: bool
}

impl Heatmap2Opts{
    pub fn generate_filename<D: std::fmt::Display>(&self, extension: D) -> String
    {
        format!(
            "v{}_{}_N{}_b{}_{}_e{}_{}.{}.{}", 
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
                gnuplot,
                rgb,
                c0
            } => {
                let len = if c0 {
                    n + 1
                } else {
                    n
                };
                if len % bins != 0 {
                    eprintln!("ERROR: {} does nt divide by {} - rest is {}", len, bins, len % bins);
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
                    normed,
                    gnuplot_exec: gnuplot,
                    rgb,
                    c0
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
    pub bin_count: usize,
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
    pub norm: bool,
    pub no_subtract: bool,
    pub max_entries: Option<NonZeroUsize>,
    pub print_bin_lens: bool,
    pub c0: bool
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
            self.bin_count,
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
                s.rsplit('.')
                    .find(|suf| !COMPRESSION_SUFFIX.contains(suf))
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
                no_subtract,
                max_entries,
                print_bin_lens,
                c0
            } => {
                let len = if c0 {
                    n + 1
                } else {
                    n
                };
                if len % bins != 0 {
                    eprintln!("ERROR: {} does nt divide by {} - rest is {}", len, bins, len % bins);
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
                    bin_count: bins,
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
                    no_subtract,
                    max_entries,
                    print_bin_lens,
                    c0
                }
            },
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone)]
pub struct GnuPalett{
    palett: GnuplotPalette
}

impl GnuPalett{
    pub fn into_inner(self) -> GnuplotPalette{
        self.palett
    }
}

impl Default for GnuPalett{
    fn default() -> Self {
        GnuPalett{palett: GnuplotPalette::PresetRGB}
    }
}

impl Deref for GnuPalett{
    type Target = GnuplotPalette;
    fn deref(&self) -> &Self::Target {
        &self.palett
    }
}

impl FromStr for GnuPalett
{
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split(',');
        let first = match iter.next(){
            Some(val) => val,
            None => return Err("No option given")
        };

        let lower = first.to_lowercase();

        return match lower.as_str() {
            "r" | "rgb" => {
                Ok(
                    GnuPalett{palett: GnuplotPalette::PresetRGB}
                )
            },
            "h" | "hsv" => {
                Ok(
                    GnuPalett{palett: GnuplotPalette::PresetHSV}
                )
            },
            "c" | "cubehelix" => {
                let mut params = CubeHelixParameter::default();
                for val in iter {
                    let l = val.to_lowercase();
                    if l.starts_with('r') || l.starts_with("reverse"){
                        params.reverse(true);
                    }
                }
                Ok(
                    GnuPalett{palett: GnuplotPalette::CubeHelix(params)}
                )
            },
            _ => {
                Err("Option unknown. Use 'r' for rgb, 'h' for hsv and 'c' for cubehelix. You can also use 'c,r' for cubehelix reversed")
            }
        }
    }
}