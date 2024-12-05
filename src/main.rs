/*
 * rust-wall - A Rust-based Bing wallpaper downloader
 * Copyright (C) 2024 Steven Showers
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct BingResponse {
    images: Vec<BingImage>,
}

#[derive(Debug, Deserialize)]
struct BingImage {
    url: String,
    startdate: String,
    copyright: String,
}

fn get_wallpaper_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("Failed to get HOME environment variable")?;
    let wall_dir = Path::new(&home).join("Pictures").join("wallpapers");
    Ok(wall_dir)
}

async fn download_image(client: &reqwest::Client, url: &str, filepath: &Path) -> Result<()> {
    if filepath.exists() {
        println!("Skipping: {} (already exists)", filepath.display());
        return Ok(());
    }

    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    fs::write(filepath, bytes)?;
    println!("Downloaded: {}", filepath.display());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::new();
    let wall_dir = get_wallpaper_dir()?;

    // Create wallpapers directory if it doesn't exist
    if !wall_dir.exists() {
        fs::create_dir_all(&wall_dir).context("Failed to create wallpapers directory")?;
    }

    // Bing API endpoint for the last 8 images
    let url = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=8&mkt=en-US";
    
    let response = client
        .get(url)
        .send()
        .await?
        .json::<BingResponse>()
        .await
        .context("Failed to fetch Bing wallpaper metadata")?;

    let total_images = response.images.len();
    let mut downloaded = 0;
    let mut skipped = 0;
    
    for image in response.images {
        let image_url = format!("https://www.bing.com{}", image.url);
        let filename = format!(
            "bing_{}_{}.jpg",
            image.startdate,
            image.copyright
                .replace("/", "_")
                .replace("\\", "_")
                .replace(" ", "_")
        );
        let filepath = wall_dir.join(filename);
        
        if filepath.exists() {
            skipped += 1;
        } else {
            downloaded += 1;
        }
        
        download_image(&client, &image_url, &filepath).await?;
    }

    println!("\nSummary:");
    println!("- Location: {}", wall_dir.display());
    println!("- Total images processed: {}", total_images);
    println!("- New downloads: {}", downloaded);
    println!("- Skipped (already exist): {}", skipped);
    Ok(())
}
