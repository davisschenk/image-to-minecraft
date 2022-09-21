use image::{io::Reader as ImageReader, RgbaImage};
use kmeans_colors::{get_kmeans_hamerly, Kmeans, Sort};
use palette::{IntoColor, Lab, LinSrgb, LinSrgba, Pixel, Srgba};

mod tree;

const RUNS: usize = 100;
const K: usize = 5;
const MAX_ITER: usize = usize::MAX;
const CONVERGE: f32 = 0.0025;
const SEED: u64 = 0;

fn get_color(path: &str) -> image::Rgba<f32> {
    let texture = ImageReader::open(path).unwrap().decode().unwrap();

    let lab: Vec<Lab> = Srgba::from_raw_slice(texture.as_bytes())
        .iter()
        .map(|x| x.into_format::<_, f32>().into_color())
        .collect();

    let mut result: Kmeans<Lab> = Kmeans::new();
    for i in 0..RUNS {
        let run_result = get_kmeans_hamerly(K, MAX_ITER, CONVERGE, false, &lab, SEED + i as u64);

        if run_result.score < result.score {
            result = run_result
        }
    }

    let res = Lab::sort_indexed_colors(&result.centroids, &result.indices);
    let dom: Srgba = Lab::get_dominant_color(&res).unwrap().into_color();

    image::Rgba([dom.red, dom.green, dom.blue, dom.alpha])
}

fn blockify(colors: &tree::Colors, input: &str, output: &str) {
    eprintln!("Blockifying {input} -> {output}");
    let start = std::time::Instant::now();
    let input_image = ImageReader::open(input).unwrap().decode().unwrap();
    let input_image = input_image.into_rgba32f();
    // let input_image = image::imageops::crop(&mut input_image, 500, 0, 800, 800).to_image();

    let mut output_image =
        image::RgbaImage::new(input_image.width() * 16, input_image.height() * 16);

    for (x, y, pixel) in input_image.enumerate_pixels() {
        if pixel[3] != 1.0 {
            continue;
        }

        let path = colors.closest(*pixel);
        let texture = ImageReader::open(path).unwrap().decode().unwrap();
        image::imageops::overlay(
            &mut output_image,
            &texture,
            (x * 16).into(),
            (y * 16).into(),
        )
    }

    output_image.save(output).unwrap();

    let end = std::time::Instant::now();
    eprintln!("Blockifying {} took {:.2?}", input, end - start);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut colors = tree::Colors::new();

    for texture in std::fs::read_dir("data/images")? {
        if let Ok(path) = texture {
            let start = std::time::Instant::now();
            let path: String = path.path().display().to_string();
            let color = get_color(&path);
            let end = std::time::Instant::now();

            eprintln!(
                "Finding the dominant color of {} took {:.2?}. Color: {:?}",
                path,
                end - start,
                color
            );

            let mut new_image = RgbaImage::new(32, 16);
            image::imageops::overlay(
                &mut new_image,
                &ImageReader::open(&path).unwrap().decode().unwrap(),
                0,
                0,
            );

            let mut fill = image::RgbaImage::new(16, 16);
            for px in fill.pixels_mut() {
                let image::Rgba(v) = &color;
                *px = image::Rgba([
                    (v[0] * 255.0) as u8,
                    (v[1] * 255.0) as u8,
                    (v[2] * 255.0) as u8,
                    (v[3] * 255.0) as u8,
                ])
            }

            image::imageops::overlay(&mut new_image, &fill, 16, 0);
            let s_path = format!("out/{}", path.rsplit_once("/").unwrap().1);
            println!("{s_path}");
            new_image.save(s_path).unwrap();

            colors.add(color, path);
        }
    }

    // blockify(&colors, "fractal.png", "out.png");

    let grad = palette::Gradient::new(vec![
        LinSrgb::new(1.0f32, 0.0, 0.0),
        LinSrgb::new(0.0, 1.0, 0.0),
        LinSrgb::new(0.0, 0.0, 1.0),
    ]);


    let mut gi = image::RgbImage::new(256, 1);

    for (px, color) in gi.pixels_mut().zip(grad.take(256)) {
        *px = image::Rgb([
            (color.red * 255.0) as u8,
            (color.green * 255.0) as u8,
            (color.blue * 255.0) as u8,
        ])
    }

    gi.save("grad.png");
    blockify(&colors, "grad.png", "grad_mc.png");

    Ok(())
}
