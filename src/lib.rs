//! # interp3d
//! 
//! This crate introduces a struct that can interpolate a 3d arbitrarily spaced data set.

mod utils;

use crate::utils::Dir;

pub use crate::utils::{
    GridSpacing,
    DataGenConfSingle,
    DataGenConf,
    Type
};

use std::f64::consts::LN_2;

/// This is the main interpolator struct.  
/// Will need to be set up before use. Either generate data (see function [`Self::generate_data()`]) or load from file (see function [`Self::import_data()`]).
#[derive(Default)]
pub struct Interp3D {
    nx: usize,
    ny: usize,
    nz: usize,
    x: Vec<f64>,
    y: Vec<f64>,
    z: Vec<f64>,
    tx: (f64, f64),
    ty: (f64, f64),
    tz: (f64, f64),
    data: Vec<f64>
}

impl Interp3D {
    fn index(&self, i: usize, j: usize, k: usize) -> usize {
        i*self.ny*self.nz + j*self.nz + k
    }

    fn grid_point_pos(dir: Dir, i: isize, conf: &DataGenConf) -> f64 {
        let conf: DataGenConfSingle = match dir {
            Dir::X => conf.x,
            Dir::Y => conf.y,
            Dir::Z => conf.z
        };

        match conf.spacing {
            GridSpacing::Exponential(k) if k != 0.0 => conf.min + (conf.max-conf.min) * ( (LN_2*(i as f64)/((conf.n as isize - 4) as f64)*k).exp() ) / (2.0f64.powf(k)-1.0),
            _ => conf.min+(conf.max-conf.min)*(i as f64)/((conf.n as isize - 4) as f64)
        }
    }

    fn setup(&mut self, conf: &DataGenConf) {
        self.nx = conf.x.n+3;
        self.ny = conf.y.n+3;
        self.nz = conf.z.n+3;

        if self.nx < 2+3 || self.ny < 2+3 || self.nz < 2+3 {
            panic!("Number of points too low (at least 2 per direction required)");
        }

        self.x = Vec::with_capacity(self.nx);
        self.y = Vec::with_capacity(self.ny);
        self.z = Vec::with_capacity(self.nz);
        for i in 0..self.nx {
            self.x.push(Self::grid_point_pos(Dir::X, i as isize - 1, &conf));
        }
        for i in 0..self.ny {
            self.y.push(Self::grid_point_pos(Dir::Y, i as isize - 1, &conf));
        }
        for i in 0..self.nz {
            self.z.push(Self::grid_point_pos(Dir::Z, i as isize - 1, &conf));
        }
        
        self.data = Vec::with_capacity(self.nx*self.ny*self.nz);
        for _ in 0..self.nx*self.ny*self.nz {
            self.data.push(0.0);
        }

        self.x[0] = 2.0*self.x[1] - self.x[2];
        self.x[self.nx-2] = 2.0*self.x[self.nx-3] - self.x[self.nx-4];
        self.x[self.nx-1] = 2.0*self.x[self.nx-2] - self.x[self.nx-3];
        
        self.y[0] = 2.0*self.y[1] - self.y[2];
        self.y[self.ny-2] = 2.0*self.y[self.ny-3] - self.y[self.ny-4];
        self.y[self.ny-1] = 2.0*self.y[self.ny-2] - self.y[self.ny-3];

        self.z[0] = 2.0*self.z[1] - self.z[2];
        self.z[self.nz-2] = 2.0*self.z[self.nz-3] - self.z[self.nz-4];
        self.z[self.nz-1] = 2.0*self.z[self.nz-2] - self.z[self.nz-3];
    }

    fn set_data_outermost(&mut self) {
        for i in 0..self.nx {
            for j in 0..self.ny {
                for k in 0..self.nz {
                    let mut i_temp = i;
                    if i_temp == 0 {
                        i_temp = 1;
                    } else if i_temp > self.nx-3 {
                        i_temp = self.nx-3;
                    }

                    let mut j_temp = j;
                    if j_temp == 0 {
                        j_temp = 1;
                    } else if j_temp > self.nx-3 {
                        j_temp = self.ny-3;
                    }

                    let mut k_temp = k;
                    if k_temp == 0 {
                        k_temp = 1;
                    } else if k_temp > self.nz-3 {
                        k_temp = self.nz-3;
                    }

                    if i != i_temp || j != j_temp || k != k_temp {
                        let index = self.index(i, j, k);
                        self.data[index] = self.data[self.index(i_temp, j_temp, k_temp)];
                    }
                }
            }
        }
    }

