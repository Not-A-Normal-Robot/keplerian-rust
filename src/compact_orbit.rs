use core::f64::consts::{PI, TAU};

use crate::{
    keplers_equation, keplers_equation_derivative, keplers_equation_second_derivative, sinhcosh, solve_monotone_cubic, ApoapsisSetterError, Matrix3x2, Orbit, OrbitTrait
};

/// A minimal struct representing a Keplerian orbit.
/// 
/// This struct minimizes memory footprint by not caching variables.  
/// Because of this, calculations can be slower than caching those variables.  
/// For this reason, you might consider using the `Orbit` struct instead.
/// 
/// # Example
/// ```
/// use keplerian_sim::{CompactOrbit, OrbitTrait};
/// 
/// let orbit = CompactOrbit::new(
///     // Initialize using eccentricity, periapsis, inclination,
///     // argument of periapsis, longitude of ascending node,
///     // and mean anomaly at epoch
/// 
///     // Eccentricity
///     0.0,
/// 
///     // Periapsis
///     1.0,
/// 
///     // Inclination
///     0.0,
/// 
///     // Argument of periapsis
///     0.0,
/// 
///     // Longitude of ascending node
///     0.0,
/// 
///     // Mean anomaly at epoch
///     0.0,
/// );
/// 
/// let orbit = CompactOrbit::with_apoapsis(
///     // Initialize using apoapsis in place of eccentricity
///     
///     // Apoapsis
///     2.0,
/// 
///     // Periapsis
///     1.0,
/// 
///     // Inclination
///     0.0,
/// 
///     // Argument of periapsis
///     0.0,
/// 
///     // Longitude of ascending node
///     0.0,
/// 
///     // Mean anomaly at epoch
///     0.0,
/// );
/// ```
/// See [Orbit::new] and [Orbit::with_apoapsis] for more information.
#[derive(Clone, Debug)]
pub struct CompactOrbit {
    /// The eccentricity of the orbit.  
    /// e < 1: ellipse  
    /// e = 1: parabola  
    /// e > 1: hyperbola  
    /// 
    /// See more: <https://en.wikipedia.org/wiki/Orbital_eccentricity>
    pub eccentricity: f64,

    /// The periapsis of the orbit, in meters.
    /// 
    /// The periapsis of an orbit is the distance at the closest point
    /// to the parent body.
    /// 
    /// More simply, this is the "minimum altitude" of an orbit.
    pub periapsis: f64,

    /// The inclination of the orbit, in radians.
    /// The inclination of an orbit is the angle between the plane of the
    /// orbit and the reference plane.
    /// 
    /// In simple terms, it tells you how "tilted" the orbit is.
    pub inclination: f64,

    /// The argument of periapsis of the orbit, in radians.
    /// 
    /// Wikipedia:  
    /// The argument of periapsis is the angle from the body's
    /// ascending node to its periapsis, measured in the direction of
    /// motion.  
    /// <https://en.wikipedia.org/wiki/Argument_of_periapsis>
    /// 
    /// In simple terms, it tells you how, and in which direction,
    /// the orbit "tilts".
    pub arg_pe: f64,

    /// The longitude of ascending node of the orbit, in radians.
    /// 
    /// Wikipedia:  
    /// The longitude of ascending node is the angle from a specified
    /// reference direction, called the origin of longitude, to the direction
    /// of the ascending node, as measured in a specified reference plane.  
    /// <https://en.wikipedia.org/wiki/Longitude_of_the_ascending_node>
    /// 
    /// In simple terms, it tells you how, and in which direction,
    /// the orbit "tilts".
    pub long_asc_node: f64,

    /// The mean anomaly at orbit epoch, in radians.
    /// 
    /// For elliptic orbits, it's measured in radians and so are bounded
    /// between 0 and tau; anything out of range will get wrapped around.  
    /// For hyperbolic orbits, it's unbounded.
    /// 
    /// Wikipedia:  
    /// The mean anomaly at epoch, `M_0`, is defined as the instantaneous mean
    /// anomaly at a given epoch, `t_0`.  
    /// <https://en.wikipedia.org/wiki/Mean_anomaly#Mean_anomaly_at_epoch>
    /// 
    /// In simple terms, this modifies the "offset" of the orbit progression.
    pub mean_anomaly: f64,
}

