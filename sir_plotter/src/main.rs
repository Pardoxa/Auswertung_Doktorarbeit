use sir_models::{model::ModelProb2, net_ensembles};
use net_ensembles::rand::{Rng, SeedableRng};
use net_ensembles::{traits::*,dot_constants::*};
use rand_pcg::Pcg64Mcg;
use sir_models::{*, traits::*};
use std::io::BufWriter;
use std::fs::File;
use net_ensembles::dot_options;


fn main() {
    
    small_world_plotter(20, 0.2, 0.1);
    println!("for i in *.dot; do b=\"${{i%.*}}\"; circo \"$b.dot\" -Tpdf > \"$b.pdf\" ; done");
}



fn small_world_plotter(n: usize, t: f64, r: f64)
{
    let mut rng = Pcg64Mcg::from_entropy();
    let ensemble_rng = Pcg64Mcg::from_rng(&mut rng).unwrap();

    let mut model = ModelProb2::new_sw(
        ensemble_rng, 
        0.1,
        n,
        rng, 
        t,
        r,
        1
    );

    let mut queue = Vec::new();

    let file = "0.dot";
    let file = File::create(file).unwrap();
    let writer = BufWriter::new(file);
    model.get_ensemble().as_ref()
            .dot_from_contained(
                writer,
                dot_options!(NO_OVERLAP, SPLINES, TRANSPARENT_BG, MARGIN_0),
                |state| {
                    let s = match state {
                        SirState::S => {
                            "S"
                        },
                        SirState::I => {
                            "I"
                        },
                        SirState::R => {
                            "R"
                        }
                    };
                    format!("{}\", style=\"filled\", fillcolor=\"{}", s, state.color())
                }
            ).unwrap();
    for i in 1.. {
        let file = format!("{}.dot", i);
        let file = File::create(file).unwrap();
        let writer = BufWriter::new(file);

        let next = model.iterate_unfinished(&mut queue);
        model.get_ensemble().as_ref()
            .dot_from_contained(
                writer,
                dot_options!(NO_OVERLAP, SPLINES, TRANSPARENT_BG, MARGIN_0),
                |state| {
                    let s = match state {
                        SirState::S => {
                            "S"
                        },
                        SirState::I => {
                            "I"
                        },
                        SirState::R => {
                            "R"
                        }
                    };
                    format!("{}\", style=\"filled\", fillcolor=\"{}", s, state.color())
                }
            ).unwrap();
        if ! next {
            break;
        }
    }
}