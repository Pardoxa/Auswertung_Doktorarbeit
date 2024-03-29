mod parse_files;
mod parse_cmd;
use parse_cmd::*;
mod analyse;
use analyse::*;
mod stats;
mod histogram;
use histogram::*;
use std::{convert::TryInto, io::*};
use std::fs::*;
use sampling::*;
use either::*;
mod heatmap2;
mod hist_analyser;
use std::process::Command;
use colored::*;
mod heatmap_generic;
use heatmap_generic::*;
use std::sync::atomic::*;

fn main() {
    let options = get_cmd_opts();
    match options {
        Opt::Heatmap{..} => write_heatmap(options.into()),
        Opt::Histogram{..} => write_histogram(options.into()),
        Opt::Heatmap2{..} => write_heatmap2(options.into()),
        Opt::Percent{..} => write_percent(options.into()),
        Opt::GenericHeatmap{..} => generate_heatmap(options.try_into().unwrap())
    };

    if heatmap2::WARNING_PRINTED.load(std::sync::atomic::Ordering::Relaxed)
    {
        eprintln!("{}", "Are you sure you wanted to norm the curves? This is probably a mistake!".red().bold())
    }
}


fn write_heatmap(opts: HeatmapOpts)
{

    let mut sorted_data = parse_files::parse_and_group_all_files(opts.clone());
    if crate::parse_files::UNFINISHED_ENCOUNTERED.load(Ordering::Relaxed)
    {
        println!("UNFINISHED COUNT: {}", crate::parse_files::UNFINISHED_COUNTER.load(Ordering::Relaxed));
    }
    println!("average bin entries: {}", sorted_data.average_entries());
    let num = if sorted_data.data().len() < 5 {
        sorted_data.data().len()
    } else {
        5
    };
    println!("max {} entries: {:?}", num, sorted_data.max_n_entries(num));
    println!("min {} entries: {:?}", num, sorted_data.min_n_entries(num));
    if opts.print_bin_lens 
    {
        sorted_data.print_lens();
    }
    if let Some(maximum) = opts.max_entries {
        sorted_data.limit_entries(maximum);
        println!("new max {} entries: {:?}", num, sorted_data.max_n_entries(num));
        println!("new average bin entries: {}", sorted_data.average_entries());
        if opts.print_bin_lens 
        {
            println!("After limiting:");
            sorted_data.print_lens();
        }
    }

    let matr =
    match opts.j  {
        0 => {
            eprintln!("0 threds not allowed, use at least 1: INVALID j");
            panic!()
        },
        1 => compare_curves(sorted_data, !opts.no_p_bar, opts.cutoff, opts.mode),
        _ => compare_curves_parallel(sorted_data, opts.j, !opts.no_p_bar, opts.cutoff, opts.mode)
         
    };
    write_matr(matr, opts);
}

fn write_histogram(opts: HistogramOpts)
{
    let hist_data = histogram::parse_and_group_all_files(opts.clone());
    let hist = histogramm_parallel(hist_data, opts.j, !opts.no_p_bar);
    
    let filename = opts.generate_filename(".dat");
    let file = File::create(filename).unwrap();
    let mut writer = BufWriter::new(file);

    writeln!(writer, "#{}", stats::get_cmd_args()).unwrap();
    for (index, (mean, error)) in hist.into_iter().enumerate()
    {
        writeln!(writer, "{} {:e} {:e}", index, mean, error).unwrap();
    }
}

fn write_heatmap2(opts: Heatmap2Opts)
{
    let filename = opts.generate_filename("h2.gp");
    println!("creating: {}", &filename);

    let (files, heatmap) = heatmap2::parse_and_count_all_files(&opts);

    let file = File::create(&filename).unwrap();
    let mut writer = BufWriter::new(file);

    writeln!(writer, "#{}", stats::get_cmd_args()).unwrap();
    files.into_iter()
        .map(canonicalize)
        .for_each(|p|
            {
                writeln!(writer, "#{}", p.unwrap().display()).unwrap();
            }
        );

    let mut settings = GnuplotSettings::new();
    let y_lab = format!("{}", opts.fun);
    let palette = if opts.rgb {
        GnuplotPalette::PresetRGB
    } else {
        let mut palette = CubeHelixParameter::default();
        palette.gamma(0.65);
        palette.into()
    };
    settings.x_label("E")
        .y_label(y_lab)
        .x_axis(GnuplotAxis::new(0.0, 1.0, 5))
        .palette(palette);

    match heatmap {
        Left(heat) => {
            println!("TOTAL: {}", heat.total());
            println!("OUTSIDE: {}", heat.total_misses());
            let frac = heat.total_misses() as f64/ heat.total() as f64;
            println!("FRAC: {}", frac);
            let heat = heat.into_heatmap_normalized_columns();

            let min_val = *heat.height_hist().borders().first().unwrap();
            let max_val = *heat.height_hist()
                .borders()
                .last()
                .unwrap();
            settings.y_axis(GnuplotAxis::new(min_val, max_val, 5));

            heat.gnuplot(
                writer,
                &filename,
                settings
            ).unwrap();
        },
        Right(heat) => {
            println!("TOTAL: {}", heat.total());
            println!("OUTSIDE: {}", heat.total_misses());
            let frac = heat.total_misses() as f64/ heat.total() as f64;
            println!("FRAC: {}", frac);
            let heat = heat.into_heatmap_normalized_columns();

            let min_val = *heat.height_hist().borders().first().unwrap() as f64;
            let max_val = *heat.height_hist()
                .borders()
                .last()
                .unwrap() - 1;
            let max_val = max_val as f64;

            settings.y_axis(GnuplotAxis::new(min_val, max_val, 5));

            heat.gnuplot(
                writer,
                &filename,
                settings
            ).unwrap();
        }
    };
    if opts.gnuplot_exec {
        match Command::new("gnuplot")
            .arg(filename)
            .output()
        {
            Ok(_) => {},
            Err(error) => {
                eprintln!("{}", error)
            }
        }
    }
}

fn write_percent(opts: PercentOpts){
    let mut hist_percent = hist_analyser::parse_and_count_all_files(&opts);

    let res = hist_percent.percent(opts.percent);
    let name = opts.generate_filename("percent");
    let file = File::create(&name)
        .unwrap();
    let mut buf = BufWriter::new(file);

    writeln!(buf, "#{}", stats::get_cmd_args()).unwrap();
    writeln!(buf, "#percent: {}", opts.percent).unwrap();
    
    let max = res.iter()
        .max_by_key(|&v| v.time)
        .unwrap();
    
    writeln!(
        buf,
        "#max_val: {} max_val_bin_left: {} max_val_bin_right: {} count: {}",
        max.time,
        max.left,
        max.right,
        max.count
    ).unwrap();

    writeln!(buf, "#left right val/time count").unwrap();

    for item in res {
        writeln!(buf, "{} {} {} {}", item.left, item.right, item.time, item.count).unwrap();
    }
    println!("generated: {}", &name);
    println!("easy plotting:");
    println!("p \"{}\" u 1:3", &name);
    println!("p \"{}\" u 1:4", name);
}