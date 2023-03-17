mod maths;
pub mod physics;

use std::{fs, time::Instant};

use crate::physics::{Enviroment, Physics};
use CFDplotlib::{Env, Plot};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = Instant::now();

    let env = Env::new();
    let mut plot = Plot::new(&env, 25);
    let title = "Cavity Flow Pressure+velocity";

    let enviroment_data =
        fs::read_to_string("Enviroments/cavity_flow.json").expect("Unable to read JSON file");

    let enviroment: Enviroment =
        serde_json::from_str(&enviroment_data).expect("JSON does not have correct format.");

    let mut physics = Physics::new(enviroment);

    let mut last_var;
    let mut iter = 0;
    let mut var = 0.0;
    let mut error = f32::INFINITY;

    plot.pcolormesh(&physics.x, &physics.y, &physics.p, "viridis", title);
    plot.quiver(&physics.x, &physics.y, &physics.u, &physics.v);
    plot.setup_animation();
    plot.update_frame(&physics.p, &physics.u, &physics.v);

    while iter < 1000 && error.abs() > 1e-7 {
        physics.iterate();

        last_var = var;
        var = physics.u.iter().fold(0.0, |acc, x| acc + x.abs());
        if var != 0.0 {
            error = (var - last_var) / var;
        } else {
            error = var - last_var;
        }

        //plot.update_frame(&physics.p, &physics.u, &physics.v);

        iter += 1;
        dbg!(iter, error.abs());
    }
    plot.update_frame(&physics.p, &physics.u, &physics.v);
    plot.finish_animation();
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(())
}
