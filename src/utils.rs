//! Contains some utils for the implementation of the interpolator.

use std::f64::consts::PI as PI;

/// Used for specification of the direction
pub enum Dir {
    X,
    Y,
    Z
}

/// Defines the grid spacing for data generation
#[derive(Copy, Clone)]
pub enum GridSpacing {
    Linear,
    Exponential(f64)
}

/// Configure how to set the data point positions in 1d (for example along X)
/// 
/// ``GridSpacing::Exponential(k)`` describes how the points are distributed.  
/// ``k`` > 0 decrease the density of points towards the upper limit (``max``). This means higher precision towards the lower end (``min``) (usually what you want).  
/// ``k`` < 0 increase the density of points towards the upper limit. (<- This is coincidental and I've never come across a use case but it's there if you need it.)  
/// The larger the absolute value of ``k`` the greater the decrease/increase in density, with ``k = 0.0`` being equivalent to the linear case.  
///   
/// As an example, for ``k = 8.0``, half of all points lie within the first ~7/8 of the specified range. Analogously, for ``k = -8.0``, half of all points will be in the last ~7/8 of the range.  
///   
/// I found that ``k = 8.0`` gives very good low-end precision but also has enough high-end precision to strike a good balance. The best choice will starkly depend on the specific use case, however.
#[derive(Copy, Clone)]
pub struct DataGenConfSingle {
    /// number of points
    pub n: usize,
    /// minimum of the range in which the points lie
    pub min: f64,
    /// maximum of the range
    pub max: f64,
    /// describes point density along that range
    pub spacing: GridSpacing,
}

/// There is nothing particular about these default values. They are just what I usually use for the calculation that I wrote this lib for.
impl Default for DataGenConfSingle {
    fn default() -> DataGenConfSingle {
        DataGenConfSingle {
            n: 300,
            min: 0.0,
            max: 15.0,
            spacing: GridSpacing::Exponential(8.0)
        }
    }
}

/// Combines 3 single direction configs
#[derive(Copy, Clone)]
pub struct DataGenConf {
    pub x: DataGenConfSingle,
    pub y: DataGenConfSingle,
    pub z: DataGenConfSingle
}

impl Default for DataGenConf {
    fn default() -> DataGenConf {
        DataGenConf {
            x: DataGenConfSingle::default(),
            y: DataGenConfSingle::default(),
            z: DataGenConfSingle {
                n: 40,
                min: 0.0,
                max: PI,
                spacing: GridSpacing::Linear,
            }
        }
    }
}

/// Used to define whether to use bicubic-unilinear or tricubic interpolation
pub enum Type {
    BicubicUnilinear,
    Tricubic
}