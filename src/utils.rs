use crate::Config;
use flate2::{write::ZlibEncoder, Compression};
use image::{DynamicImage, GrayImage};
use rand::Rng;
use std::io::Write;

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

pub fn compress(raw: Vec<u8>) -> Vec<u8> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::new(9));
    e.write_all(&raw).unwrap();
    e.finish().unwrap()
}

pub async fn get_random_image(config: &Config) -> (String, Vec<u8>) {
    let result = config
        .client
        .list_objects_v2()
        .bucket(config.bucket_name.clone())
        .prefix(config.prefix.clone().unwrap_or_default())
        .send()
        .await
        .unwrap();
    let count = result.key_count.unwrap();
    let n = {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..count)
    };
    let all = result.contents();
    let obj = all.get(n as usize).unwrap();
    let key = obj.key.clone().unwrap();
    println!("downloading {key}");
    let result = config
        .client
        .get_object()
        .bucket(config.bucket_name.clone())
        .key(&key)
        .send()
        .await
        .unwrap();
    let content = result.body.collect().await.unwrap().to_vec();
    println!("fetched {} bytes", content.len());
    (key, content)
}

pub fn generate_white_image() -> image::ImageBuffer<image::Luma<u8>, Vec<u8>> {
    let mut t: image::ImageBuffer<image::Luma<u8>, Vec<u8>> = GrayImage::new(1600, 1200);

    for x in t.iter_mut() {
        *x = 255;
    }
    t
}
