mod parse_files;
use parse_files::*;
mod parse_cmd;
use parse_cmd::*;
mod analyse;
use analyse::*;

fn main() {
    let options = get_cmd_opts();
    match options {
        Opt::Heatmap{..} => write_heatmap(options.into()),
    };
}


fn write_heatmap(opts: HeatmapOpts)
{
    let save_file = opts.generate_filename();
    let vec = parse_all_files(opts.clone());
    let sorted_data = group_data(vec, opts.clone());
    let matr =
    match opts.j  {
        0 => {
            eprintln!("0 threds not allowed, use at least 1: INVALID j");
            panic!()
        },
        1 => compare_curves(sorted_data, !opts.no_p_bar),
        _ => compare_curves_parallel(sorted_data, opts.j, !opts.no_p_bar),
    };
    write_matr(matr, save_file);
}