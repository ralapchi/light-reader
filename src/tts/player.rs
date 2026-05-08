use rodio::{OutputStream, Sink};

/// Default sample rate for Xiaomi PCM16 output.
const PCM_SAMPLE_RATE: u32 = 24000;
/// Default channel count for Xiaomi PCM16 output.
const PCM_CHANNELS: u16 = 1;

pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
}

#[allow(dead_code)]
impl AudioPlayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        Ok(Self {
            _stream: stream,
            sink,
        })
    }

    /// Append a pre-encoded audio source (WAV, MP3, etc.) to the playback queue.
    /// rodio auto-detects the format from the byte content.
    pub fn append(&self, data: Vec<u8>) -> Result<(), String> {
        let cursor = std::io::Cursor::new(data);
        let source =
            rodio::Decoder::new(cursor).map_err(|e| format!("音频解码失败: {}", e))?;
        self.sink.append(source);
        Ok(())
    }

    /// Append raw PCM16 audio data, wrapping it in a WAV header for rodio.
    ///
    /// The Xiaomi TTS API returns raw PCM16 samples without a container header.
    /// This method constructs a proper WAV header so rodio can decode it.
    pub fn append_pcm16(&self, pcm_data: Vec<u8>) -> Result<(), String> {
        let wav = wrap_pcm16_as_wav(&pcm_data, PCM_SAMPLE_RATE, PCM_CHANNELS);
        self.append(wav)
    }

    /// Stop playback and clear the queue.
    pub fn stop(&self) {
        self.sink.stop();
    }

    /// Pause playback (can be resumed with play()).
    pub fn pause(&self) {
        self.sink.pause();
    }

    /// Resume playback after pause.
    pub fn play(&self) {
        self.sink.play();
    }

    /// Whether the sink is paused.
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    /// Whether the sink has no more audio to play.
    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    /// Number of audio sources still queued.
    pub fn len(&self) -> usize {
        self.sink.len()
    }
}

/// Wrap raw PCM16 data in a valid WAV header.
///
/// Produces a standard RIFF/WAVE container suitable for rodio's decoder.
/// PCM format (1), 16-bit little-endian samples.
pub fn wrap_pcm16_as_wav(pcm_data: &[u8], sample_rate: u32, channels: u16) -> Vec<u8> {
    let bits_per_sample: u16 = 16;
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * block_align as u32;
    let data_size = pcm_data.len() as u32;
    let file_size = 36 + data_size; // total - 8 (RIFF header size field)

    let mut wav = Vec::with_capacity(44 + pcm_data.len());

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt sub-chunk (16 bytes for PCM)
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav.extend_from_slice(&1u16.to_le_bytes());  // audio format (1 = PCM)
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data sub-chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    wav.extend_from_slice(pcm_data);

    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_is_valid() {
        let pcm = vec![0u8; 100]; // dummy PCM16 data
        let wav = wrap_pcm16_as_wav(&pcm, 24000, 1);

        // RIFF header
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");

        // fmt chunk
        assert_eq!(&wav[12..16], b"fmt ");
        let fmt_size = u32::from_le_bytes([wav[16], wav[17], wav[18], wav[19]]);
        assert_eq!(fmt_size, 16);
        let audio_fmt = u16::from_le_bytes([wav[20], wav[21]]);
        assert_eq!(audio_fmt, 1); // PCM

        // data chunk
        assert_eq!(&wav[36..40], b"data");
        let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
        assert_eq!(data_size, 100);

        // total size: 44 byte header + 100 byte data
        assert_eq!(wav.len(), 144);
    }

    #[test]
    fn wav_header_stereo_44100() {
        let pcm = vec![0u8; 200];
        let wav = wrap_pcm16_as_wav(&pcm, 44100, 2);

        // Channels
        let channels = u16::from_le_bytes([wav[22], wav[23]]);
        assert_eq!(channels, 2);

        // Sample rate
        let rate = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
        assert_eq!(rate, 44100);

        // Block align: 2ch * 2 bytes = 4
        let block_align = u16::from_le_bytes([wav[32], wav[33]]);
        assert_eq!(block_align, 4);

        // Byte rate
        let byte_rate = u32::from_le_bytes([wav[28], wav[29], wav[30], wav[31]]);
        assert_eq!(byte_rate, 44100 * 4);
    }
}
