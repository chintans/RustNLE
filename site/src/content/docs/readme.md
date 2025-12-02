# Rust-NLE

**A High-Performance, Professional Non-Linear Video Editor in Rust**

Rust-NLE is an ambitious project to build a modern, stable, and high-performance video editing application designed to provide professional-grade functionality, with the safety and concurrency benefits of the Rust programming language.

## 🎯 Objective

The primary goal of Rust-NLE is to address the common pain points of legacy NLEs:
*   **Stability**: Eliminate crashes caused by memory mismanagement (segmentation faults) using Rust's memory safety guarantees.
*   **Performance**: Leverage modern hardware acceleration (GPU) and efficient multi-threading to prevent UI freezes and ensure smooth playback.
*   **Architecture**: Decouple the UI from the processing engine to ensure a responsive user experience even under heavy load.

## 🏗️ Architecture

The project follows a **Split-State Architecture**, separating the User Interface from the heavy lifting of audio/video processing.

### Workspace Structure

*   **`nle_app`**: The application entry point and CLI/Debug interface.
*   **`nle_core`**: Central state management and orchestration.
*   **`nle_data`**: The "Truth" of the project. Defines data structures like the Timeline and Clips, utilizing **Interval Trees** for efficient queries and **rkyv** for zero-copy serialization.
*   **`nle_media`**: Asynchronous video decoding engine using **ffmpeg-next**. Implements an Actor model for fetching frames.
*   **`nle_render`**: The compositing engine built on **wgpu**. Handles the render graph, shaders, and final image output.
*   **`nle_audio`**: The audio mixing subsystem using **cpal**. Acts as the master clock for synchronization.
*   **`nle_utils`**: Shared utilities for logging, timecode handling, etc.

## 🚀 Current Status

**Phase 1: The Core Engine**
We are currently in the initial phase of development, focusing on building a headless renderer that can:
*   Decode video asynchronously.
*   Mix audio streams.
*   Composite frames using GPU compute shaders.
*   Verify functionality without a GUI.

## 🗺️ Roadmap & Comparison

### Why Rust-NLE?

| Feature | Legacy NLEs | Rust-NLE |
| :--- | :---: | :---: |
| **Memory Safety** | ❌ (Manual Management) | ✅ (Rust Ownership) |
| **Crash Resilience** | ❌ (Frequent Crashes) | ✅ (Safe Concurrency) |
| **Zero-Copy Pipeline** | ❌ (Legacy Bottlenecks) | ✅ (Modern WGPU) |
| **Startup Time** | ❌ (Slow) | ✅ (Instant) |
| **Modern UI** | ❌ (Legacy Frameworks) | ✅ (GPU-Accelerated) |

### Phased Development

#### ✅ Phase 1: The Core Engine (Current)
- [x] Asynchronous Video Decoding (FFmpeg)
- [x] Audio Mixing Engine (CPAL)
- [x] GPU Compositing (WGPU)
- [x] Headless Rendering Test

#### 🚧 Phase 2: The UI Skeleton (Next)
- [ ] Makepad UI Integration
- [ ] Source & Program Monitors
- [ ] Basic Timeline Visualization
- [ ] File Import Workflow

#### 📅 Phase 3: The Editor
- [ ] Timeline Editing Tools (Ripple, Roll, Slip, Slide)
- [ ] Undo/Redo System (rkyv snapshots)
- [ ] Project Persistence (Save/Load)

#### 📅 Phase 4: Professional Features
- [ ] VST3 Plugin Hosting
- [ ] Color Grading (OCIO)
- [ ] Export/Render Queue
- [ ] WASM Plugin System

## 🛠️ Prerequisites

To build and run Rust-NLE, you need:

1.  **Rust Toolchain**: Install the latest stable version from [rustup.rs](https://rustup.rs/).
2.  **FFmpeg Development Libraries**:
    *   **Windows**: You may need to set `FFMPEG_DIR` environment variable pointing to your FFmpeg installation (with include/lib folders).
    *   **Linux**: Install `ffmpeg-dev` or equivalent (e.g., `libavcodec-dev`, `libavformat-dev`, `libavutil-dev`, `libswscale-dev`, `libavfilter-dev`, `libavdevice-dev`).
    *   **macOS**: `brew install ffmpeg`

## 💻 Setup & Usage

1.  **Clone the Repository**:
    ```bash
    git clone https://github.com/chintans/RustNLE.git
    cd RustNLE
    ```

2.  **Build the Project**:
    ```bash
    cargo build
    ```

3.  **Run the Headless Test**:
    Currently, the main entry point is a headless test that initializes the engine and performs a render test.
    ```bash
    cargo run --bin nle_headless_test
    ```

## 🤝 How to Contribute

We welcome contributions! Here's how you can help:

1.  **Fork the Repository**: Create your own fork on GitHub.
2.  **Create a Branch**: `git checkout -b feature/my-new-feature`
3.  **Make Changes**: Write clean, idiomatic Rust code.
4.  **Format & Lint**: Ensure your code follows the project's style.
    ```bash
    cargo fmt
    cargo clippy
    ```
5.  **Run Tests**: Verify that your changes don't break existing functionality.
    ```bash
    cargo test
    ```
6.  **Submit a Pull Request**: Describe your changes and the problem they solve.

## 📄 License

[License Information Here - e.g., MIT/Apache 2.0]
