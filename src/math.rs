use nalgebra::{DMatrix, DVector};

/// Solves the Lagrangian equations: M α + C + G = 0
/// This version preserves 1-based indexing for direct mapping to physics derivations.
pub struct NPendulumMath {
    pub g: f64,
    pub n: usize,
    pub masses: Vec<f64>,   // [0, m1, m2, ..., mn]
    pub lengths: Vec<f64>,  // [0, l1, l2, ..., ln]
    pub angles: Vec<f64>,   // [0, θ1, θ2, ..., θn]
    pub ang_vels: Vec<f64>, // [0, ω1, ω2, ..., ωn]
}

impl NPendulumMath {
    pub fn new(n: usize, masses: Vec<f64>, lengths: Vec<f64>, angles: Vec<f64>, ang_vels: Vec<f64>) -> Self {
        Self {
            g: 9.81,
            n,
            masses,
            lengths,
            angles,
            ang_vels,
        }
    }

    /// Helper to sum masses from index k to n.
    fn mass_sum_from(&self, k: usize) -> f64 {
        self.masses[k..=self.n].iter().sum()
    }

    /// Computes Mass Matrix M (n x n)
    pub fn set_mass_matrix(&self) -> DMatrix<f64> {
        // nalgebra matrices are 0-indexed internally, so M(0,0) corresponds to your M_{1,1}
        let mut m_matrix = DMatrix::zeros(self.n, self.n);

        for row in 1..=self.n {
            for col in 1..=self.n {
                let k = row.max(col);
                let m_val = self.mass_sum_from(k);
                
                let term = m_val 
                    * self.lengths[row] 
                    * self.lengths[col] 
                    * (self.angles[row] - self.angles[col]).cos();
                
                m_matrix[(row - 1, col - 1)] = term;
            }
        }
        m_matrix
    }

    /// Computes Centripetal Vector C (n x 1)
    pub fn set_centripetal_matrix(&self) -> DVector<f64> {
        let mut c_vec = DVector::zeros(self.n);

        for i in 1..=self.n {
            let mut f_term = 0.0;
            for j in 1..=self.n {
                let m_val = self.mass_sum_from(i.max(j));
                
                let term = m_val 
                    * self.lengths[i] 
                    * self.lengths[j] 
                    * (self.angles[i] - self.angles[j]).sin() 
                    * (self.ang_vels[j] * self.ang_vels[j]);
                
                f_term += term;
            }
            c_vec[i - 1] = f_term;
        }
        c_vec
    }

    /// Computes Gravity Vector G (n x 1)
    pub fn set_grav_matrix(&self) -> DVector<f64> {
        let mut g_vec = DVector::zeros(self.n);

        for i in 1..=self.n {
            let m_val = self.mass_sum_from(i);
            let term = m_val * self.g * self.lengths[i] * self.angles[i].sin();
            g_vec[i - 1] = term;
        }
        g_vec
    }
}