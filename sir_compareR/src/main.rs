mod parse_files;
mod parse_cmd;
use parse_cmd::*;
mod analyse;
use analyse::*;
mod stats;
mod histogram;
use histogram::*;
use std::io::*;
use std::fs::*;
use net_ensembles::sampling::*;
use either::*;
mod heatmap2;

fn main() {
    let options = get_cmd_opts();
    match options {
        Opt::Heatmap{..} => write_heatmap(options.into()),
        Opt::Histogram{..} => write_histogram(options.into()),
        Opt::Heatmap2{..} => write_heatmap2(options.into())
    };
}


fn write_heatmap(opts: HeatmapOpts)
{

    let sorted_data = parse_files::parse_and_group_all_files(opts.clone());
    let matr =
    match opts.j  {
        0 => {
            eprintln!("0 threds not allowed, use at least 1: INVALID j");
            panic!()
        },
        1 => compare_curves(sorted_data, !opts.no_p_bar, opts.cutoff, opts.mode.into()),
        _ => compare_curves_parallel(sorted_data, opts.j, !opts.no_p_bar, opts.cutoff, opts.mode.into())
         
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

    let heatmap = heatmap2::parse_and_count_all_files(&opts);

    let file = File::create(&filename).unwrap();
    let mut writer = BufWriter::new(file);

    writeln!(writer, "#{}", stats::get_cmd_args()).unwrap();

    let mut settings = GnuplotSettings::new();
    let y_lab = format!("{:?}", opts.fun);
    settings.x_label("E")
        .y_label(y_lab)
        .x_axis(GnuplotAxis::new(0.0, 1.0, 5))
        .pallet(GnuplotPallet::PresetRGB);

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
                filename,
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
                filename,
                settings
            ).unwrap();
        }
    };

    // mirror
    //for inner in 0..inner_len
    //{
    //    for outer in 0..outer_len - 1{
    //        write!(writer, "{} ", heatmap[outer][inner]).unwrap();
    //    }
    //    writeln!(writer, "{}", heatmap[outer_len-1][inner]).unwrap();
    //}
}