// Initialization and cache management
impl CompactOrbit {
    /// Creates a new `CompactOrbit` instance with the given parameters.
    /// 
    /// Note: This function uses eccentricity instead of apoapsis.  
    /// If you want to provide an apoapsis instead, consider using the
    /// [`CompactOrbit::with_apoapsis`] function instead.
    /// 
    /// ### Parameters
    /// - `eccentricity`: The eccentricity of the orbit.
    /// - `periapsis`: The periapsis of the orbit, in meters.
    /// - `inclination`: The inclination of the orbit, in radians.
    /// - `arg_pe`: The argument of periapsis of the orbit, in radians.
    /// - `long_asc_node`: The longitude of ascending node of the orbit, in radians.
    /// - `mean_anomaly`: The mean anomaly of the orbit, in radians.
    pub fn new(
        eccentricity: f64, periapsis: f64,
        inclination: f64, arg_pe: f64, long_asc_node: f64,
        mean_anomaly: f64
    ) -> CompactOrbit {
        return CompactOrbit {
            eccentricity, periapsis,
            inclination, arg_pe, long_asc_node,
            mean_anomaly,
        };
    }

    /// Creates a new `CompactOrbit` instance with the given parameters.
    /// 
    /// Note: This function uses apoapsis instead of eccentricity.  
    /// Because of this, it's not recommended to initialize
    /// parabolic or hyperbolic 'orbits' with this function.  
    /// If you're looking to initialize a parabolic or hyperbolic
    /// trajectory, consider using the [`CompactOrbit::new`] function instead.
    /// 
    /// ### Parameters
    /// - `apoapsis`: The apoapsis of the orbit, in meters.
    /// - `periapsis`: The periapsis of the orbit, in meters.
    /// - `inclination`: The inclination of the orbit, in radians.
    /// - `arg_pe`: The argument of periapsis of the orbit, in radians.
    /// - `long_asc_node`: The longitude of ascending node of the orbit, in radians.
    /// - `mean_anomaly`: The mean anomaly of the orbit, in radians.
    pub fn with_apoapsis(
        apoapsis: f64, periapsis: f64,
        inclination: f64, arg_pe: f64, long_asc_node: f64,
        mean_anomaly: f64
    ) -> CompactOrbit {
        let eccentricity = (apoapsis - periapsis ) / (apoapsis + periapsis);
        return CompactOrbit::new(eccentricity, periapsis, inclination, arg_pe, long_asc_node, mean_anomaly);
    }

    /// Creates a unit orbit.
    /// 
    /// The unit orbit is a perfect circle of radius 1 and no "tilt".
    pub fn new_default() -> CompactOrbit {
        return Self::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
    }
}


/// A constant used to get the initial seed for the eccentric anomaly.
/// 
/// It's very arbitrary, but according to some testing, a value just
/// below 1 works better than exactly 1.
/// 
/// Source:
/// "Two fast and accurate routines for solving the elliptic Kepler
/// equation for all values of the eccentricity and mean anomaly"
/// by Daniele Tommasini and David N. Olivieri,
/// section 2.1.2, 'The "rational seed"'
/// 
/// https://doi.org/10.1051/0004-6361/202141423
const B: f64 = 0.999999;

/// A constant used for the Laguerre method.
/// 
/// The paper "An improved algorithm due to
/// laguerre for the solution of Kepler's equation."
/// says:
/// 
/// > Similar experimentation has been done with values of n both greater and smaller
/// than n = 5. The speed of convergence seems to be very insensitive to the choice of n.
/// No value of n was found to yield consistently better convergence properties than the
/// choice of n = 5 though specific cases were found where other choices would give
/// faster convergence.
const N_U32: u32 = 5;

