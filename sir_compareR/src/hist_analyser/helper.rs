use net_ensembles::sampling::*;


#[derive(Debug, Clone)]
pub struct HistSampler<V, H>
{
    hist: H,
    binned_vals: Vec<Vec<V>>,
    miss_count: usize
}



impl<H, V> HistSampler<V, H>
where H: Histogram
{
    pub fn new(hist: H) -> Self
    {
        let mut binned_vals = Vec::with_capacity(hist.bin_count());
        binned_vals.extend(
            (0..hist.bin_count())
                .map(|_| Vec::new())
        );
        Self{
            hist,
            binned_vals,
            miss_count: 0,
        }
    }
}

impl<H, V> HistSampler<V, H>
where H: Histogram + HistogramVal<usize>
{
    pub fn count(&mut self, energy: usize, val: V)
    {
        let index = self.hist.count_val(energy);
        match index {
            Ok(index) => {
                self.binned_vals[index].push(val);
            },
            Err(_) => self.miss_count += 1
        };
    }
}

impl<H> HistSampler<usize, H>
where H: Histogram + HistogramVal<usize>
{
    pub fn percent(&mut self, p: f64) -> Vec<PercentResult>
    {
        self.binned_vals
            .iter_mut()
            .for_each(|vec| vec.sort_unstable());

        
        let gegen_wk = 1.0 - p;
        let min_needed = 1.0 / gegen_wk;
        let min_count_needed = min_needed.ceil() as usize; 
        if self.binned_vals.iter().any(|v| v.len() < min_count_needed)
        {
            eprintln!("Bin with less than {} values present!", min_count_needed);
            for (index, v) in self.binned_vals.iter().enumerate() {
                if v.len() < min_count_needed {
                    eprintln!("index: {} count: {}", index, v.len());
                }
            }
        }
        
        let percent_vec: Vec<_> = self.binned_vals
            .iter()
            .map(|vec| percent_helper(vec, p))
            .collect();
        
        let bins = self.hist.borders_clone()
            .unwrap();
        let bins_left = &bins[..];
        let bins_right = &bins[1..];

        bins_left.iter()
            .zip(bins_right.iter())
            .zip(percent_vec.iter())
            .zip(self.binned_vals.iter().map(|v| v.len()))
            .map(|(((left, right), val), len)| {
                PercentResult{
                    left: *left,
                    right: *right,
                    time: *val,
                    count: len
                }
            }).collect()
        
    }

    pub fn dirty_add(&mut self, other: &Self){
        self.miss_count += other.miss_count;
        self.binned_vals
            .iter_mut()
            .zip(other.binned_vals.iter())
            .for_each(
                |(this, other)|
                {
                    this.extend_from_slice(other.as_slice())
                }
            );
    }
}

/// assumes sorted slice
fn percent_helper(slice: &[usize], p: f64) -> usize
{
    let len = slice.len();
    let index = (len as f64 * p).ceil() as usize - 1;
    assert!(index < len);
    slice[index]
}

#[derive(Debug, Clone)]
pub struct PercentResult{
    pub left: usize,
    pub right: usize,
    pub time: usize,
    pub count: usize
}