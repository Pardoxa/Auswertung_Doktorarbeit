use crate::heatmap_generic::*;
use sampling::*;
use glob;
use std::{fs::*, io::{BufRead, BufReader, BufWriter, Read}, str::FromStr};
use lzma::LzmaReader;
use flate2::read::*;
use num_traits::AsPrimitive;

pub fn generate_heatmap(opts: HeatmapGenericOpts)
{
    let hist_x = opts.hist_x.build()
        .expect("Error during histogram X build!");
    let hist_y = opts.hist_y.build()
        .expect("Error during histogram Y build!");

    match hist_x {
        HistWrapper::Isize{hist} => {
            let hist_x = hist;
            match hist_y{
                HistWrapper::Isize{hist} => {
                    let hist_y = hist;
                    work(opts, hist_x, hist_y)
                },
                HistWrapper::F64 {hist} => {
                    let hist_y = hist;
                    work(opts, hist_x, hist_y)
                }
            }
        },
        HistWrapper::F64{hist} => {
            let hist_x = hist;
            match hist_y {
                HistWrapper::Isize{hist} => {
                    let hist_y = hist;
                    work(opts, hist_x, hist_y)
                },
                HistWrapper::F64 {hist} => {
                    let hist_y = hist;
                    work(opts, hist_x, hist_y)
                }
            }
        }
    }
}

pub fn work<X, Y, HX, HY>(
    opts: HeatmapGenericOpts,
    hist_x: HX,
    hist_y: HY
)
    where HX: Histogram + HistogramVal<X>,
    HY: Histogram + HistogramVal<Y>,
    X: FromStr + AsPrimitive<f64>,
    Y: FromStr + AsPrimitive<f64>
{
    let borders = hist_x.borders_clone().unwrap();
    let x_min = borders.first().unwrap().as_();
    let x_max = borders.last().unwrap().as_();
    let borders = hist_y.borders_clone().unwrap();
    let y_min = borders.first().unwrap().as_();
    let y_max = borders.last().unwrap().as_();
    let mut heatmap = HeatmapU::<HX, HY>::new(hist_x, hist_y);
    
    glob::glob(&opts.files)
        .unwrap()
        .filter_map(Result::ok)
        .for_each(
            |p| 
            {
                let file = File::open(p.as_path())
                    .expect("cannot open file");

                let ending = p.extension()
                    .unwrap()
                    .to_str()
                    .unwrap();
                match ending {
                    "xz" => {
                        let decoder = LzmaReader::new_decompressor(file).unwrap();
                        count_into_heatmap(decoder, &mut heatmap, opts.clone())
                    },
                    "gz" => {
                        let decoder = GzDecoder::new(file);
                        count_into_heatmap(decoder, &mut heatmap, opts.clone())
                    }
                    _ => {
                        count_into_heatmap(file, &mut heatmap, opts.clone())
                    }
                }
            }
        );
    let mut settings = GnuplotSettings::new();
    if let Some(x_label) = opts.x_label
    {
        settings.x_label(x_label);
    }
    if let Some(y_label) = opts.y_label
    {
        settings.y_label(y_label);
    }

    
    settings
        .x_axis(GnuplotAxis::new(x_min, x_max, 5))
        .pallet(GnuplotPallet::PresetHSV)
        .y_axis(GnuplotAxis::new(y_min, y_max, 5));
    
    println!("creating {}", &opts.gnuplot_name);
    let file = File::create(opts.gnuplot_name).unwrap();
    let writer = BufWriter::new(file);
    println!("Using gnuplot will generate: {}", &opts.gnuplot_output_name);
    let total = heatmap.total();
    let misses = heatmap.total_misses();
    let frac = misses as f64 / total as f64;

    if opts.non_normalized
    {
        heatmap.gnuplot(writer, opts.gnuplot_output_name, settings)
            .unwrap();
    } else {
        heatmap.into_heatmap_normalized_columns()
            .gnuplot(writer, opts.gnuplot_output_name, settings)
            .unwrap();
    }
    
    
    println!("fraction of misses, i.e., outside heatmap: {}", frac);
    println!("Total: {}", total);

}

pub fn count_into_heatmap<X, Y, Hx, Hy, R>(
        reader: R,
        heatmap: &mut HeatmapU<Hx, Hy>,
        opts: HeatmapGenericOpts
    )
where R: Read,
    X: FromStr,
    Y: FromStr,
    Hx: HistogramVal<X>,
    Hy: HistogramVal<Y>
{
    let buf_r = BufReader::new(reader);
    let index_x = opts.x_index;
    let index_y = opts.y_index;
    let smaller_index = index_x.min(index_y);
    let bigger_index = index_x.max(index_y);
    let dif = bigger_index - smaller_index - 1;
    buf_r.lines()
        .map(|v| v.unwrap())
        .filter(
            |line|
            {
                !line.trim_start().starts_with("#") // skip comments
                && !line.is_empty()
            }
        ).step_by(opts.every.get())
        .for_each(
            |line|
            {
                let slice = line.trim_start();
                let mut it = slice.split(" ");

                let smaller = it.nth(smaller_index).unwrap();
                let bigger = it.nth(dif).unwrap();
                let (val_x, val_y) = if index_x < index_y {
                    (
                        smaller.parse::<X>().ok().unwrap(),
                        bigger.parse::<Y>().ok().unwrap()
                    )
                } else {
                    (
                        bigger.parse::<X>().ok().unwrap(),
                        smaller.parse::<Y>().ok().unwrap()
                    )
                };
                let _ = heatmap.count(val_x, val_y);
            }
        )
}