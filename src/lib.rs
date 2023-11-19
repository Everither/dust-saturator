use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

mod editor;

// The maximum deviation from the actual sample position
pub const MAX_AMOUNT: usize = 64;
pub const MAX_CURVE: f32 = 1.0;
pub const MIN_CURVE: f32 = 0.5;

pub struct DustSaturator {
    params: Arc<DustSaturatorParams>,
    previous_buffer: Vec<Vec<f32>>
}

#[derive(Params)]
struct DustSaturatorParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
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
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
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

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
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

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
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
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        context.set_latency_samples(buffer_config.max_buffer_size);

        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
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

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for DustSaturator {
    const VST3_CLASS_ID: [u8; 16] = *b"Exactly16Chars!!";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(DustSaturator);
nih_export_vst3!(DustSaturator);
