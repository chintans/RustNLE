use anyhow::Result;
use std::ffi::c_void;

#[derive(Debug)]
pub enum FrameData {
    Cpu(Vec<u8>),               // Fallback (Software Decode)
    DmaBuf(std::os::raw::c_int), // Linux (VAAPI) - using c_int for RawFd
    Dx12Handle(isize),          // Windows (DirectX)
    MetalRef(*mut c_void),      // macOS (VideoToolbox)
}

// Implement Send/Sync for FrameData manually if needed, but Vec<u8> and primitives are Send/Sync.
// Pointers are not Send/Sync by default.
unsafe impl Send for FrameData {}
unsafe impl Sync for FrameData {}

#[derive(Debug)]
pub struct VideoFrame {
    pub ptr: FrameData,
    pub timecode: u64,
    pub width: u32,
    pub height: u32,
}

pub trait MediaSource: Send + Sync {
    fn get_frame_at(&mut self, time: u64) -> Result<VideoFrame>;
}

// Mock implementation for testing
pub struct MockSource {
    pub width: u32,
    pub height: u32,
}

impl MockSource {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl MediaSource for MockSource {
    fn get_frame_at(&mut self, time: u64) -> Result<VideoFrame> {
        // Return a dummy CPU frame
        let buffer_size = (self.width * self.height * 4) as usize;
        let data = vec![0u8; buffer_size];
        
        Ok(VideoFrame {
            ptr: FrameData::Cpu(data),
            timecode: time,
            width: self.width,
            height: self.height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_decoder_accuracy() {
        let mut source = MockSource::new(1920, 1080);
        let frame = source.get_frame_at(1000).unwrap();
        
        assert_eq!(frame.width, 1920);
        assert_eq!(frame.height, 1080);
        assert_eq!(frame.timecode, 1000);
        
        if let FrameData::Cpu(data) = frame.ptr {
            assert_eq!(data.len(), 1920 * 1080 * 4);
        } else {
            panic!("Expected CPU frame from MockSource");
        }
    }
}
