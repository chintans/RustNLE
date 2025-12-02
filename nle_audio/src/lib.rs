use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioEngine {
    _stream: cpal::Stream,
}

impl AudioEngine {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No output device available"))?;
        let config = device.default_output_config()?;

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into())?,
            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into())?,
            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into())?,
            _ => return Err(anyhow::anyhow!("Unsupported sample format")),
        };

        Ok(Self { _stream: stream })
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<cpal::Stream>
where
    T: cpal::Sample + cpal::FromSample<f32> + cpal::SizedSample,
{
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_silence(data, channels);
        },
        err_fn,
        None,
    )?;

    stream.play()?;
    Ok(stream)
}

fn write_silence<T: cpal::Sample + cpal::FromSample<f32>>(data: &mut [T], _: usize) {
    for sample in data.iter_mut() {
        *sample = T::from_sample(0.0f32);
    }
}

pub fn mix_signals(signals: &[&[f32]]) -> f32 {
    signals.iter().map(|s| s.iter().sum::<f32>()).sum()
}

#[cfg(test)]
mod tests {
    use crate::mix_signals;

    #[test]
    fn test_stereo_summing() {
        let signal_a = vec![0.5, 0.5]; // Left, Right
        let signal_b = vec![0.2, 0.2];

        let mixed = mix_signals(&[&signal_a[..], &signal_b[..]]);
        // 0.5 + 0.5 + 0.2 + 0.2 = 1.4
        assert!((mixed - 1.4f32).abs() < f32::EPSILON);
    }
}
