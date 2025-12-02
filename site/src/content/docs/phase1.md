This is a **Phase 1 Technical Specification** for "Rust-NLE," focusing exclusively on the Core Engine (The "headless" renderer and state manager). This document serves as the implementation guide for the first 3 months of development.

### **Phase 1: The Core Engine & Composition Pipeline**

**Objective:** Build a thread-safe, high-performance media engine capable of asynchronous video decoding, audio mixing, and GPU compositing.
**Deliverable:** A CLI-based executable (and library) that can load a project file, play video to a debug window, and export a rendered frame to disk.

-----

### **1. Modular Architecture (Cargo Workspace)**

To ensure compilation speed and separation of concerns, the project will use a Cargo Workspace with the following crate structure.

#### **Directory Structure**

rust-nle/
├── Cargo.toml                  \# Workspace definition
├── nle\_app/                    \# Entry point (CLI/Debug Window)
├── nle\_core/                   \# State management & Orchestration
├── nle\_data/                   \# Data structures (Timeline, IntervalTree)
├── nle\_media/                  \# FFmpeg actors & Hardware Decoding
├── nle\_render/                 \# WGPU pipeline & Shader Graph
├── nle\_audio/                  \# CPAL audio backend & DSP
└── nle\_utils/                  \# Shared utilities (Timecode, Logging)

#### **Crate Dependency Graph**

  * `nle_app` depends on `nle_core`
  * `nle_core` depends on `nle_data`, `nle_media`, `nle_render`, `nle_audio`
  * `nle_data` is a leaf node (minimal dependencies).

-----

### **2. Component Specifications**

#### **2.1 `nle_data`: The Timeline Model**

This crate defines the "truth" of the editing project. It must be serializable (via `rkyv`) and allow for $O(\log N)$ spatial queries.

  * **Key Data Structure:** `Track`
    Instead of a naive `Vec<Clip>`, we use a **Gap-Query Interval Tree**. This allows us to instantly query "What clip is at Timecode $T$?" without iterating through the whole list.

  * **Struct Definitions:**

    ```rust
    // nle_data/src/model.rs

    use rkyv::{Archive, Deserialize, Serialize};

    #
    pub struct TimeRange {
        pub start: u64, // Microseconds
        pub duration: u64,
    }

    #
    pub struct Clip {
        pub asset_id: uuid::Uuid,
        pub source_range: TimeRange, // In-point/Out-point in source file
        pub timeline_range: TimeRange, // Position in timeline
        pub track_index: u32,
    }

    // The Timeline holds a collection of Tracks, which hold IntervalTrees of Clips
    pub struct Timeline {
        // Using `rangemap` or a custom IntervalTree implementation
        pub video_tracks: Vec<rangemap::RangeMap<u64, Clip>>,
        pub audio_tracks: Vec<rangemap::RangeMap<u64, Clip>>,
    }
    ```

  * **Unit Tests (`nle_data`):**

      * `test_insert_overlap()`: Ensure inserting a clip on top of another correctly "cuts" the underlying clip (overwrite logic).
      * `test_ripple_delete()`: Verify that removing a range shifts subsequent items left.

#### **2.2 `nle_media`: The Async Decoder**

This crate manages `ffmpeg-next`. Since video decoding is blocking and CPU-intensive, it must run in a dedicated thread pool (Actor model).

  * **Actor Logic:**
    Each `Asset` has a `DecoderActor`. The Engine sends `Seek(Time)` or `NextFrame` messages. The Actor responds with `DecodedFrame(WgpuTexture)`.

  * **Zero-Copy Interface (Crucial for Performance):**
    We define a `VideoFrame` trait that abstracts the data source.

    ```rust
    // nle_media/src/frame.rs

    pub enum FrameData {
        Cpu(Vec<u8>),               // Fallback (Software Decode)
        DmaBuf(std::os::fd::RawFd), // Linux (VAAPI)
        Dx12Handle(isize),          // Windows (DirectX)
        MetalRef(*mut c_void),      // macOS (VideoToolbox)
    }

    pub struct VideoFrame {
        pub ptr: FrameData,
        pub timecode: u64,
        pub width: u32,
        pub height: u32,
    }
    ```

  * **Modularity Requirement:**
    This module must allow **Mocking**. In unit tests, we don't want to load real MP4 files.

    ```rust
    pub trait MediaSource: Send + Sync {
        fn get_frame_at(&mut self, time: u64) -> Result<VideoFrame>;
    }

    // Mock implementation for testing
    pub struct SolidColorSource { color: [u8; 4] }
    ```

#### **2.3 `nle_render`: The Compositor (WGPU)**

