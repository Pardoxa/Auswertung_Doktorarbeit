mod parse_files;
use parse_files::*;
mod parse_cmd;
use parse_cmd::*;
mod calc;
use calc::*;

fn main() {
    let options = get_cmd_opts();
    match options {
        Opt::Read{..} => write_heatmap(options.into()),
    };
}


fn write_heatmap(opts: ReadOpts)
{
    let save_file = opts.generate_filename();
    let vec = parse_all_files(opts.clone());
    let sorted_data = group_data(vec, opts);
    let matr = compare_curves_parallel(sorted_data);
    write_matr(matr, save_file);
    //println!("{:?}", sorted_data);
}