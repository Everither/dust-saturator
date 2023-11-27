use nih_plug::prelude::*;

use linear_interpolator::LinearInterpolator;

fn main() {
    nih_export_standalone::<LinearInterpolator>();
}