use nih_plug::prelude::*;
use std::sync::Arc;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct Stoner {
    params: Arc<StonerParams>,
    old_buffers: Vec<Vec<f32>>, //buffers are kept in contiguous order, the oldest sample at the beginning and the latest at the end
    //old buffers always start at the beginning of a buflen
    samples_till_next_throwback: isize,
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
    #[id = "buflen"]
    pub buflen: IntParam,
}

impl Default for Stoner {
    fn default() -> Self {
        Self {
            params: Arc::new(StonerParams::default()),
            old_buffers: vec![],
            samples_till_next_throwback: 0,
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
            buflen: IntParam::new(
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

        self.samples_till_next_throwback =
            self.params.buflen.value() as isize * self.params.skip.value() as isize;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext,
    ) -> ProcessStatus {
        let every = self.params.every.value() as u8;
        let bl = self.params.buflen.value() as usize;
        for channel in 0..buffer.channels() {
            //the first element of a channel is the oldest sample
            // let max_old_size = self.params.skip.value() as usize * bl;
            let ob = &mut self.old_buffers[channel];
            // let mut dividend = ob.len();

            let ch = &mut buffer.as_slice()[channel];
            let vech = ch.to_vec();

            let mut processed = 0;
            while processed < ch.len() {
                if self.samples_till_next_throwback < 0 {
                    let samp = (-self.samples_till_next_throwback as usize)
                        .min(ch.len() - processed)
                        .min(ob.len());

                    ch[processed..samp + processed].copy_from_slice(&ob[..samp]);

                    self.samples_till_next_throwback += samp as isize;
                    processed += samp;
                    *ob = ob[samp..].to_vec()
                } else {
                    let samp =
                        (self.samples_till_next_throwback as usize).min(ch.len() - processed);

                    ob.extend_from_slice(&ch[processed..samp + processed]);
                    self.samples_till_next_throwback -= samp as isize;
                }
            }

            // ob.extend_from_slice(ch);

            // while dividend < ob.len() {
            //     if dividend < max_old_size {
            //         let neg_offset = max_old_size as isize - dividend as isize;
            //         match ch.len() as isize {
            //             o if -o < neg_offset => todo!(),
            //             u if -u > neg_offset => todo!(),
            //             e => todo!(),
            //         }
            //     } else {
            //         let offset = dividend % max_old_size;
            //         if offset / bl > 0 {
            //             todo!() // more than a chunk of samples to deal with
            //         } else if offset > 0 {
            //             todo!()
            //         } else {
            //             todo!()
            //         }
            //     }
            // }

            //copy one or many buflens from the old buffers into the current buffer if it's time to do that (here)
            //then trim that many buflens from the front of the old buffer

            // remove whole numbers of buflens from the start of the old buffers
            // let need_removing = ob.len().saturating_sub(max_old_size) / bl;
            // if need_removing > 0 {
            //     *ob = ob[need_removing * bl..].to_vec()
            // }
        }

        ProcessStatus::Normal
    }
}

fn get_oldest_chunk(buf: &[f32], buflen: usize) -> &[f32] {
    &buf[..buflen]
}

fn without_first_chunk(buf: &[f32], buflen: usize) -> &[f32] {
    &buf[buflen..]
}

impl Vst3Plugin for Stoner {
    const VST3_CLASS_ID: [u8; 16] = *b"AVerySoberStoner";

    // And don't forget to change these categories, see the docstring on `VST3_CATEGORIES` for more
    // information
    const VST3_CATEGORIES: &'static str = "Fx";
}

nih_export_vst3!(Stoner);