/// A constant used for the Laguerre method.
/// 
/// The paper "An improved algorithm due to
/// laguerre for the solution of Kepler's equation."
/// says:
/// 
/// > Similar experimentation has been done with values of n both greater and smaller
/// than n = 5. The speed of convergence seems to be very insensitive to the choice of n.
/// No value of n was found to yield consistently better convergence properties than the
/// choice of n = 5 though specific cases were found where other choices would give
/// faster convergence.
const N_F64: f64 = 5.0;

/// The maximum number of iterations for numerical approach methods.
/// 
/// This is used to prevent infinite loops in case the method fails to converge.
const NUMERIC_MAX_ITERS: u32 = 1000;

const PI_SQUARED: f64 = PI * PI;

impl CompactOrbit {
    // "An improved algorithm due to laguerre for the solution of Kepler's equation."
    // by Bruce A. Conway
    // https://doi.org/10.1007/bf01230852
    fn get_eccentric_anomaly_elliptic(&self, mut mean_anomaly: f64) -> f64 {
        let mut sign = 1.0;
        // Use the symmetry and periodicity of the eccentric anomaly
        // Equation 2 from the paper
        // "Two fast and accurate routines for solving
        // the elliptic Kepler equation for all values
        // of the eccentricity and mean anomaly"
        mean_anomaly %= TAU;
        if mean_anomaly > PI {
            // return self.get_eccentric_anomaly_elliptic(mean_anomaly - TAU);
            mean_anomaly -= TAU;
        }
        if mean_anomaly < 0.0 {
            // return -self.get_eccentric_anomaly_elliptic(-mean_anomaly);
            mean_anomaly = -mean_anomaly;
            sign = -1.0;
        }

        // Starting guess
        // Section 2.1.2, 'The "rational seed"',
        // equation 19, from the paper
        // "Two fast and accurate routines for solving
        // the elliptic Kepler equation for all values
        // of the eccentricity and mean anomaly"
        //
        // E_0 = M + (4beM(pi - M)) / (8eM + 4e(e-pi) + pi^2)
        // where:
        // e = eccentricity
        // M = mean anomaly
        // pi = the constant PI
        // b = the constant B
        let mut eccentric_anomaly =
            mean_anomaly +
            (4.0 * self.eccentricity * B * mean_anomaly * (PI - mean_anomaly)) /
            (
                8.0 * self.eccentricity * mean_anomaly +
                4.0 * self.eccentricity * (self.eccentricity - PI) +
                PI_SQUARED
            );

        // Laguerre's method
        // 
        // i = 2, 3, ..., n
        //
        // D = sqrt((n-1)^2(f'(x_i))^2 - n(n-1)f(x_i)f''(x_i))
        //
        // x_i+1 = x_i - (nf(x_i) / (f'(x_i) +/- D))
        // ...where the "+/-" is chosen to so that abs(denominator) is maximized
        for _ in 2..N_U32 {
            let f = keplers_equation(mean_anomaly, eccentric_anomaly, self.eccentricity);
            let fp = keplers_equation_derivative(eccentric_anomaly, self.eccentricity);
            let fpp = keplers_equation_second_derivative(eccentric_anomaly, self.eccentricity);

            let n = N_F64;
            let n_minus_1 = n - 1.0;
            let d = ((n_minus_1 * n_minus_1) * fp * fp - n * n_minus_1 * f * fpp).abs().sqrt().copysign(fp);

            let denominator = n * f / (fp + d.max(1e-30));
            eccentric_anomaly -= denominator;

            if denominator.abs() < 1e-30 || !denominator.is_finite() {
                // dangerously close to div-by-zero, break out
                break;
            }
        }

        return eccentric_anomaly * sign;
    }

