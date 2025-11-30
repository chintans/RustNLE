use rangemap::RangeMap;
use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use uuid::Uuid;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Archive,
    Deserialize,
    Serialize,
    SerdeSerialize,
    SerdeDeserialize,
)]
#[archive(check_bytes)]
pub struct TimeRange {
    pub start: u64, // Microseconds
    pub duration: u64,
}

impl TimeRange {
    pub fn new(start: u64, duration: u64) -> Self {
        Self { start, duration }
    }

    pub fn end(&self) -> u64 {
        self.start + self.duration
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Archive, Deserialize, Serialize, SerdeSerialize, SerdeDeserialize,
)]
#[archive(check_bytes)]
pub struct Clip {
    pub asset_id: [u8; 16],
    pub source_range: TimeRange,   // In-point/Out-point in source file
    pub timeline_range: TimeRange, // Position in timeline
    pub track_index: u32,
    pub name: String,
}

impl Clip {
    pub fn new(
        name: String,
        asset_id: Uuid,
        source_range: TimeRange,
        timeline_range: TimeRange,
        track_index: u32,
    ) -> Self {
        Self {
            asset_id: *asset_id.as_bytes(),
            source_range,
            timeline_range,
            track_index,
            name,
        }
    }

    pub fn asset_uuid(&self) -> Uuid {
        Uuid::from_bytes(self.asset_id)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Track {
    // RangeMap maps a range (start..end) to a value (Clip).
    // The key is u64 (time in microseconds).
    pub clips: RangeMap<u64, Clip>,
}

impl Track {
    pub fn new() -> Self {
        Self {
            clips: RangeMap::new(),
        }
    }

    pub fn add(&mut self, clip: Clip) {
        // RangeMap handles overlaps by overwriting the existing range.
        // However, we need to ensure the key range matches the clip's timeline range.
        let start = clip.timeline_range.start;
        let end = clip.timeline_range.end();
        self.clips.insert(start..end, clip);
    }

    pub fn query(&self, time: u64) -> Option<&Clip> {
        self.clips.get(&time)
    }

    // Helper for testing
    pub fn get_clips(&self) -> &RangeMap<u64, Clip> {
        &self.clips
    }
}

#[derive(Debug, Clone, Default)]
pub struct Timeline {
    pub video_tracks: Vec<Track>,
    pub audio_tracks: Vec<Track>,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            video_tracks: Vec::new(),
            audio_tracks: Vec::new(),
        }
    }

    pub fn add_video_track(&mut self) -> usize {
        self.video_tracks.push(Track::new());
        self.video_tracks.len() - 1
    }

    pub fn add_audio_track(&mut self) -> usize {
        self.audio_tracks.push(Track::new());
        self.audio_tracks.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_overlap() {
        let mut track = Track::new();
        let asset_id = Uuid::new_v4();

        // Clip A: 0s to 10s
        let clip_a = Clip::new(
            "A".to_string(),
            asset_id,
            TimeRange::new(0, 10_000_000),
            TimeRange::new(0, 10_000_000),
            0,
        );
        track.add(clip_a);

        // Clip B: 4s to 6s (Overwriting middle of A)
        let clip_b = Clip::new(
            "B".to_string(),
            asset_id,
            TimeRange::new(0, 2_000_000),
            TimeRange::new(4_000_000, 2_000_000),
            0,
        );
        track.add(clip_b);

        // Verify
        assert_eq!(track.query(2_000_000).unwrap().name, "A");
        assert_eq!(track.query(5_000_000).unwrap().name, "B");
        assert_eq!(track.query(8_000_000).unwrap().name, "A");
    }

    #[test]
    fn test_ripple_delete_placeholder() {
        assert!(true);
    }
}