    /// Use this to generate new data for the interpolator.
    /// The data generated like this can also be written to file with export_data.
    /// The passed closure could, for example, call a computationally intensive function.
    /// 
    /// # Example
    /// 
    /// ```
    /// use interp3d::*;
    /// 
    /// let mut ip: Interp3D = Interp3D::default();
    /// 
    /// let config = DataGenConfSingle {
    ///     n: 11,
    ///     min: 0.0,
    ///     max: 10.0,
    ///     spacing: GridSpacing::Exponential(1.0)
    /// };
    /// // using the same config for all 3 directions
    /// let config = DataGenConf {
    ///     x: config,
    ///     y: config,
    ///     z: config
    /// };
    /// 
    /// let outside_val = 1.0;
    /// let mut mutable_outside_val = 0;
    /// 
    /// let f = |x: f64, y: f64, z: f64| -> f64 { mutable_outside_val += 1; ((-x*x - y*y - z*z)/5.0).exp() + outside_val };
    /// 
    /// ip.generate_data(f, &config);
    /// // ip is now set up for use
    /// ```
    pub fn generate_data<F>(&mut self, mut f: F, conf: &DataGenConf/*, monitor_progress: bool*/)
    where F: FnMut(f64, f64, f64) -> f64 {
        self.setup(&conf);

        //MARK: -add multithreading
        for i in 1..self.nx-2 {
            for j in 1..self.ny-2 {
                for k in 1..self.nz-2 {
                    let index = self.index(i, j, k);
                    self.data[index] = f(self.x[i], self.y[j], self.z[k]);
                }
            }
        }
        self.set_data_outermost();
    }

    /// This allows a construction, similar to the example for [`Self::generate_data()`], but here we construct and set up the object directly using the passed config.
    ///  
    /// # Example
    /// 
    /// ```
    /// use interp3d::*;
    /// 
    /// let config = DataGenConfSingle {
    ///     n: 11,
    ///     min: 0.0,
    ///     max: 10.0,
    ///     spacing: GridSpacing::Exponential(1.0),
    /// };
    /// // using the same config for all 3 directions
    /// let config = DataGenConf {
    ///     x: config,
    ///     y: config,
    ///     z: config
    /// };
    /// 
    /// let outside_val = 1.0;
    /// let mut mutable_outside_val = 0;
    /// 
    /// let f = |x: f64, y: f64, z: f64| -> f64 { mutable_outside_val += 1; ((-x*x - y*y - z*z)/5.0).exp() + outside_val };
    /// 
    /// let ip: Interp3D = Interp3D::from_config(f, &config); 
    /// // ip is now set up for use
    /// ```
    pub fn from_config<F>(f: F, conf: &DataGenConf) -> Self
    where F: FnMut(f64, f64, f64) -> f64 {
        let mut ip: Interp3D = Interp3D::default();
        ip.generate_data(f, conf);

        ip
    }

    /// You can also make Interp3D load from file directly.  
    /// The data in the file can stem from either a previous export after data generation or you can format you own existing data for use with this interpolator.
    /// Information on the data format can be found at <github.com/y-hoffmann/interp3d> or <crates.io/interp3d>.
    /// 
    /// # Example
    /// ```
    /// use inter3p::*;
    /// 
    /// let file = String::from("some/file.ip3d"); // file extension can be whatever (also nothing)
    /// let ip = Interp3D::from_file(file);
    /// // ip is now set up for use
    /// ```
    pub fn from_file<F>(file: &str) -> Self {
        let ip: Interp3D = Interp3D::default();
        //ip.import_data(file);

        ip
    }

    /// This will export a loaded data set and grid to file.
    pub fn export_data(file: &str) {

    }
}