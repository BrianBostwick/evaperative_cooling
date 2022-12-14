//! Single particle in a cross beam optical dipole trap
extern crate atomecs as lib;
extern crate nalgebra;
use lib::atom::{Atom, Force, Mass, Position, Velocity};
use lib::dipole::{self, DipolePlugin};
use lib::integrator::Timestep;
use lib::laser::{self, LaserPlugin};
use lib::laser::gaussian::GaussianBeam;
use lib::laser::intensity::{LaserIntensitySamplers};
use lib::output::file::{FileOutputPlugin, Text};
use lib::simulation::SimulationBuilder;
use nalgebra::Vector3;
use specs::prelude::*;
use std::time::Instant;
use lib::initiate::NewlyCreated;
use std::fs::File;
use std::io::{Error, Write};
use lib::collisions::{CollisionPlugin, ApplyCollisionsOption, CollisionParameters, CollisionsTracker};
use lib::sim_region::{ SimulationVolume, VolumeType};
use lib::shapes::Sphere;
use lib::ramp::{Ramp, RampUpdateSystem};

use easy_ml::matrices::Matrix;
use easy_ml::distributions::MultivariateGaussian;
use rand::{Rng, SeedableRng};
use rand::distributions::{DistIter, Standard};
use rand_chacha::ChaCha8Rng;

// use lib::gravity::GravityPlugin;


const BEAM_NUMBER: usize = 2;

