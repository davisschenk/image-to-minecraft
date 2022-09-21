use image::Rgba;
use kdtree::KdTree;
use palette::{ColorDifference, IntoColor, Laba, Pixel, Srgba};

#[derive(Debug)]
pub struct Colors {
    tree: KdTree<f32, String, Vec<f32>>,
}

impl Colors {
    pub fn new() -> Self {
        Colors {
            tree: KdTree::new(4),
        }
    }

    pub fn add(&mut self, color: Rgba<f32>, path: String) {
        let lab: Laba = Srgba::from_raw_slice(&color.0)[0].into_color();
        let (l, a, b, al) = lab.into_components();
        self.tree.add(vec![l, a, b, al], path).unwrap();
    }

    fn closest_lab(a: &[f32], b: &[f32]) -> f32 {
        let lab_a: Laba = Srgba::from_raw_slice(a)[0].into_color();
        let lab_b: Laba = Srgba::from_raw_slice(b)[0].into_color();

        lab_a.get_color_difference(&lab_b)
    }

    pub fn closest(&self, color: Rgba<f32>) -> &str {
        let Rgba(v) = color;
        self.tree.nearest(&v, 1, &Self::closest_lab).unwrap()[0].1
    }
}
