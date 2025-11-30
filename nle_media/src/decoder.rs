use crate::frame::{MediaSource, MockSource, VideoFrame};
use anyhow::Result;
use tokio::sync::mpsc;

pub enum DecoderMessage {
    GetFrame {
        time: u64,
        response: tokio::sync::oneshot::Sender<Result<VideoFrame>>,
    },
}

pub struct DecoderActor {
    source: Box<dyn MediaSource>,
    receiver: mpsc::Receiver<DecoderMessage>,
}

impl DecoderActor {
    pub fn new(source: Box<dyn MediaSource>, receiver: mpsc::Receiver<DecoderMessage>) -> Self {
        Self { source, receiver }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                DecoderMessage::GetFrame { time, response } => {
                    let frame = self.source.get_frame_at(time);
                    let _ = response.send(frame);
                }
            }
        }
    }
}

// Helper to spawn a decoder
pub fn spawn_decoder(width: u32, height: u32) -> mpsc::Sender<DecoderMessage> {
    let (tx, rx) = mpsc::channel(32);
    let source = Box::new(MockSource::new(width, height));
    let actor = DecoderActor::new(source, rx);

    tokio::spawn(async move {
        actor.run().await;
    });

    tx
}