fn main() {
    let now = Instant::now();

                                   //units
    let wavelength    = 1064.0e-9; //m
    let e_radius      = 60.0e-6/2.0_f64.sqrt(); //m

    let dt            = 1.0e-6;      //s
    let sim_length    = 100000;       //none

    let initial_power = 7.0;             //W
    let final_power   = 0.25;          //W
    let rate          = (  final_power - initial_power   ) / ( 0.5 * (sim_length as f64) * dt ); //0.0;       //W/s

    let data_rate = 500;

    // Configure simulation output.
    let mut sim_builder = SimulationBuilder::default();

    sim_builder.world.register::<NewlyCreated>();
    sim_builder.add_plugin(LaserPlugin::<{BEAM_NUMBER}>);
    sim_builder.add_plugin(DipolePlugin::<{BEAM_NUMBER}>);
    sim_builder.add_end_frame_systems();
    sim_builder.add_plugin(CollisionPlugin);
    sim_builder.add_plugin(FileOutputPlugin::<Position, Text, Atom>::new("D:/data_1/pos_ramp_test_007.txt".to_string(), data_rate));
    sim_builder.add_plugin(FileOutputPlugin::<Velocity, Text, Atom>::new("D:/data_1/vel_ramp_test_007.txt".to_string(), data_rate));
    sim_builder.add_plugin(
        FileOutputPlugin::<
            LaserIntensitySamplers<{BEAM_NUMBER}>,
            Text,
            LaserIntensitySamplers<{BEAM_NUMBER}>>::new(
                "D:/data_1/intensity_ramp_test_007.txt".to_string(),
                data_rate
            )
    );
    // sin_builder.add_plugin(GravityPlugin);
    sim_builder.dispatcher_builder.add(
        RampUpdateSystem::<GaussianBeam>::default(),
        "update_comp",
        &[],
    );

    let mut sim = sim_builder.build();

    // Creating simulation volume
    let sphere_pos = Vector3::new(0.0, 0.0, 0.0);
    let sphere_radius = 60.0e-6;

    sim.world
        .create_entity()
        .with(Position { pos: sphere_pos })
        .with(Sphere {
            radius: sphere_radius,
        })
        .with(SimulationVolume {
            volume_type: VolumeType::Inclusive,
        })
        .build();

    // Creating a mutable frames vector for the ramp
    let mut frames = vec![];

    // Appending the ramp powers for each frame to the vector, this in the form of a paired list (time, componant value)
    // for i in 0..sim_length {
    //     frames.append(
    //         &mut vec![
    //             (
    //                 i as f64 * dt,
    //                 GaussianBeam {
    //                     intersection: Vector3::new(0.0, 0.0, 0.0),
    //                     e_radius:  e_radius,
    //                     // This is a exponetial ramp down of the power
    //                     // power: ( initial_power - final_power ) * 2.0_f64.powf( -1.0 * rate * i as f64 * dt ) + final_power,
    //                     // Linear ramp for testing
    //                     power: rate * ( i as f64 ) * dt + initial_power,
    //
    //                     // power: ( 0.187 / (0.01 * 2.0 * 3.1415) ) * 2.0_f64.powf( - (  i as f64 * dt - 0.1 ).powf(2.0) / ( 2.0 * 0.01_f64.powf(2.0) ) ) + 7.0,
    //                     direction: Vector3::x(),
    //                     rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(&wavelength, &e_radius),
    //                     ellipticity: 0.0
    //                 }
    //             )
    //         ]
    //     )
    // }


    for i in 0..sim_length {
        if i <= (( sim_length as f64 * 0.5) as i32 ) {
            frames.append(
                &mut vec![
                    (
                        i as f64 * dt,
                        GaussianBeam {
                            intersection: Vector3::new(0.0, 0.0, 0.0),
                            e_radius:  e_radius,
                            // This is a exponetial ramp down of the power
                            // power: ( initial_power - final_power ) * 2.0_f64.powf( -1.0 * rate * i as f64 * dt ) + final_power,
                            // Linear ramp for testing
                            power: rate * ( i as f64 ) * dt + initial_power,

                            // power: ( 0.187 / (0.01 * 2.0 * 3.1415) ) * 2.0_f64.powf( - (  i as f64 * dt - 0.1 ).powf(2.0) / ( 2.0 * 0.01_f64.powf(2.0) ) ) + 7.0,
                            direction: Vector3::x(),
                            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(&wavelength, &e_radius),
                            ellipticity: 0.0
                        }
                    )
                ]
            )
        }
        else {
            frames.append(
                &mut vec![
                    (
                        i as f64 * dt,
                        GaussianBeam {
                            intersection: Vector3::new(0.0, 0.0, 0.0),
                            e_radius:  e_radius,
                            // This is a exponetial ramp down of the power
                            // power: ( initial_power - final_power ) * 2.0_f64.powf( -1.0 * rate * i as f64 * dt ) + final_power,
                            // Linear ramp for testing
                            power: final_power,

                            // power: ( 0.187 / (0.01 * 2.0 * 3.1415) ) * 2.0_f64.powf( - (  i as f64 * dt - 0.1 ).powf(2.0) / ( 2.0 * 0.01_f64.powf(2.0) ) ) + 7.0,
                            direction: Vector3::x(),
                            rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(&wavelength, &e_radius),
                            ellipticity: 0.0
                        }
                    )
                ]
            )
        }

        }


    // Adding beam along the x direction
    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius: e_radius,
        power: 7.0,
        direction: Vector3::x(),
        rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(&wavelength, &e_radius),
        ellipticity: 0.0,
    };

    let ramp = Ramp{
        prev: 0,
        keyframes: frames,
    };

    sim.world
        .create_entity()
        .with(gaussian_beam)
        .with(dipole::DipoleLight { wavelength })
        .with(laser::frame::Frame {
            x_vector: Vector3::y(),
            y_vector: Vector3::z(),
        })
        .with(ramp.clone())
        .build();

    // Adding beam along the y direction
    let gaussian_beam = GaussianBeam {
        intersection: Vector3::new(0.0, 0.0, 0.0),
        e_radius: e_radius,
        power: 7.0,
        direction: Vector3::y(),
        rayleigh_range: crate::laser::gaussian::calculate_rayleigh_range(&wavelength, &e_radius),
        ellipticity: 0.0,
    };

    sim.world
        .create_entity()
        .with(gaussian_beam)
        .with(dipole::DipoleLight { wavelength })
        .with(laser::frame::Frame {
            x_vector: Vector3::x(),
            y_vector: Vector3::z(),
        })
        .with(ramp.clone())
        .build();


    let mut rng = rand::thread_rng();
    let x: u8 = rng.gen();
    // use a fixed seed random generator from the rand crate
    let mut random_generator = ChaCha8Rng::seed_from_u64(x.into());

    let cluster_xy = MultivariateGaussian::new(
            Matrix::column(vec![ 0.0, 0.0 ]),
            Matrix::from(vec![
                vec![ 6.390371318625299e-11, 0.0 ],
                vec![ 0.0, 4.7799785348289716e-04  ]
                ])
            );

    let cluster_z = MultivariateGaussian::new(
            Matrix::column(vec![ 0.0, 0.0 ]),
            Matrix::from(vec![
                vec![ 3.195234569049243e-11, 0.0 ],
                vec![ 0.0, 4.7799785348289716e-04 ]
                ])
            );

    // Generate points for each cluster
    let points = 1;
    let mut random_numbers: DistIter<Standard, &mut ChaCha8Rng, f64> = (&mut random_generator).sample_iter(Standard);

    // Create a single test atom
    let atom_number = 2_500;
    for _ in 0..atom_number {

        let x_points = cluster_xy.draw( &mut random_numbers, points).unwrap();
        let y_points = cluster_xy.draw( &mut random_numbers, points).unwrap();
        let z_points = cluster_z.draw( &mut random_numbers, points).unwrap();

        sim.world
            .create_entity()
            .with(Atom)
            .with(Mass { value: 87.0 })
            .with(Force::new())
            .with(Position {
                pos: Vector3::new(
                    x_points.get(0,0),
                    y_points.get(0,0),
                    z_points.get(0,0),
                ),
            })
            .with(Velocity {
                vel: Vector3::new(
                    x_points.get(0,1),
                    y_points.get(0,1),
                    z_points.get(0,1),
                ),
            })

            .with(dipole::Polarizability::calculate_for(
                wavelength, 461e-9, 2.1e8,
            ))
            .with(lib::initiate::NewlyCreated)
            .build();
    }

    sim.world.insert(ApplyCollisionsOption);
    sim.world.insert(CollisionParameters {
        macroparticle: 4e2,        //The number of real partcles per particle simulated
        box_number: 1000,          //Any number large enough to cover entire cloud with collision boxes. Overestimating box number will not affect performance.
        box_width: 1e-6,           //Too few particles per box will both underestimate collision rate and cause large statistical fluctuations.
                                   //Boxes must also be smaller than typical length scale of density variations within the cloud, since the collisions model treats gas within a box as homogeneous.
        sigma: 1.95e-19,           //Approximate collisional cross section of Sr
        collision_limit: 10_000.0, //Maximum number of collisions that can be calculated in one frame.
                                   //This avoids absurdly high collision numbers if many atoms are initialised with the same position, for example.
    });
    sim.world.insert(CollisionsTracker {
        num_collisions: Vec::new(),
        num_atoms: Vec::new(),
        num_particles: Vec::new(),
    });

    // Define timestep
    sim.world.insert(Timestep { delta: dt });
    //Timestep must also be much smaller than mean collision time

    let mut filename = File::create("D:/data_1/collisions_ramp_test_007.txt").expect("Cannot create file.");

    // Run the simulation for a number of steps.
    for _i in 0..sim_length {
        sim.step();

        if (_i > 0) && (_i % 50_i32 == 0) {
            let tracker = sim.world.read_resource::<CollisionsTracker>();
            let _result = write_collisions_tracker(
                &mut filename,
                &_i,
                &tracker.num_collisions,
                &tracker.num_atoms,
                &tracker.num_particles,
            )
            .expect("Could not write collision stats file.");
        }
    }
    println!("Simulation completed in {} ms.", now.elapsed().as_millis());
}


// Write collision stats to file

fn write_collisions_tracker(
    filename: &mut File,
    step: &i32,
    num_collisions: &Vec<i32>,
    num_atoms: &Vec<f64>,
    num_particles: &Vec<i32>,
) -> Result<(), Error> {
    let str_collisions: Vec<String> = num_collisions.iter().map(|n| n.to_string()).collect();
    let str_atoms: Vec<String> = num_atoms.iter().map(|n| format!("{:.2}", n)).collect();
    let str_particles: Vec<String> = num_particles.iter().map(|n| n.to_string()).collect();
    write!(
        filename,
        "{:?}\r\n{:}\r\n{:}\r\n{:}\r\n",
        step,
        str_collisions.join(" "),
        str_atoms.join(" "),
        str_particles.join(" ")
    )?;
    Ok(())
}