    /// Get an initial guess for the hyperbolic eccentric anomaly of an orbit.
    /// 
    /// From the paper:  
    /// "A new method for solving the hyperbolic Kepler equation"  
    /// by Baisheng Wu et al.  
    /// Quote:
    /// "we divide the hyperbolic eccentric anomaly interval into two parts:
    /// a finite interval and an infinite interval. For the finite interval,
    /// we apply a piecewise Pade approximation to establish an initial
    /// approximate solution of HKE. For the infinite interval, an analytical
    /// initial approximate solution is constructed."
    pub fn get_approx_hyp_ecc_anomaly(&self, mean_anomaly: f64) -> f64 {
        let sign = mean_anomaly.signum();
        let mean_anomaly = mean_anomaly.abs();
        const SINH_5: f64 = 74.20321057778875;

        // (Paragraph after Eq. 5 in the aforementioned paper)
        //   The [mean anomaly] interval [0, e_c sinh(5) - 5) can
        //   be separated into fifteen subintervals corresponding to
        //   those intervals of F in [0, 5), see Eq. (4).
        return sign * if mean_anomaly < self.eccentricity * SINH_5 - 5.0 {
            // We use the Pade approximation of sinh of order
            // [3 / 2], in `crate::generated_sinh_approximator`.
            // We can then rearrange the equation to a cubic
            // equation in terms of (F - a) and solve it.
            //
            // To quote the paper:
            //   Replacing sinh(F) in [the hyperbolic Kepler
            //   equation] with its piecewise Pade approximation
            //   defined in Eq. (4) [`crate::generated_sinh_approximator`]
            //   yields:
            //     e_c P(F) - F = M_h                          (6)
            // 
            //   Eq. (6) can be written as a cubic equation in u = F - a, as
            //     (e_c p_3 - q_2)u^3 +
            //     (e_c p_2 - (M_h + a)q_2 - q_1) u^2 +
            //     (e_c p_1 - (M_h + a)q_1 - 1)u +
            //     e_c s - M_h - a = 0                         (7)
            //   
            //   Solving Eq. (7) and picking the real root F = F_0 in the
            //   corresponding subinterval results in an initial approximate
            //   solution to [the hyperbolic Kepler equation].
            //
            // For context:
            // - `e_c` is eccentricity
            // - `p_*`, `q_*`, `a`, and `s` is derived from the Pade approximation
            //   arguments, which can be retrieved using the
            //   `generated_sinh_approximator::get_sinh_approx_params` function
            // - `M_h` is the mean anomaly
            // - `F` is the eccentric anomaly

            use crate::generated_sinh_approximator::get_sinh_approx_params;
            let params = get_sinh_approx_params(mean_anomaly);

            // We first get the value of each coefficient in the cubic equation:
            // Au^3 + Bu^2 + Cu + D = 0
            let mean_anom_plus_a = mean_anomaly + params.a;
            let coeff_a = self.eccentricity * params.p_3 - params.q_2;
            let coeff_b = self.eccentricity * params.p_2 - mean_anom_plus_a * params.q_2 - params.q_1;
            let coeff_c = self.eccentricity * params.p_1 - mean_anom_plus_a * params.q_1 - 1.0;
            let coeff_d = self.eccentricity * params.s - mean_anomaly - params.a;

            // Then we solve it to get the value of u = F - a
            let u = solve_monotone_cubic(coeff_a, coeff_b, coeff_c, coeff_d);
            
            u + params.a
        } else {
            // Equation 13
            // A *very* rough guess, with an error that may exceed 1%.
            let rough_guess = (2.0 * mean_anomaly / self.eccentricity).ln();

            /*
            A fourth-order Schröder iteration of the second kind
            is performed to create a better guess.
            ...Apparently it's not a well-known thing, but the aforementioned paper
            referenced this other paper about Schröder iterations:
            https://doi.org/10.1016/j.cam.2019.02.035
            
            To do the Schröder iteration, we need to compute a delta value
            to be added to the rough guess. Part of Equation 15 from the paper is below.
            
            delta = (
                    6 * [e_c^2 / (4 * M_h) + F_a] / (e_c * c_a - 1) +
                    3 * [e_c * s_a / (e_c * c_a - 1)]{[e_c^2 / (4 * M_h) + F_a] / (e_c * c_a - 1)}^2
                ) / (
                    6 +
                    6 * [e_c * s_a / (e_c * c_a - 1)]{[e_c^2 / (4 * M_h) + F_a] / (e_c * c_a - 1)} +
                    [e_c * c_a / (e_c * c_a - 1)]{[e_c^2 / (4 * M_h) + F_a] / (e_c * c_a - 1)}^2
                )
            ...where:
            e_c = eccentricity
            F_a = rough guess
            c_a = cosh(F_a) = 0.5 * [2 * M_h / e_c + e_c / (2 * M_h)],
            s_a = sinh(F_a) = 0.5 * [2 * M_h / e_c - e_c / (2 * M_h)]
            
            Although the equation may look intimidating, there are a lot of repeated values.
            We can simplify the equation by extracting the repeated values.
            
            Let:
                alpha = e_c^2 / (4 * M_h) + F_a
                beta  = 1 / (e_c * c_a - 1)
                gamma = alpha * beta
            
            The equation gets simplified into:
            
            delta = (
                    6 * gamma +
                    3 * e_c * s_a * beta * gamma^2
                ) / (
                    6 +
                    6 * e_c * s_a * beta * gamma +
                    e_c * c_a * beta * gamma^2
                )

            Then we can refine the rough guess into the initial guess:
            F_0 = F_a + delta
            */

            let (c_a, s_a) = {
                // c_a and s_a has a lot of repeated values, so we can
                // optimize by calculating them together.
                // c_a, s_a = 0.5 * [2 * M_h / e_c +- e_c / (2 * M_h)]
                //
                // define "left"  = 2 * M_h / e_c
                // define "right" = e_c / (2 * M_h)

                let left = 2.0 * mean_anomaly / self.eccentricity;
                let right = self.eccentricity / (2.0 * mean_anomaly);

                (0.5 * (left + right), 0.5 * (left - right))
            };

            let alpha =
                self.eccentricity * self.eccentricity /
                (4.0 * mean_anomaly) + rough_guess;
            
            let beta = (self.eccentricity * c_a - 1.0).recip();

            let gamma = alpha * beta;
            let gamma_sq = gamma * gamma;

            let delta = (
                6.0 * alpha * beta +
                3.0 * (self.eccentricity * s_a * beta) * gamma_sq
            ) / (
                6.0 +
                6.0 * (self.eccentricity * s_a * beta) * gamma +
                (self.eccentricity * c_a * beta) * gamma_sq
            );

            rough_guess + delta
        };
    }

