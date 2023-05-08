use std::path::Path;
use std::time::Duration;

use opencv::prelude::*;
use opencv::videoio;

/// A struct representing an iterator over video frames.
pub struct VideoFrameIterator {
    pub capture: videoio::VideoCapture,
    pub end_frame: u64,
    pub frame_number: u64,
}

impl VideoFrameIterator {
    /// Create a new `VideoFrameIterator` for the given video file.
    ///
    /// # Arguments
    ///
    /// * `video_file` - A path to the video file.
    /// * `start_timestamp` - An optional duration representing the starting timestamp of the video.
    /// * `end_timestamp` - An optional duration representing the ending timestamp of the video.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `VideoFrameIterator` or an error if the video file cannot be opened.
    pub fn new(
        video_file: &Path,
        start_timestamp: Option<Duration>,
        end_timestamp: Option<Duration>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut capture =
            videoio::VideoCapture::from_file(video_file.to_str().unwrap(), videoio::CAP_ANY)?;
        let fps = capture.get(videoio::CAP_PROP_FPS)?;
        let total_frames = capture.get(videoio::CAP_PROP_FRAME_COUNT)? as u64;

        let start_frame = start_timestamp.map_or(0, |ts| (ts.as_secs_f64() * fps).round() as u64);
        let end_frame =
            end_timestamp.map_or(total_frames, |ts| (ts.as_secs_f64() * fps).round() as u64);

        if start_frame > 0 {
            capture.set(videoio::CAP_PROP_POS_FRAMES, start_frame as f64)?;
        }

        Ok(Self {
            capture,
            end_frame,
            frame_number: start_frame,
        })
    }
}

impl Iterator for VideoFrameIterator {
    type Item = Mat;

    /// Return the next frame in the video.
    ///
    /// # Returns
    ///
    /// An `Option` containing the next frame as a `Mat` object, or `None` if there are no more frames.
    fn next(&mut self) -> Option<Self::Item> {
        if self.frame_number >= self.end_frame {
            return None;
        }

        let mut frame = Mat::default();
        if !self.capture.read(&mut frame).unwrap() {
            return None;
        }

        self.frame_number += 1;
        Some(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencv::core::Size;
    use std::path::PathBuf;

    fn get_video_path() -> PathBuf {
        PathBuf::from("video.mp4")
    }

    #[test]
    fn test_video_frame_iterator_creation() {
        let video_path = get_video_path();
        let iterator = VideoFrameIterator::new(&video_path, None, None);

        assert!(iterator.is_ok());
    }

    #[test]
    fn test_video_frame_iterator_iteration() {
        let video_path = get_video_path();
        let iterator = VideoFrameIterator::new(&video_path, None, None).unwrap();
        let mut frame_count = 0;

        for _frame in iterator {
            frame_count += 1;
        }

        assert!(frame_count > 0);
    }

    #[test]
    fn test_video_frame_iterator_start_and_end_timestamps() {
        let video_path = get_video_path();
        let start_timestamp = Duration::from_secs(2);
        let end_timestamp = Duration::from_secs(4);
        let iterator = VideoFrameIterator::new(&video_path, Some(start_timestamp), Some(end_timestamp)).unwrap();

        let mut frame_count = 0;
        let mut last_frame = None;
        for frame in iterator {
            frame_count += 1;
            last_frame = Some(frame);
        }

        assert!(frame_count > 0);

        if let Some(frame) = last_frame {
            let size = frame.size().unwrap();
            assert_eq!(size, Size::new(1280, 720));
        } else {
            panic!("No frames found");
        }
    }
}
