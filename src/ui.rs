// src/ui.rs
use crate::logic::NPendulumSolver;
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use nalgebra::DVector;

#[derive(Deserialize)]
pub struct SimParams {
    n: usize,                // Number of pendulums
    masses: String,          // Comma-separated masses
    lengths: String,         // Comma-separated lengths
    initial_angles: String,  // Comma-separated initial angles (degrees)
    t_max: f64,              // Simulation duration
    n_points: usize,         // Resolution
}

#[derive(Serialize)]
struct SimResponse {
    success: bool,
    animation_data: AnimationData,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Serialize, Default)]
struct AnimationData {
    positions: Vec<Vec<f64>>, // Flattened [x1, y1, x2, y2...] per time step
    n: usize,
    limit: f64,               // Boundary for frontend scaling
}

/// Helper: Parses a comma-separated string into a Vec<f64>.
fn parse_csv_f64(s: &str) -> Vec<f64> {
    s.split(',')
        .filter_map(|x| x.trim().parse::<f64>().ok())
        .collect()
}

/// Helper: Converts angular states (theta) into Cartesian coordinates (x, y).
/// Returns a vector of time steps, where each step is [x1, y1, x2, y2, ...].
fn compute_positions(sol: &[DVector<f64>], n: usize, lengths: &[f64]) -> Vec<Vec<f64>> {
    let mut positions = Vec::with_capacity(sol.len());

    for state in sol {
        let mut step_coords = Vec::with_capacity(2 * n);
        let mut curr_x = 0.0;
        let mut curr_y = 0.0;

        // state contains [theta_1 ... theta_n, omega_1 ... omega_n]
        // logic.rs uses 1-based indexing for lengths (index 0 is dummy)
        // state vector from nalgebra is 0-indexed: state[0] is theta_1
        for k in 0..n {
            let theta = state[k]; // theta_(k+1)
            let len = lengths[k + 1]; // L_(k+1)

            curr_x += len * theta.sin();
            curr_y -= len * theta.cos();

            step_coords.push(curr_x);
            step_coords.push(curr_y);
        }
        positions.push(step_coords);
    }
    positions
}

/// Main Handler: Orchestrates parsing, solving, and response formatting.
pub async fn simulate_handler(params: web::Json<SimParams>) -> Result<HttpResponse> {
    // 1. Parse Inputs
    let masses = parse_csv_f64(&params.masses);
    let lengths = parse_csv_f64(&params.lengths);
    let angles_deg = parse_csv_f64(&params.initial_angles);

    // 2. Validate Inputs
    if masses.len() != params.n || lengths.len() != params.n || angles_deg.len() != params.n {
        return Ok(HttpResponse::Ok().json(SimResponse {
            success: false,
            animation_data: AnimationData::default(),
            message: Some(format!(
                "Input length mismatch. Expected {}, got M:{}, L:{}, A:{}",
                params.n, masses.len(), lengths.len(), angles_deg.len()
            )),
        }));
    }

    // 3. Prepare Physics Vectors (1-based indexing padding)
    // We prepend 0.0 because the physics logic (math.rs) expects 1-based indices [dummy, m1, m2...]
    let mut full_masses = vec![0.0];
    full_masses.extend(&masses);

    let mut full_lengths = vec![0.0];
    full_lengths.extend(&lengths);

    let mut full_angles = vec![0.0];
    full_angles.extend(angles_deg.iter().map(|d| d.to_radians()));

    let initial_ang_vels = vec![0.0; params.n + 1]; // Start from rest

    // 4. Initialize Solver
    let solver = NPendulumSolver::new(params.n, full_masses, full_lengths.clone());

    // 5. Run Simulation
    // returns (time_vector, state_vectors)
    let (_t, sol) = solver.solve(
        full_angles,
        initial_ang_vels,
        params.t_max,
        params.n_points,
    );

    // 6. Post-Process Results
    // Calculate display limit (Total length + padding)
    let limit: f64 = lengths.iter().sum::<f64>() + 0.5;
    
    // Convert angles to Cartesian coordinates for the frontend
    let positions = compute_positions(&sol, params.n, &full_lengths);

    // 7. Return JSON
    Ok(HttpResponse::Ok().json(SimResponse {
        success: true,
        animation_data: AnimationData {
            positions,
            n: params.n,
            limit,
        },
        message: None,
    }))
}