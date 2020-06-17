use crate::ReadOpts;
use average::Mean;

pub fn norm(source: Vec<Vec<usize>>) -> Vec<Vec<f64>>
{
    source.into_iter().map(
        |v|
        {
            let max = *v.iter().max().unwrap();
            let inverse = 1.0 / max as f64;
            v.into_iter().map(|num| num as f64 * inverse).collect()
        }
    ).collect()
}

pub fn group_data(data: Vec<Vec<usize>>, opts: ReadOpts) -> Vec<Vec<Vec<f64>>>
{
    let index = |energy| (energy - 1) / opts.bin_size;
    let mut vec = vec![Vec::new(); opts.bins];
    for curve in data {
        let energy = curve[0];
        // find max
        let max = *curve.iter().skip(2).max().unwrap();
        let inverse = 1.0 / max as f64;
        let normed: Vec<_> = curve.into_iter()
            .skip(2)    // skip energy and extinction time
            .map(|val|
                {
                    val as f64 * inverse
                }
            ).collect();
        vec[index(energy)].push(normed);
    }
    vec
}

pub fn compare_curves(data: Vec<Vec<Vec<f64>>>)
{
    let mut tmp_vec = Vec::new();
    let mut diff_helper = Vec::new();
    let mut matr = Vec::with_capacity(data.len());
    for i in 0..data.len(){
        matr.push(Vec::with_capacity(data.len()));
        for _ in 0..data.len(){
            matr[i].push(0.0);
        }
        
    }
    for i in 0..data.len(){
        for j in 0..data.len(){
            if i < j {
                continue;
            }
            diff_helper.clear();
            for c1 in data[i].iter(){
                for c2 in data[j].iter(){
                    tmp_vec.clear();
                    tmp_vec.extend(
                        c1.iter()
                            .zip(c2.iter())
                            .map(|(val1, val2)| (val1 - val2).abs())
                    );
                    let mean: Mean = tmp_vec.iter().collect();
                    diff_helper.push(mean.mean());
                }
            }
            let res: Mean = diff_helper.iter().collect();
            println!("{:e}", res.mean());
            matr[i][j] = res.mean();
            matr[j][i] = matr[i][j];
        }
        println!();
    }
    for line in matr{
        for val in line{
            print!("{:e} ", val);
        }
        println!();
    }
}