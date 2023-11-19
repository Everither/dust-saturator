use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

mod editor;

// The maximum deviation from the actual sample position
pub const MAX_AMOUNT: usize = 64;
// Max value for curve parameter
pub const MAX_CURVE: f32 = 1.0;
// Min value for curve parameter
pub const MIN_CURVE: f32 = 0.5;

pub struct DustSaturator {
    params: Arc<DustSaturatorParams>,
    previous_buffer: Vec<Vec<f32>>
}

#[derive(Params)]
struct DustSaturatorParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "amount"]
    pub amount: IntParam,

    #[id = "curve"]  
    pub curve: FloatParam
}

impl Default for DustSaturator {
    fn default() -> Self {
        Self {
            params: Arc::new(DustSaturatorParams::default()),
            previous_buffer: vec![]
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
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        // Set the latency (this is assuming buffer size is fixed)
        context.set_latency_samples(buffer_config.max_buffer_size);

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
        let amount = self.params.amount.smoothed.next();
        let curve = self.params.curve.smoothed.next();

        // Make deep copy of the current buffer
        let mut buffer_deep_copy = vec![];      

        // Scuffed enumeration
        let mut channel_number = 0;

        // Process left and right channels
        for channel_samples in buffer_slice {

            // Copy of channel samples
            let mut channel_copy = vec![];

            // Copy the samples
            for i in 0..channel_samples.len() {
                channel_copy.push(channel_samples[i]);
            }

            // No processing required for the first ever buffer
            if !self.previous_buffer.is_empty() {
                // The actual processing
                for i in 0..self.previous_buffer[channel_number].len() {
                    if i >= channel_samples.len() {
                        continue
                    }
                    let idx = (
                        (((self.previous_buffer[channel_number][i]*(amount as f32)).abs()).powf(curve)) * ((amount as f32).powf(MAX_CURVE-curve))
                    ) as usize;
                    if i+idx < self.previous_buffer[channel_number].len() {
                        // When the future sample does not exceed the bounds of the current buffer
                        channel_samples[i] = self.previous_buffer[channel_number][i+idx];
                    } else {
                        // When the future sample is reaching into the next buffer
                        channel_samples[i] = channel_copy[i+idx -  self.previous_buffer[channel_number].len()];
                    }    
                }
            }

            // Increment channel number
            channel_number += 1;

            // Making deep copy
            buffer_deep_copy.push(channel_copy);
        }

        // Store the current buffer for the next processing cycle to use
        self.previous_buffer = buffer_deep_copy;

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
