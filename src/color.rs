use std::collections::HashMap;
use std::fmt::Write;
use std::hash::{Hash, Hasher};

use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use kmeans::{KMeans, KMeansConfig};
use opencv::prelude::*;
use opencv::{core, imgproc};
use palette::{Hsv, IntoColor, Srgb};

use crate::video::VideoFrameIterator;

#[derive(Clone, Copy, Debug)]
pub struct Color(pub Srgb<u8>);

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.red.hash(state);
        self.0.green.hash(state);
        self.0.blue.hash(state);
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.0.red == other.0.red && self.0.green == other.0.green && self.0.blue == other.0.blue
    }
}

impl Eq for Color {}

#[derive(Debug)]
pub struct ColorCluster {
    pub centroid: Color,
    pub assignments: Vec<(Color, usize)>,
}

pub fn extract_color_ranking_from_video(
    frame_iterator: VideoFrameIterator,
    resize_height: i32,
    saturation_threshold: f32,
    luminance_threshold: f32,
) -> Result<Vec<(Color, usize)>, Box<dyn std::error::Error>> {
    let mut color_counts: HashMap<Color, usize> = HashMap::new();

    let pb_length = frame_iterator.end_frame - frame_iterator.frame_number;
    let pb = ProgressBar::new(pb_length);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );

    for (frame_number, frame) in frame_iterator.enumerate() {
        let colors = extract_colors_from_frame(
            &frame,
            resize_height,
            saturation_threshold,
            luminance_threshold,
        );
        for color in colors {
            *color_counts.entry(color).or_insert(0) += 1;
        }

        pb.set_position(frame_number as u64);
    }

    pb.finish_with_message("Color extraction complete!");

    let mut color_ranking: Vec<(Color, usize)> = color_counts.into_iter().collect();
    color_ranking.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(color_ranking)
}

pub fn extract_colors_from_frame(
    frame: &Mat,
    resize_height: i32,
    saturation_threshold: f32,
    luminance_threshold: f32,
) -> Vec<Color> {
    let aspect_ratio = frame.cols() as f64 / frame.rows() as f64;
    let new_width = (resize_height as f64 * aspect_ratio).round() as i32;

    let mut resized_frame = Mat::default();
    imgproc::resize(
        &frame,
        &mut resized_frame,
        core::Size::new(new_width, resize_height),
        0.,
        0.,
        imgproc::INTER_AREA,
    )
    .unwrap();
    let mut rgb_frame = Mat::default();
    imgproc::cvt_color(&resized_frame, &mut rgb_frame, imgproc::COLOR_BGR2RGB, 3).unwrap();

    let data_start = rgb_frame.datastart();
    let data_end = rgb_frame.dataend();
    let data_slice =
        unsafe { std::slice::from_raw_parts(data_start, data_end as usize - data_start as usize) };

    let mut colors = Vec::new();
    for chunk in data_slice.chunks(3) {
        let color = Srgb::new(chunk[0], chunk[1], chunk[2]);

        let white_threshold = 250;
        let black_threshold = 5;

        if (color.red >= white_threshold
            && color.green >= white_threshold
            && color.blue >= white_threshold)
            || (color.red <= black_threshold
                && color.green <= black_threshold
                && color.blue <= black_threshold)
        {
            continue;
        }

        // Check if the color's saturation is above the threshold
        let hsv_color: Hsv = color.into_format::<f32>().into_color();
        if hsv_color.saturation >= saturation_threshold && hsv_color.value >= luminance_threshold {
            colors.push(Color(color));
        }
    }
    colors
}

pub fn create_color_clusters(
    color_ranking: &[(Color, usize)],
    num_clusters: usize,
) -> Vec<ColorCluster> {
    // Convert colors to f64 and create a Vec of tuples containing the color data and count
    let data_and_counts: Vec<(Vec<f64>, usize)> = color_ranking
        .iter()
        .map(|(color, count)| {
            let color_f = color.0.into_format::<f64>();
            (vec![color_f.red, color_f.green, color_f.blue], *count)
        })
        .collect();

    // Calculate the total number of samples and the sample dimensions
    let total_samples: usize = data_and_counts.iter().map(|(_, count)| count).sum();
    let sample_dims = 3;

    // Create a flat Vec of f64 data for KMeans clustering
    let mut data: Vec<f64> = Vec::with_capacity(total_samples * sample_dims);
    for (color_data, count) in data_and_counts {
        for _ in 0..count {
            // repeating this for each count gives more weight to colors with higher counts
            data.extend(color_data.clone());
        }
    }

    // Perform KMeans clustering
    let kmeans = KMeans::new(data, total_samples, sample_dims);
    let result = kmeans.kmeans_lloyd(
        num_clusters,
        100,
        KMeans::init_kmeanplusplus,
        &KMeansConfig::default(),
    );

    // Assign colors to their respective clusters based on the clustering result
    let mut assignments: Vec<Vec<(Color, usize)>> = vec![vec![]; num_clusters];
    let mut assignment_iter = result.assignments.into_iter();
    for (color, count) in color_ranking {
        if let Some(cluster_idx) = assignment_iter.next() {
            assignments[cluster_idx].push((*color, *count));
        }
    }

    // Convert centroids back to Color and create ColorCluster structs
    let centroids = result.centroids.chunks(sample_dims).map(|chunk| {
        let r = (chunk[0] * 255.0).clamp(0.0, 255.0) as u8;
        let g = (chunk[1] * 255.0).clamp(0.0, 255.0) as u8;
        let b = (chunk[2] * 255.0).clamp(0.0, 255.0) as u8;
        Color(Srgb::new(r, g, b))
    });

    centroids
        .into_iter()
        .zip(assignments.into_iter())
        .map(|(centroid, assignments)| ColorCluster {
            centroid,
            assignments,
        })
        .collect()
}
