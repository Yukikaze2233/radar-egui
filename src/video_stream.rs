#[cfg(feature = "video")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "video")]
use gst::prelude::*;
#[cfg(feature = "video")]
use gst_app::AppSink;

#[cfg(feature = "video")]
pub struct VideoFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[cfg(feature = "video")]
pub struct VideoStream {
    pipeline: Option<gst::Pipeline>,
    frame: Arc<Mutex<Option<VideoFrame>>>,
}

#[cfg(feature = "video")]
impl VideoStream {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            frame: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&mut self, sdp_path: &str) -> bool {
        gst::init().ok();

        let pipeline_str = format!(
            "filesrc location={} ! sdpdemux ! rtph264depay ! avdec_h264 ! videoconvert ! video/x-raw,format=BGRA ! appsink name=sink emit-signals=true",
            sdp_path
        );

        let element = match gst::parse::launch(&pipeline_str) {
            Ok(e) => e,
            Err(e) => {
                log::error!("Failed to create pipeline: {}", e);
                return false;
            }
        };

        let pipeline = element.dynamic_cast::<gst::Pipeline>().unwrap();

        // Monitor bus for errors
        let bus = pipeline.bus().unwrap();
        let _bus_watch = bus
            .add_watch(move |_, msg| {
                use gst::MessageView;
                match msg.view() {
                    MessageView::Error(err) => {
                        log::error!(
                            "GStreamer error from {:?}: {} ({})",
                            err.src().map(|s| s.path_string()),
                            err.error(),
                            err.debug().unwrap_or_default()
                        );
                    }
                    MessageView::Warning(warn) => {
                        log::warn!(
                            "GStreamer warning from {:?}: {} ({})",
                            warn.src().map(|s| s.path_string()),
                            warn.error(),
                            warn.debug().unwrap_or_default()
                        );
                    }
                    MessageView::Eos(_) => {
                        log::info!("GStreamer EOS");
                    }
                    MessageView::StateChanged(state) => {
                        log::debug!(
                            "GStreamer state: {:?} -> {:?}",
                            state.old(),
                            state.current()
                        );
                    }
                    _ => {}
                }
                gst::glib::ControlFlow::Continue
            })
            .ok();

        let appsink = pipeline.by_name("sink").unwrap().downcast::<AppSink>().unwrap();

        let frame = self.frame.clone();
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                    let caps = sample.caps().ok_or(gst::FlowError::Error)?;
                    let info = gst_video::VideoInfo::from_caps(caps).map_err(|_| gst::FlowError::Error)?;

                    let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                    let data = map.to_vec();

                    if let Ok(mut f) = frame.lock() {
                        *f = Some(VideoFrame {
                            data,
                            width: info.width(),
                            height: info.height(),
                        });
                    }

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        if pipeline.set_state(gst::State::Playing).is_err() {
            log::error!("Failed to set pipeline to Playing");
            return false;
        }

        self.pipeline = Some(pipeline);
        log::info!("Video stream started from {}", sdp_path);
        true
    }

    pub fn get_frame(&self) -> Option<VideoFrame> {
        self.frame.lock().ok()?.take()
    }
}

#[cfg(feature = "video")]
impl Drop for VideoStream {
    fn drop(&mut self) {
        if let Some(pipeline) = &self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
    }
}

#[cfg(not(feature = "video"))]
pub struct VideoStream;

#[cfg(not(feature = "video"))]
impl VideoStream {
    pub fn new() -> Self { Self }
    pub fn start(&mut self, _sdp_path: &str) -> bool {
        log::warn!("Video feature not enabled");
        false
    }
    pub fn get_frame(&self) -> Option<()> { None }
}
