use butterworth::{Cutoff, Filter};
use core::f32;
use nih_plug::prelude::*;
use std::sync::Arc;

struct StarlightVocoder {
    params: Arc<StlVocoderParams>,
    // buffer_config: BufferConfig,
}

/// The [`Params`] derive macro gathers all of the information needed for the wrapper to know about
/// the plugin's parameters, persistent serializable fields, and nested parameter groups. You can
/// also easily implement [`Params`] by hand if you want to, for instance, have multiple instances
/// of a parameters struct for multiple identical oscillators/filters/envelopes.
#[derive(Params)]
struct StlVocoderParams {
    #[id = "low_freq_cutoff"]
    pub low_freq_cutoff: FloatParam,
    #[id = "high_freq_cutoff"]
    pub high_freq_cutoff: FloatParam,
    #[id = "bands"]
    pub bands: IntParam,
}

impl Default for StarlightVocoder {
    fn default() -> Self {
        Self {
            params: Arc::new(StlVocoderParams::default()),
            // buffer_config: BufferConfig {
            //     sample_rate: 1.0,
            //     min_buffer_size: None,
            //     max_buffer_size: 0,
            //     process_mode: ProcessMode::Realtime,
            // },
        }
    }
}

impl Default for StlVocoderParams {
    fn default() -> Self {
        Self {
            low_freq_cutoff: FloatParam::new(
                "Formant lower end",
                300f32,
                FloatRange::Linear {
                    min: (0f32),
                    max: (20000_f32),
                },
            ),
            high_freq_cutoff: FloatParam::new(
                "Formant upper end",
                3400f32,
                FloatRange::Linear {
                    min: (0f32),
                    max: (20000_f32),
                },
            ),
            bands: IntParam::new("Number of Bands", 20, IntRange::Linear { min: 1, max: 256 }),
        }
    }
}

impl Plugin for StarlightVocoder {
    const NAME: &'static str = "StarlightSimpleVocoder";
    const VENDOR: &'static str = "starlight_caffeine";
    // You can use `env!("CARGO_PKG_HOMEPAGE")` to reference the homepage field from the
    // `Cargo.toml` file here
    const URL: &'static str = "https://youtu.be/dQw4w9WgXcQ";
    const EMAIL: &'static str = "info@example.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),

            aux_input_ports: &[],
            aux_output_ports: &[],

            // Individual ports and the layout as a whole can be named here. By default these names
            // are generated as needed. This layout will be called 'Stereo', while the other one is
            // given the name 'Mono' based no the number of input and output channels.
            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    // Setting this to `true` will tell the wrapper to split the buffer up into smaller blocks
    // whenever there are inter-buffer parameter changes. This way no changes to the plugin are
    // required to support sample accurate automation and the wrapper handles all of the boring
    // stuff like making sure transport and other timing information stays consistent between the
    // splits.
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

    // This plugin doesn't need any special initialization, but if you need to do anything expensive
    // then this would be the place. State is kept around when the host reconfigures the
    // plugin. If we do need special initialization, we could implement the `initialize()` and/or
    // `reset()` methods

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            // let gain = self.params.gain.smoothed.next();

            // for sample in channel_samples {
            //     *sample *= gain;
            // }

            let mut samples_f32: Vec<&mut f32> = channel_samples.into_iter().collect();
            let q: f64 = 300f64;
            let bp_filter = Filter::new(
                1,
                // self.buffer_config.sample_rate.into(),
                1.0,
                Cutoff::BandPass(
                    self.params.low_freq_cutoff.value().into(),
                    f64::from(self.params.low_freq_cutoff.value()) + q,
                ),
            )
            .unwrap();

            let samples_f64: Vec<f64> = samples_f32.iter().map(|f| f64::from(**f)).collect();

            let filter_res: Vec<f32> = bp_filter
                .bidirectional(&samples_f64)
                .unwrap()
                .iter()
                .map(|f| *f as f32)
                .collect();

            for (i, f) in filter_res.iter().enumerate() {
                *samples_f32[i] = *f;
            }
        }

        ProcessStatus::Normal
    }

    // This can be used for cleaning up special resources like socket connections whenever the
    // plugin is deactivated. Most plugins won't need to do anything here.
    fn deactivate(&mut self) {}
}

impl ClapPlugin for StarlightVocoder {
    const CLAP_ID: &'static str = "com.starlight_caffeine.vocoder";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple vocoder");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for StarlightVocoder {
    const VST3_CLASS_ID: [u8; 16] = *b"StarlightVocoder";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Filter];
}

nih_export_clap!(StarlightVocoder);
nih_export_vst3!(StarlightVocoder);