    /// From the paper:  
    /// "A new method for solving the hyperbolic Kepler equation"  
    /// by Baisheng Wu et al.  
    fn get_eccentric_anomaly_hyperbolic(&self, mean_anomaly: f64) -> f64 {
        let mut ecc_anom = self.get_approx_hyp_ecc_anomaly(mean_anomaly);

        /*
        Do a fourth-order Schröder iteration of the second kind

        Equation 25 of "A new method for solving the hyperbolic Kepler equation"
        by Baisheng Wu et al.
        Slightly restructured:

        F_1^(4) = F_0 - (
            (6h/h' - 3h^2 h'' / h'^3) /
            (6 - 6h h'' / h'^2 + h^2 h'''/h'^3)
        )

        ...where:
        e_c = eccentricity
        F_0 = initial guess
        h   = e_c sinh(F_0) - F_0 - M_h
        h'  = e_c cosh(F_0) - 1
        h'' = e_c sinh(F_0)
            = h + F_0 + M_h
        h'''= h' + 1

        Rearranging for efficiency:
        h'''= e_c cosh(F_0)
        h'  = h''' - 1
        h'' = e_c sinh(F_0)
        h   = h'' - F_0 - M_h

        Factoring out 1/h':

        let r = 1 / h'

        F_1^(4) = F_0 - (
            (6hr - 3h^2 h'' r^3) / 
            (6 - 6h h'' r^2 + h^2 h''' r^3)
        )

        Since sinh and cosh are very similar algebraically,
        it may be better to calculate them together.

        Paper about Schröder iterations:
        https://doi.org/10.1016/j.cam.2019.02.035
         */

        for _ in 0..NUMERIC_MAX_ITERS {
            let (sinh_eca, cosh_eca) = sinhcosh(ecc_anom);

            let hppp = self.eccentricity * cosh_eca;
            let hp = hppp - 1.0;
            let hpp = self.eccentricity * sinh_eca;
            let h = hpp - ecc_anom - mean_anomaly;

            let h_sq = h * h;
            let r = hp.recip();
            let r_sq = r * r;
            let r_cub = r_sq * r;

            let denominator =
                6.0 - 6.0 * h * hpp * r_sq + h_sq * hppp * r_cub;
            
            if denominator.abs() < 1e-30 || !denominator.is_finite() {
                // dangerously close to div-by-zero, break out
                #[cfg(debug_assertions)]
                eprintln!("Hyperbolic eccentric anomaly solver: denominator is too small or not finite");
                break;
            }

            let numerator = 6.0 * h * r - 3.0 * h_sq * hpp * r_cub;
            let delta = numerator / denominator;

            ecc_anom -= delta;

            if delta.abs() < 1e-12 {
                break;
            }
        }

        return ecc_anom;
    }
}

