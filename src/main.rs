mod args;
mod color;
mod video;

use std::path::Path;

use clap::Parser;
use colored::*;
use palette::Srgb;

use args::Args;
use color::{create_color_clusters, extract_color_ranking_from_video, Color, ColorCluster};
use video::VideoFrameIterator;

fn main() {
    let args = Args::parse();

    let video_file = Path::new(&args.video);
    if !video_file.is_file() {
        eprintln!("Video path is not a file");
        std::process::exit(1);
    }

    let video_frame_iter = match VideoFrameIterator::new(video_file, args.start, args.end) {
        Ok(iter) => iter,
        Err(e) => {
            eprintln!("Error opening video: {}", e);
            std::process::exit(1);
        }
    };

    let color_ranking = match extract_color_ranking_from_video(
        video_frame_iter,
        args.resize_height,
        args.saturation,
        args.luminance,
    ) {
        Ok(palette) => palette,
        Err(e) => {
            eprintln!("Error processing video: {}", e);
            std::process::exit(1);
        }
    };

    print_color_palette(color_ranking.as_slice());

    let color_clusters = create_color_clusters(color_ranking.as_slice(), args.color_clusters);

    print_color_clusters(color_clusters.as_slice())
}

fn print_color_palette(palette: &[(Color, usize)]) {
    let num_colors = 10;
    println!("Top {} colors in the video:", num_colors);

    for (idx, (color, count)) in palette.iter().take(num_colors).enumerate() {
        let Srgb {
            red, green, blue, ..
        } = color.0;
        let color_code = format!("{:02X}{:02X}{:02X}", red, green, blue);
        let color_block = "  ".on_truecolor(red, green, blue);
        println!(
            "{}. #{} {} (count: {})",
            idx + 1,
            color_code,
            color_block,
            count
        );
    }
}

fn print_color_clusters(clusters: &[ColorCluster]) {
    println!("Color clusters:");
    for (idx, cluster) in clusters.iter().enumerate() {
        let Srgb {
            red, green, blue, ..
        } = cluster.centroid.0;
        let color_code = format!("{:02X}{:02X}{:02X}", red, green, blue);
        let color_block = "  ".on_truecolor(red, green, blue);
        println!("{}. Centroid: #{} {}", idx + 1, color_code, color_block);
    }
}
