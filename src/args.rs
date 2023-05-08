use clap::Parser;
use std::path::PathBuf;

/// Extracts the color palette from a video
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The input video to parse for colors
    #[arg(value_name = "FILE")]
    pub video: PathBuf,

    /// Saturation threshold for colors
    #[arg(short, long, default_value = "0.0")]
    pub saturation: f32,

    /// Luminance threshold for colors
    #[arg(short, long, default_value = "0.0")]
    pub luminance: f32,

    /// Resize the video to this height (maintains aspect ratio)
    #[arg(short, long, default_value = "12")]
    pub resize_height: i32,

    /// Number of color clusters to create
    #[arg(short, long, default_value = "5")]
    pub color_clusters: usize,

    /// Start time of the video to extract colors (format: HH:MM:SS)
    #[arg(long, value_parser = parse_duration)]
    pub start: Option<std::time::Duration>,

    /// End time of the video to extract colors (format: HH:MM:SS)
    #[arg(long, value_parser = parse_duration)]
    pub end: Option<std::time::Duration>,
}

fn parse_duration(arg: &str) -> Result<std::time::Duration, std::num::ParseIntError> {
    let mut parts = arg.split(':').map(|s| s.parse::<u64>());

    let hours = parts.next().unwrap_or(Ok(0))?;
    let minutes = parts.next().unwrap_or(Ok(0))?;
    let seconds = parts.next().unwrap_or(Ok(0))?;

    Ok(std::time::Duration::from_secs(
        hours * 3600 + minutes * 60 + seconds,
    ))
}
