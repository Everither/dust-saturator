use nih_plug::prelude::*;

use dust_saturator::DustSaturator;

fn main() {
    nih_export_standalone::<DustSaturator>();
}