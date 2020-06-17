mod parse_files;
use parse_files::*;
mod parse_cmd;
use parse_cmd::*;
mod calc;
use calc::*;

fn main() {
    let options = get_cmd_opts();
    match options {
        Opt::Read{..} => read_helper(options.into()),
    };
}


fn read_helper(opts: ReadOpts)
{
    let vec = parse_all_files(opts.clone());
    
    let sorted_data = group_data(vec, opts);
    compare_curves(sorted_data);
    //println!("{:?}", sorted_data);
}