use nih_plug::prelude::*;
use std::sync::Arc;

use rand::{thread_rng, Rng};

struct Stoner {
    params: Arc<StonerParams>,
    old_buffers: Vec<Vec<f32>>, //buffers are kept contiguously, the oldest sample at the beginning and the latest at the end
    //old buffers always start at the beginning of a buflen
    modulo: u8,
    smaller_modulo: u8,
    dev_urandom: u8,
}

#[derive(Params)]
struct StonerParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "skip"]
    pub skip: IntParam,
    #[id = "every"]
    pub every: IntParam,
    #[id = "multiple"]
    pub multiple: IntParam,
    #[id = "random"]
    pub random: BoolParam,
}

impl Default for Stoner {
    fn default() -> Self {
        Self {
            params: Arc::new(StonerParams::default()),
            old_buffers: vec![],
            modulo: 0,
            smaller_modulo: 0,
            dev_urandom: 0,
        }
    }
}

impl Default for StonerParams {
    fn default() -> Self {
        Self {
            skip: IntParam::new(
                "(max) Buffers in the past to pull from",
                2,
                IntRange::Linear { min: 1, max: 10 },
            )
            .with_unit(" buffers"),
            every: IntParam::new(
                "How often to pull from buffers",
                2,
                IntRange::Linear { min: 1, max: 10 },
            ),
            multiple: IntParam::new(
                "How many frames at a time to play",
                10,
                IntRange::Linear { min: 1, max: 50 },
            )
            .with_unit(" frames"),
            random: BoolParam::new("Whether to randomize which buffer to pull from", true),
        }
    }
}

impl Plugin for Stoner {
    const NAME: &'static str = "Stoner";
    const VENDOR: &'static str = "Devon Sawatsky";
    const URL: &'static str = "https://github.com/novedevo/stoner";
    const EMAIL: &'static str = "devon@nove.dev";

    const VERSION: &'static str = "0.0.1";

    const DEFAULT_INPUT_CHANNELS: u32 = 2;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 2;

    const DEFAULT_AUX_INPUTS: Option<AuxiliaryIOConfig> = None;
    const DEFAULT_AUX_OUTPUTS: Option<AuxiliaryIOConfig> = None;

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        // This works with any symmetrical IO layout
        config.num_input_channels == config.num_output_channels && config.num_input_channels > 0
    }

    fn initialize(
        &mut self,
        bus_config: &BusConfig,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function.
        for _ in 0..bus_config.num_input_channels {
            self.old_buffers
                .push(Vec::with_capacity(buffer_config.max_buffer_size as usize))
        }
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not allocate.
        for buf in self.old_buffers.iter_mut() {
            buf.clear()
        }
        self.modulo = 0;
        self.smaller_modulo = 0;
    }

    fn process(
        &mut self,
        frame: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext,
    ) -> ProcessStatus {
        let random = self.params.random.value();
        let every = self.params.every.value();
        let skip = self.params.skip.value();
        let mul = self.params.multiple.value();
        for channel in 0..frame.channels() {
            //the first element of a channel is the oldest sample
            let ob = &mut self.old_buffers[channel];
            // let mut dividend = ob.len();

            let ch = &mut frame.as_slice()[channel];
            let max_old_size = skip as usize * ch.len() * mul as usize;

            ob.extend_from_slice(ch);

            let start = if random {
                self.dev_urandom as usize * ch.len()
            } else {
                0
            };

            if self.modulo == 0 && ob.len() >= max_old_size {
                ch.copy_from_slice(&ob[start..ch.len() + start]);
                *ob = ob[ob.len() - max_old_size..].to_vec();
            }
        }

        if self.smaller_modulo == 0 {
            self.dev_urandom = thread_rng().gen_range(0..skip as u8);
            self.modulo += 1;
            self.modulo %= every as u8;
        }
        self.smaller_modulo += 1;
        self.smaller_modulo %= mul as u8;
        ProcessStatus::Normal
    }
}

impl Vst3Plugin for Stoner {
    const VST3_CLASS_ID: [u8; 16] = *b"AVerySoberStoner";

    // And don't forget to change these categories, see the docstring on `VST3_CATEGORIES` for more
    // information
    const VST3_CATEGORIES: &'static str = "Fx";
}

nih_export_vst3!(Stoner);