impl OrbitTrait for CompactOrbit {
    fn get_semi_major_axis(&self) -> f64 {
        return self.periapsis / (1.0 - self.eccentricity);
    }
    
    fn get_semi_minor_axis(&self) -> f64 {
        let semi_major_axis = self.get_semi_major_axis();
        let eccentricity_squared = self.eccentricity * self.eccentricity;
        return semi_major_axis * (1.0 - eccentricity_squared).abs().sqrt();
    }
    
    fn get_linear_eccentricity(&self) -> f64 {
        return self.get_semi_major_axis() - self.periapsis;
    }
    
    fn get_apoapsis(&self) -> f64 {
        if self.eccentricity == 1.0 {
            return f64::INFINITY;
        } else {
            return self.get_semi_major_axis() * (1.0 + self.eccentricity);
        }
    }

    fn set_apoapsis(&mut self, apoapsis: f64) -> Result<(), ApoapsisSetterError> {
        if apoapsis < 0.0 {
            return Err(ApoapsisSetterError::ApoapsisNegative);
        } else if apoapsis < self.periapsis {
            return Err(ApoapsisSetterError::ApoapsisLessThanPeriapsis);
        }

        self.eccentricity = (apoapsis - self.periapsis) / (apoapsis + self.periapsis);

        return Ok(());
    }

    fn set_apoapsis_force(&mut self, apoapsis: f64) {
        let mut apoapsis = apoapsis;
        if apoapsis < self.periapsis && apoapsis > 0.0 {
            (apoapsis, self.periapsis) = (self.periapsis, apoapsis);
        }

        self.eccentricity = (apoapsis - self.periapsis) / (apoapsis + self.periapsis);
    }

    fn get_transformation_matrix(&self) -> Matrix3x2<f64> {
        let mut matrix = Matrix3x2::<f64>::filled_with(0.0);
        {
            let (sin_inc, cos_inc) = self.inclination.sin_cos();
            let (sin_arg_pe, cos_arg_pe) = self.arg_pe.sin_cos();
            let (sin_lan, cos_lan) = self.long_asc_node.sin_cos();
    
            // https://downloads.rene-schwarz.com/download/M001-Keplerian_Orbit_Elements_to_Cartesian_State_Vectors.pdf
            matrix.e11 = cos_arg_pe * cos_lan - sin_arg_pe * cos_inc * sin_lan;
            matrix.e12 = -(sin_arg_pe * cos_lan + cos_arg_pe * cos_inc * sin_lan);
            
            matrix.e21 = cos_arg_pe * sin_lan + sin_arg_pe * cos_inc * cos_lan;
            matrix.e22 = cos_arg_pe * cos_inc * cos_lan - sin_arg_pe * sin_lan;
    
            matrix.e31 = sin_arg_pe * sin_inc;
            matrix.e32 = cos_arg_pe * sin_inc;
        }
        return matrix;
    }

