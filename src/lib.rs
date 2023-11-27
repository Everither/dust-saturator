use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

mod editor;

// The maximum deviation from the actual sample position
pub const MAX_AMOUNT: f32 = 100.0;
// The minimum deviation from the actual sample position
pub const MIN_AMOUNT: f32 = 0.0;
// Minimum value for tolerance
pub const MIN_TOLERANCE: f32 = 0.0;
// Maximum value for tolerance
pub const MAX_TOLERANCE: f32 = 1.0;
// Scuffed way of marking a sample
pub const MARKER: f32 = 100.0;

pub struct LinearInterpolator {
    params: Arc<LinearInterpolatorParams>,
    aux_buffer: Vec<Vec<f32>>,
    gradient: Vec<f32>,
    c: Vec<f32>,
    carry_over: f32

}

#[derive(Params)]
struct LinearInterpolatorParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "amount"]
    pub amount: FloatParam,

    #[id = "tolerance"]
    pub tolerance: FloatParam,

    #[id = "dither"]
    pub dither: BoolParam
}

impl Default for LinearInterpolator {
    fn default() -> Self {
        Self {
            params: Arc::new(LinearInterpolatorParams::default()),
            aux_buffer: vec![],
            gradient: vec![],
            c: vec![],
            carry_over: 0.0
        }
    }
}

impl Default for LinearInterpolatorParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            amount: FloatParam::new(
                "Amount",
                0.0,
                FloatRange::Linear {
                    min: MIN_AMOUNT, 
                    max: MAX_AMOUNT 
                }
            ),

            tolerance: FloatParam::new(
                "Tolerance",
                1.0,
                FloatRange::Linear {
                    min: MIN_TOLERANCE, 
                    max: MAX_TOLERANCE 
                }
            ),

            dither: BoolParam::new(
                "Dither",
                false
            )
        }
    }
}

impl Plugin for LinearInterpolator {
    const NAME: &'static str = "Linear Interpolator";
    const VENDOR: &'static str = "Evr!";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "everither.every@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];


    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Set the latency (this is assuming buffer size is fixed)
        // context.set_latency_samples(buffer_config.max_buffer_size);

        true
    }

    fn reset(&mut self) {
        // Unused at the moment
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let buffer_slice = buffer.as_slice();
        let amount_float: f32 = self.params.amount.smoothed.next();
        let tolerance: f32 = self.params.tolerance.smoothed.next();

        // Scuffed enumeration
        let mut channel_number = 0;

        // Process left and right channels
        for channel_samples in buffer_slice {

            if self.aux_buffer.len() <= channel_number {
                self.aux_buffer.push(vec![]);
                self.c.push(0.0);
                self.gradient.push(1.0);
            }

            for i in 0..channel_samples.len() {
                if self.aux_buffer[channel_number].len() <= MAX_AMOUNT as usize {
                    // Upon first launching the plugin, the aux buffer is not (fully) populated yet
                    self.aux_buffer[channel_number].push(channel_samples[i])
                } else {
                    // The actual processing
                    if !(self.aux_buffer[channel_number][0] == MARKER) {
                        
                        // Get amount + handle dithering
                        let mut amount = amount_float as usize;
                        if self.params.dither.value() {
                            self.carry_over += amount_float % 1.0;
                            if self.carry_over > 1.0 {
                                self.carry_over -= 1.0;
                                amount += 1;
                            }
                        }

                        // Calculate initial position (c) and gradient
                        let c = self.aux_buffer[channel_number][0];
                        let gradient = (self.aux_buffer[channel_number][amount] - self.c[channel_number]) / (amount as f32);    
                        
                        let mut end_point = amount;

                        // Line function
                        for i in 1..amount {
                            let approximation = gradient * (i as f32) + c;
                            let actual = self.aux_buffer[channel_number][i];
                            // Compute error
                            if (approximation - actual).abs() > tolerance {
                                end_point = i;
                                break;
                            }
                        }

                        // Calculate initial position (c) and gradient (2nd iteration)
                        self.c[channel_number] = self.aux_buffer[channel_number][0];
                        self.gradient[channel_number] = (self.aux_buffer[channel_number][end_point] - self.c[channel_number]) / (end_point as f32);                      

                        // Set everything between initial and end point to MARKER
                        for i in 1..end_point {
                            self.aux_buffer[channel_number][i] = MARKER;
                        }
                    } else {
                        self.c[channel_number] += self.gradient[channel_number];
                    }

                    // Append new sample + Serve oldest sample
                    self.aux_buffer[channel_number].push(channel_samples[i]);
                    self.aux_buffer[channel_number].remove(0);

                    // Apply to current buffer
                    channel_samples[i] = self.c[channel_number];

                }
            }

            // Increment channel number
            channel_number += 1;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for LinearInterpolator {
    const CLAP_ID: &'static str = "com.your-domain.Linear-Interpolator";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A short description of your plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for LinearInterpolator {
    const VST3_CLASS_ID: [u8; 16] = *b"LineArinTerpOlat";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(LinearInterpolator);
nih_export_vst3!(LinearInterpolator);
