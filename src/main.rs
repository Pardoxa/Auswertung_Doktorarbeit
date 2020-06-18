mod parse_files;
use parse_files::*;
mod parse_cmd;
use parse_cmd::*;
mod analyse;
use analyse::*;
mod stats;

fn main() {
    let options = get_cmd_opts();
    match options {
        Opt::Heatmap{..} => write_heatmap(options.into()),
    };
}


fn write_heatmap(opts: HeatmapOpts)
{
    let sorted_data = parse_and_group_all_files( opts.clone());
    let matr =
    match opts.j  {
        0 => {
            eprintln!("0 threds not allowed, use at least 1: INVALID j");
            panic!()
        },
        1 => compare_curves(sorted_data, !opts.no_p_bar, opts.cutoff),
        _ => compare_curves_parallel(sorted_data, opts.j, !opts.no_p_bar, opts.cutoff),
    };
    write_matr(matr, opts);
}