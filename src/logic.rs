use crate::math::NPendulumMath;
use nalgebra::{DVector};

pub struct NPendulumSolver {
    pub n: usize,
    pub masses: Vec<f64>,
    pub lengths: Vec<f64>,
}

impl NPendulumSolver {
    pub fn new(n: usize, masses: Vec<f64>, lengths: Vec<f64>) -> Self {
        Self { n, masses, lengths }
    }

    /// Computes α = M⁻¹ (-C - G)
    pub fn accelerations(&self, angles: &[f64], ang_vels: &[f64]) -> DVector<f64> {
        let math = NPendulumMath::new(
            self.n,
            self.masses.clone(), // Still technically a clone, but math.rs can be updated to borrow
            self.lengths.clone(),
            angles.to_vec(),
            ang_vels.to_vec(),
        );

        let m_mat = math.set_mass_matrix();
        let c_vec = math.set_centripetal_matrix();
        let g_vec = math.set_grav_matrix();

        // RHS = -(C + G)
        let rhs = -(c_vec + g_vec);

        // nalgebra's LU decomposition solver (efficient for n < 100)
        m_mat.lu().solve(&rhs).expect("Linear system is singular")
    }

    /// Computes dy/dt = [ω, α]
    pub fn deriv(&self, y: &DVector<f64>) -> DVector<f64> {
        let n = self.n;
        
        // Prepare 1-indexed vectors for math logic
        let mut angles = vec![0.0; n + 1];
        let mut ang_vels = vec![0.0; n + 1];
        
        // Use slice copies to avoid manual loops
        angles[1..=n].copy_from_slice(y.rows(0, n).as_slice());
        ang_vels[1..=n].copy_from_slice(y.rows(n, n).as_slice());

        let alpha = self.accelerations(&angles, &ang_vels);

        let mut dydt = DVector::zeros(2 * n);
        
        // dθ/dt = ω
        dydt.rows_mut(0, n).copy_from(&y.rows(n, n));
        // dω/dt = α
        dydt.rows_mut(n, n).copy_from(&alpha);
        
        dydt
    }

    /// Standard RK4 Step with reduced allocations
    fn rk4_step(&self, y: &DVector<f64>, dt: f64) -> DVector<f64> {
        let k1 = self.deriv(y);
        let k2 = self.deriv(&(y + &k1 * (dt * 0.5)));
        let k3 = self.deriv(&(y + &k2 * (dt * 0.5)));
        let k4 = self.deriv(&(y + &k3 * dt));

        y + (k1 + k2 * 2.0 + k3 * 2.0 + k4) * (dt / 6.0)
    }

    /// Main integration loop
    pub fn solve(
        &self,
        initial_angles: Vec<f64>,
        initial_ang_vels: Vec<f64>,
        t_max: f64,
        n_points: usize,
    ) -> (Vec<f64>, Vec<DVector<f64>>) {
        let n = self.n;
        let dt = t_max / (n_points - 1) as f64;
        
        let mut t_axis = Vec::with_capacity(n_points);
        let mut sol = Vec::with_capacity(n_points);

        // Initialize state vector [θ1...θn, ω1...ωn]
        let mut y = DVector::zeros(2 * n);
        y.rows_mut(0, n).copy_from_slice(&initial_angles[1..=n]);
        y.rows_mut(n, n).copy_from_slice(&initial_ang_vels[1..=n]);

        let mut curr_t = 0.0;
        for _ in 0..n_points {
            t_axis.push(curr_t);
            sol.push(y.clone());
            
            y = self.rk4_step(&y, dt);
            curr_t += dt;
        }

        (t_axis, sol)
    }
}