use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::{sync::Arc, u128::MAX};

mod editor;

// The maximum deviation from the actual sample position
pub const MAX_AMOUNT: usize = 100;
// The minimum deviation from the actual sample position
pub const MIN_AMOUNT: usize = 0;
// Max value for curve parameter
pub const MAX_CURVE: f32 = 1.0;
// Min value for curve parameter
pub const MIN_CURVE: f32 = 0.1;

pub struct DustSaturator {
    params: Arc<DustSaturatorParams>,
    aux_buffer: Vec<Vec<f32>>
}

#[derive(Params)]
struct DustSaturatorParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "amount"]
    pub amount: IntParam,

    #[id = "curve"]  
    pub curve: FloatParam,

    #[id = "invert"]  
    pub invert: BoolParam
}

impl Default for DustSaturator {
    fn default() -> Self {
        Self {
            params: Arc::new(DustSaturatorParams::default()),
            aux_buffer: vec![]
        }
    }
}

impl Default for DustSaturatorParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            amount: IntParam::new(
                "Amount",
                20,
                IntRange::Linear {
                    min: 0,
                    max: MAX_AMOUNT as i32,
                }
            ),

            curve: FloatParam::new(
                "Curve",
                1.0,
                FloatRange::Linear { min: MIN_CURVE, max: MAX_CURVE }
            ),

            invert: BoolParam::new(
                "Invert",
                false
            )
        }
    }
}

impl Plugin for DustSaturator {
    const NAME: &'static str = "Dust Saturator";
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
        let amount: f32 = self.params.amount.smoothed.next() as f32;
        let curve: f32 = self.params.curve.smoothed.next();
        let invert: bool = self.params.invert.value();

        // Scuffed enumeration
        let mut channel_number = 0;

        // Process left and right channels
        for channel_samples in buffer_slice {

            if self.aux_buffer.len() <= channel_number {
                self.aux_buffer.push(vec![]);
            }

            for i in 0..channel_samples.len() {
                if self.aux_buffer[channel_number].len() <= MAX_AMOUNT as usize {
                    // Upon first launching the plugin, the aux buffer is not (fully) populated yet
                    self.aux_buffer[channel_number].push(channel_samples[i])
                } else {
                    // The actual processing

                    // Calculate new index
                    let mut idx = (
                        (((self.aux_buffer[channel_number][0]*amount).abs()).powf(curve)) * (amount.powf(MAX_CURVE-curve))
                    ) as usize;

                    // Cap the index at max amount
                    if idx > MAX_AMOUNT {
                        idx = MAX_AMOUNT;
                    }

                    // Invert when specified
                    if invert {
                        idx = MAX_AMOUNT - idx;
                    }

                    // Append new sample + Serve oldest sample
                    self.aux_buffer[channel_number].push(channel_samples[i]);
                    self.aux_buffer[channel_number].remove(0);

                    // Apply the index
                    channel_samples[i] = self.aux_buffer[channel_number][idx];
                }
            }

            // Increment channel number
            channel_number += 1;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for DustSaturator {
    const CLAP_ID: &'static str = "com.your-domain.Dust-Saturator";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A short description of your plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for DustSaturator {
    const VST3_CLASS_ID: [u8; 16] = *b"Exactly16Chars!!";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(DustSaturator);
nih_export_vst3!(DustSaturator);
