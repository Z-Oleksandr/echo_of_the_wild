use hound::{WavReader, WavWriter};

pub fn get_volume_level(file_path: &str) -> Result<f32, hound::Error> {
    let mut reader = WavReader::open(file_path)?;
    let rms_amplitude: f32 = reader
        .samples::<i16>()
        .map(|s| {
            let sample = s.unwrap() as f32;
            sample * sample
        })
        .sum::<f32>()
        .sqrt();
    Ok(rms_amplitude)
}

pub fn adjust_volume(input_file: &str, output_file: &str, volume_factor: f32) -> Result<(), hound::Error> {
    let reader = WavReader::open(input_file)?;
    let spec = reader.spec();
    let mut writer = WavWriter::create(output_file, spec)?;

    for sample in reader.into_samples::<i16>() {
        let mut sample = sample?;
        sample = (sample as f32 * volume_factor) as i16;
        writer.write_sample(sample)?;
    }

    writer.finalize()?;

    Ok(())
}