This crate builds the Render Graph. It takes a list of `VideoFrame`s and applies shaders.

  * **Pipeline Design:**

    1.  **Upload Stage:** Convert `FrameData` (from `nle_media`) into `wgpu::Texture`.
    2.  **Compute Stage:** Apply effects (scaling, color correction).
    3.  **Composite Stage:** Blend layers (Alpha Over) using a fragment shader.

  * **The Render Node Trait:**

    ```rust
    pub trait RenderNode {
        fn update(&mut self, params: &ParamBlock);
        fn encode(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView);
    }
    ```

  * **Headless Testing:**
    We will use `wgpu`'s **Texture-to-Buffer copy** capabilities to verify rendering without a window.

      * *Test:* Render a Red square over a Blue background.
      * *Assertion:* Read the output buffer. Pixel (10,10) must be Red (255, 0, 0).

#### **2.4 `nle_audio`: The Mixer**

This crate handles `cpal`. It is the **Master Clock** of the engine.

  * **Synchronization Strategy:**
    The audio callback fires roughly every 10ms (buffer size dependent).
    1.  `cpal` requests `N` samples.
    2.  `nle_audio` fetches samples from active clips in `nle_data`.
    3.  If a clip needs resampling (e.g., 44.1kHz source in 48kHz project), `rubato` is used.
    4.  The current "Audio Time" is atomically written to a shared `AtomicU64` that the Video Engine reads to know which frame to draw.

-----

### **3. Detailed Functionality & Data Flow**

#### **3.1 The "Play" Command Flow**

1.  **User Action:** User presses Spacebar in `nle_app`.
2.  **Core:** `Engine` state sets `playing = true`.
3.  **Audio Thread:**
      * Wakes up. Calculates required samples for the next buffer.
      * Updates `GlobalTime` variable.
4.  **Render Thread (60Hz Loop):**
      * Reads `GlobalTime`.
      * Queries `Timeline` for visible clips at `GlobalTime`.
      * Sends `GetFrame` requests to `DecoderActors`.
      * `DecoderActors` return frames (ideally from a pre-fetched queue).
      * `nle_render` composites frames.
      * Draws to Screen.

#### **3.2 Unit Test Plan (Phase 1)**

The quality of the engine relies on testing *without* the GUI.

**Test Suite 1: Timeline Logic (`nle_data`)**

```rust
#[test]
fn test_overwrite_edit() {
    let mut track = Track::new();
    // Clip A: 0s to 10s
    track.add(Clip::new("A", 0..10)); 
    // Clip B: 4s to 6s (Overwriting middle of A)
    track.add(Clip::new("B", 4..6));
    
    assert_eq!(track.query(2).id, "A");
    assert_eq!(track.query(5).id, "B");
    assert_eq!(track.query(8).id, "A"); // Should have split Clip A into two
}
```

**Test Suite 2: The Mock Decoder (`nle_media`)**

```rust
#[test]
fn test_mock_decoder_accuracy() {
    let mut source = MockSource::new().with_pattern(TestPattern::ColorBars);
    let frame = source.get_frame_at(1000).unwrap();
    // Verify we got a valid frame struct back
    assert_eq!(frame.width, 1920);
    assert_eq!(frame.height, 1080);
}
```

**Test Suite 3: Audio Mixing (`nle_audio`)**

```rust
#[test]
fn test_stereo_summing() {
    let signal_a = vec![0.5, 0.5]; // Left, Right
    let signal_b = vec![0.2, 0.2];
    let mixed = mix_signals(&[&signal_a, &signal_b]);
    
    assert_eq!(mixed, 0.7); // 0.5 + 0.2
}
```

-----

### **4. Rust Crate Selection (Phase 1)**

| Dependency | Version | Purpose |
| :--- | :--- | :--- |
| `wgpu` | `0.19` | Graphics API (WebGPU implementation). |
| `ffmpeg-next` | `6.0` | Safe bindings for FFmpeg. |
| `cpal` | `0.15` | Low-level Audio I/O. |
| `rkyv` | `0.7` | Zero-copy serialization for project files. |
| `tokio` | `1.0` | Async runtime for Decoder Actors. |
| `crossbeam-channel`| `0.5` | High-performance messaging between Audio/Video threads. |
| `rangemap` | `1.4` | Data structure for track intervals. |
| `bytemuck` | `1.14` | Casting raw bytes for GPU buffers. |

### **5. Definition of Done (Phase 1)**

Phase 1 is complete when `cargo run --bin nle_headless_test` executes successfully.
The test binary must:

1.  Initialize the engine.
2.  Programmatically create a timeline with 2 video tracks and 1 audio track.
3.  Load a "test\_pattern.mp4" (generated or included).
4.  Render 100 frames to a directory `./output/`.
5.  Exit with code 0.
6.  `cargo test` passes all unit tests for Data, Media, and Audio crates.