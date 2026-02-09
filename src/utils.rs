use crate::Config;
use cached::proc_macro::cached;
use flate2::{Compression, write::ZlibEncoder};
use image::{DynamicImage, GrayImage};
use rand::RngExt;
use std::io::Write;
use std::time::Duration;

pub fn resize(img: DynamicImage) -> DynamicImage {
    img.resize(1600, 1200, image::imageops::FilterType::Nearest)
}

pub fn to_raw(image: Vec<u8>) -> Vec<u8> {
    let mut buf = vec![];
    let mut pixel: u8 = 0;
    for (idx, dot) in image.iter().enumerate() {
        if idx % 2 == 0 {
            pixel |= dot / 16;
        } else {
            pixel |= (dot / 16) << 4;
            buf.push(pixel);
            pixel = 0;
        }
    }
    buf
}

/// compress the byte array with zlib and conpression level 9
pub fn compress(raw: Vec<u8>) -> Vec<u8> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::new(1));
    e.write_all(&raw).unwrap();
    e.finish().unwrap()
}

pub async fn get_random_image(config: &Config) -> (String, Vec<u8>) {
    let all = get_s3_listing(config).await;
    let count = all.len();
    let n = {
        let mut rng = rand::rng();
        rng.random_range(0..count)
    };
    let key = all.get(n as usize).unwrap();
    println!("downloading {key}");
    let result = config
        .client
        .get_object()
        .bucket(config.bucket_name.clone())
        .key(key)
        .send()
        .await
        .unwrap();
    let content = result.body.collect().await.unwrap().to_vec();
    println!("fetched {} bytes", content.len());
    (key.clone(), content)
}

pub async fn cache_refresh(config: Config) {
    loop {
        get_s3_listing(&config).await;
        tokio::time::sleep(Duration::from_secs(86400)).await;
    }
}

/// cache the s3 listing for 3 days
#[cached(time = 259200, key = "bool", convert = r#"{ true }"#)]
async fn get_s3_listing(config: &Config) -> Vec<String> {
    let mut result = vec![];
    let mut continuation_token = None;

    loop {
        let list_output = config
            .client
            .list_objects_v2()
            .bucket(config.bucket_name.clone())
            .prefix(config.prefix.clone().unwrap_or_default())
            .set_continuation_token(continuation_token)
            .send()
            .await
            .unwrap();

        for obj in list_output.contents() {
            result.push(obj.key.clone().unwrap());
        }

        if list_output.next_continuation_token.is_none() {
            break;
        }
        continuation_token = list_output.next_continuation_token;
    }

    result
}

pub fn generate_white_image() -> image::ImageBuffer<image::Luma<u8>, Vec<u8>> {
    let mut t: image::ImageBuffer<image::Luma<u8>, Vec<u8>> = GrayImage::new(1600, 1200);

    for x in t.iter_mut() {
        *x = 255;
    }
    t
}
