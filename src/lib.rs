use nih_plug::prelude::*;
use std::sync::Arc;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct Stoner {
    params: Arc<StonerParams>,
    old_buffers: Vec<Vec<f32>>, //buffers are kept in contiguous order, the oldest sample at the beginning and the latest at the end
    modulo: u8,
    passthru: bool,
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
    #[id = "buffer size"]
    pub buffer_size: IntParam,
}

impl Default for Stoner {
    fn default() -> Self {
        Self {
            params: Arc::new(StonerParams::default()),
            old_buffers: vec![],
            modulo: 0,
            passthru: true,
        }
    }
}

impl Default for StonerParams {
    fn default() -> Self {
        Self {
            skip: IntParam::new(
                "Buffers in the past to pull from",
                2,
                IntRange::Linear { min: 1, max: 10 },
            )
            .with_unit(" buffers"),
            every: IntParam::new(
                "How often to pull from buffers",
                2,
                IntRange::Linear { min: 2, max: 10 },
            ),
            buffer_size: IntParam::new(
                "Buffer size",
                12025,
                IntRange::Linear {
                    min: 64,
                    max: 88200,
                },
            )
            .with_unit(" samples"),
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
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        for _ in 0..bus_config.num_input_channels {
            self.old_buffers
                .push(Vec::with_capacity(buffer_config.max_buffer_size as usize))
        }
        true
    }

    fn reset(&mut self) {
        for buf in self.old_buffers.iter_mut() {
            buf.clear()
        }
        self.modulo = 0;
        self.passthru = true;
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext,
    ) -> ProcessStatus {
        let every = self.params.every.value() as u8;
        let bs = self.params.buffer_size.value() as usize;
        for channel in 0..buffer.channels() {
            //the first element of a channel is the oldest sample
            let max_old_size = self.params.skip.value() as usize * bs;
            let ob = &mut self.old_buffers[channel];
            let dividend = ob.len();
            let offset = if dividend < max_old_size {
                max_old_size as isize - dividend as isize
            } else {
                (dividend % max_old_size) as isize
            };

            let ch = &mut buffer.as_slice()[channel];

            self.old_buffers[channel].extend_from_slice(ch);

            match offset {
                p if p > 0 => todo!(),
                n if n < 0 => match ch.len() as isize {
                    o if -o < n => todo!(),
                    u if -u > n => todo!(),
                    e => todo!(),
                },
                z => todo!(),
            }

            //copy samples from the old buffers into the current buffer if it's time to do that (here)
            //then trim that many samples from the front of the old buffer

            // remove whole numbers of buflens from the start of the old buffers
            let need_removing = self.old_buffers[channel].len().saturating_sub(max_old_size) / bs;
            if need_removing > 0 {
                self.old_buffers[channel] = self.old_buffers[channel][need_removing..].to_vec()
            }
        }

        self.modulo = (self.modulo + 1) % every;

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