    fn get_eccentric_anomaly(&self, mean_anomaly: f64) -> f64 {
        if self.eccentricity < 1.0 {
            return self.get_eccentric_anomaly_elliptic(mean_anomaly);
        } else {
            return self.get_eccentric_anomaly_hyperbolic(mean_anomaly);
        }
    }

    fn get_true_anomaly_at_eccentric_anomaly(&self, eccentric_anomaly: f64) -> f64 {
        if self.eccentricity < 1.0 {
            // https://en.wikipedia.org/wiki/True_anomaly#From_the_eccentric_anomaly
            let eccentricity = self.eccentricity;
            let (s, c) = eccentric_anomaly.sin_cos();
            let beta = eccentricity / (1.0 + (1.0 - eccentricity * eccentricity).sqrt());
    
            return eccentric_anomaly + 2.0 * 
                (beta * s / (1.0 - beta * c)).atan();
        } else {
            // From the presentation "Spacecraft Dynamics and Control"  
            // by Matthew M. Peet  
            // https://control.asu.edu/Classes/MAE462/462Lecture05.pdf  
            // Slide 25 of 27  
            // Section "The Method for Hyperbolic Orbits"  
            //
            // tan(f/2) = sqrt((e+1)/(e-1))*tanh(H/2)
            // f/2 = atan(sqrt((e+1)/(e-1))*tanh(H/2))
            // f = 2atan(sqrt((e+1)/(e-1))*tanh(H/2))

            return 2.0 * (
                ((self.eccentricity + 1.0) / (self.eccentricity - 1.0)).sqrt() *
                (eccentric_anomaly * 0.5).tanh()
            ).atan();
        }
    }

    fn get_semi_latus_rectum(&self) -> f64 {
        if self.eccentricity == 1.0 {
            return 2.0 * self.periapsis;
        }

        return self.get_semi_major_axis() *
            (1.0 - self.eccentricity * self.eccentricity);
    }

    fn get_altitude_at_angle(&self, true_anomaly: f64) -> f64 {
        return (
            self.get_semi_latus_rectum() /
            (1.0 + self.eccentricity * true_anomaly.cos())
        ).abs();
    }

    fn get_mean_anomaly_at_time(&self, t: f64) -> f64 {
        return t * TAU + self.mean_anomaly;
    }

    fn get_eccentricity         (&self) -> f64 { self.eccentricity }
    fn get_periapsis            (&self) -> f64 { self.periapsis }
    fn get_inclination          (&self) -> f64 { self.inclination }
    fn get_arg_pe               (&self) -> f64 { self.arg_pe }
    fn get_long_asc_node        (&self) -> f64 { self.long_asc_node }
    fn get_mean_anomaly_at_epoch(&self) -> f64 { self.mean_anomaly }

    fn set_eccentricity         (&mut self, value: f64) { self.eccentricity  = value }
    fn set_periapsis            (&mut self, value: f64) { self.periapsis     = value }
    fn set_inclination          (&mut self, value: f64) { self.inclination   = value }
    fn set_arg_pe               (&mut self, value: f64) { self.arg_pe        = value }
    fn set_long_asc_node        (&mut self, value: f64) { self.long_asc_node = value }
    fn set_mean_anomaly_at_epoch(&mut self, value: f64) { self.mean_anomaly  = value }
}

impl From<Orbit> for CompactOrbit {
    fn from(cached: Orbit) -> Self {
        return Self {
            eccentricity: cached.get_eccentricity(),
            periapsis: cached.get_periapsis(),
            inclination: cached.get_inclination(),
            arg_pe: cached.get_arg_pe(),
            long_asc_node: cached.get_long_asc_node(),
            mean_anomaly: cached.get_mean_anomaly_at_epoch()
        };
    }
}

impl Orbit {
    /// Compactify the cached orbit into a compact orbit to increase memory efficiency
    /// while sacrificing calculation speed.
    pub fn compactify(self) -> CompactOrbit {
        CompactOrbit::from(self)
    }
}