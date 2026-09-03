// Copyright 2024 the Kompari Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::{Path, PathBuf};

use image::RgbImage;

use retgui::drivers::headless::HeadlessApp;
use retgui::elements::Window;

pub fn screenshot_rgb(test: &mut HeadlessApp, window: &Window) -> RgbImage {
    let screenshot = test.screenshot(window);
    let image = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        screenshot.width as u32,
        screenshot.height as u32,
        screenshot.pixels,
    )
    .expect("renderer returned an invalid screenshot buffer");
    image::DynamicImage::ImageRgba8(image).to_rgb8()
}

/// Directory where current tests creates images
pub fn current_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("retgui")
        .join("tests")
        .join("current")
}

/// Directory with blessed snapshots
pub fn snapshot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("retgui")
        .join("tests")
        .join("snapshots")
}

pub fn is_generate_all_mode() -> bool {
    std::env::var("retgui_TEST")
        .map(|x| x.eq_ignore_ascii_case("generate-all"))
        .unwrap_or(false)
}

/// Check an image against snapshot
pub fn check_snapshot(image: RgbImage, image_name: &str) {
    let snapshot_dir = snapshot_dir();
    println!("Snapshots DIR: {}", snapshot_dir.to_str().unwrap());
    let snapshot = image::ImageReader::open(snapshot_dir.join(image_name))
        .map_err(|e| e.to_string())
        .and_then(|x| x.decode().map_err(|e| e.to_string()))
        .map(|x| x.to_rgb8());
    if let Ok(snapshot) = snapshot {
        if snapshot != image {
            image.save(current_dir().join(image_name)).unwrap();
            panic!("Snapshot is different; run 'cargo xtask report' for report")
        }
    } else {
        println!("writing test to {}", current_dir().join(image_name).display());
        image.save(current_dir().join(image_name)).unwrap();
        snapshot.unwrap();
    }
    if is_generate_all_mode() {
        image.save(current_dir().join(image_name)).unwrap();
    }
}
