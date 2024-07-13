use aws_config::Region;
use aws_sdk_s3::{config::SharedCredentialsProvider, Client};
use aws_util::StaticCredentials;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use image::{io::Reader as ImageReader, GenericImage};
use libheif_rs::{
    ColorSpace, CompressionFormat, DecodingOptions, EncodingOptions, HeifContext, LibHeif,
};
use serde::Deserialize;
use std::{io::Cursor, time::Instant};
use utils::{compress, generate_white_image, get_random_image, resize, to_raw};

mod aws_util;
mod utils;

#[derive(Deserialize)]
struct QueryParams {
    secret: Option<String>,
}

#[axum::debug_handler]
async fn handler(
    Query(query): Query<QueryParams>,
    State(config): State<Config>,
) -> Result<Vec<u8>, StatusCode> {
    // check fot matching secrets
    if query.secret != config.secret {
        return Err(StatusCode::FORBIDDEN);
    }
    let mut now = Instant::now();
    let start = Instant::now();
    let (key, bytes) = get_random_image(&config).await;
    let parts: Vec<&str> = key.split('/').collect();
    let filename = parts.last().unwrap().to_ascii_lowercase();
    println!("downloaded: {:?}", now.elapsed());
    now = Instant::now();
    // debug
    if std::env::var("DEBUG").is_ok() {
        let mut f = std::fs::File::create(&filename).unwrap();
        _ = std::io::Write::write(&mut f, &bytes);
        _ = std::io::Write::flush(&mut f);
    }
    let parts: Vec<&str> = filename.split('.').collect();
    let suffix = parts.last().unwrap();

    //detect heic
    let img = if *suffix == "heic" {
        let lib_heif = LibHeif::new();
        let ctx = HeifContext::read_from_bytes(&bytes).unwrap();
        let handle = ctx.primary_image_handle().unwrap();
        println!("heic: {} {}", handle.width(), handle.height());
        let mut opts = DecodingOptions::new().unwrap();
        opts.set_convert_hdr_to_8bit(true);
        opts.set_ignore_transformations(true);
        let image = lib_heif
            .decode(&handle, ColorSpace::Undefined, Some(opts))
            .unwrap();
        let mut encoder = lib_heif
            .encoder_for_format(CompressionFormat::Jpeg)
            .unwrap();
        let encoding_options: EncodingOptions = Default::default();
        let mut context = HeifContext::new().unwrap();
        context
            .encode_image(&image, &mut encoder, Some(encoding_options))
            .unwrap();
        let jpeg_bytes = context.write_to_bytes().unwrap();
        let cursor = Cursor::new(jpeg_bytes);
        ImageReader::with_format(cursor, image::ImageFormat::Jpeg)
            .decode()
            .unwrap()
    } else {
        // any other format
        let cursor = Cursor::new(bytes);
        match *suffix {
            "jpeg" | "jpg" => ImageReader::with_format(cursor, image::ImageFormat::Jpeg)
                .decode()
                .unwrap(),
            "png" => ImageReader::with_format(cursor, image::ImageFormat::Png)
                .decode()
                .unwrap(),
            _ => ImageReader::with_format(cursor, image::ImageFormat::Jpeg)
                .decode()
                .unwrap(),
        }
    };
    println!("image read: {:?}", now.elapsed());
    now = Instant::now();
    // transform image
    let img = resize(img);
    let img = img.grayscale();
    let img = img.into_luma8();
    let img_w = img.width();
    let img_h = img.height();
    println!("image greyed: {:?}", now.elapsed());
    now = Instant::now();
    // apply white background
    let mut background = generate_white_image();
    let offset_x = if img_w == 1600 { 0 } else { (1600 - img_w) / 2 };
    let offset_y = if img_h == 1200 { 0 } else { (1200 - img_h) / 2 };
    background.copy_from(&img, offset_x, offset_y).unwrap();
    let bytes = background.to_vec();
    // convert
    let x = to_raw(bytes);
    let z = compress(x);
    println!("image compressed: {:?}", now.elapsed());
    println!("response latency: {:?}", start.elapsed());
    Ok(z)
}

#[derive(Clone)]
pub struct Config {
    pub bucket_name: String,
    pub prefix: Option<String>,
    pub client: Client,
    pub secret: Option<String>,
}

#[tokio::main]
async fn main() {
    // init config from env
    let region = std::env::var("REGION").unwrap_or("eu-central-1".to_string());
    let access_key = std::env::var("ACCESS_KEY").expect("ACCESS_KEY not set in environment");
    let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY not set in environment");

    let config = aws_config::SdkConfig::builder()
        .use_dual_stack(true)
        .region(Region::new(region))
        .credentials_provider(SharedCredentialsProvider::new(StaticCredentials {
            access_key,
            secret_key,
        }))
        .build();

    let client = Client::new(&config);
    let cfg = Config {
        bucket_name: std::env::var("BUCKET_NAME").unwrap(),
        prefix: std::env::var("PREFIX").ok(),
        client,
        secret: std::env::var("SECRET").ok(),
    };
    let app = Router::new().route("/", get(handler)).with_state(cfg);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
