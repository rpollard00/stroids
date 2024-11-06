use bevy::{prelude::*, render::render_resource::TextureFormat};

// pub type Hull = Vec<Vec2>;
#[derive(Clone, Default)]
pub struct Hull(pub Vec<Vec2>);

impl Hull {
    pub fn new(image: &Image) -> Hull {
        let image_vec = extract_visible_pixels(&image);

        convex_hull(&image_vec)
    }

    pub fn draw_as_lines(self: &Self, gizmos: &mut Gizmos, color: Color) {
        let Hull(pts) = self;
        for line in pts.windows(2) {
            let start = line[0];
            let end = line[1];
            gizmos.line_2d(start, end, color);
        }
    }
}

#[derive(Eq, PartialEq)]
enum Orientation {
    Collinear,
    Clockwise,
    CounterClockwise,
}

fn orientation(a: Vec2, b: Vec2, c: Vec2) -> Orientation {
    let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);

    if cross == 0. {
        Orientation::Collinear
    } else if cross > 0. {
        Orientation::CounterClockwise
    } else {
        Orientation::Clockwise
    }
}

fn convex_hull(pixel_data: &Vec<Vec2>) -> Hull {
    // scan for the minimum, leftmost coord, i don't know if this is the best but for now we will
    // consider 0,0 to be the smallest possible coord
    //
    let mut origin: Vec2 = Vec2::MAX;

    // let origin_index: usize = 0;

    // scan and find the minimum origin
    for pt in pixel_data {
        if pt.y < origin.y || (pt.y == origin.y && pt.x < origin.x) {
            origin = *pt;
        }
    }

    let mut sorted_pixel_data: Vec<Vec2> = pixel_data
        .iter()
        .filter(|pt| pt.x != origin.x && pt.y != origin.y)
        .cloned()
        .collect();

    sorted_pixel_data.sort_by(|a, b| {
        let angle_a = angle_from_vec(a.x - origin.x, a.y - origin.y);
        let angle_b = angle_from_vec(b.x - origin.x, b.y - origin.y);

        angle_a.partial_cmp(&angle_b).unwrap()
    });

    let mut hull: Vec<Vec2> = vec![origin];

    for pt in sorted_pixel_data {
        while hull.len() > 1
            && orientation(hull[hull.len() - 2], hull[hull.len() - 1], pt) == Orientation::Clockwise
        {
            hull.pop();
        }
        hull.push(pt);
    }

    hull.push(*hull.first().clone().unwrap());

    Hull(hull)
}

fn angle_from_vec(x: f32, y: f32) -> f32 {
    y.atan2(x)
}

fn extract_visible_pixels(image: &Image) -> Vec<Vec2> {
    let mut visible_points = Vec::new();

    let pixel_data = &image.data;
    let width = image.texture_descriptor.size.width as usize;
    let height = image.texture_descriptor.size.height as usize;

    if image.texture_descriptor.format == TextureFormat::Rgba8UnormSrgb {
        // vec of u8, each pixel is made up of [RGBA], each value is a u8
        // ex width is 4
        //    height is 4
        //  first pixel would be  (y(0) * image_width(4) + x(0) * 4(size of pixel data)) = 0
        //  5th pixel would be y = 1 * 4 = 4 + 0 = 4 * 4 = 16
        //  alpha of that is the last component from the calculated position so its index + 3
        //
        //          0    1    2    3 --> x
        // so its 0 RGBA,RGBA,RGBA,RGBA
        // so its 1 RGBA,RGBA,RGBA,RGBA
        // so its 2 RGBA,RGBA,RGBA,RGBA
        // so its 3 RGBA,RGBA,RGBA,RGBA
        //        y
        // row 1(second row) pixel 15(0 index)
        // y = 1
        // x = 15
        for y in 0..height {
            for x in 0..width {
                let pixel_index = (y * width + x) * 4;
                let pixel_alpha = pixel_data[pixel_index + 3];

                // realign the pixels around the center of the image
                let x_f = (x as f32) - (width as f32) / 2.;
                let y_f = (y as f32) - (height as f32) / 2.;

                if pixel_alpha > 0 {
                    visible_points.push(Vec2::new(x_f as f32, y_f as f32))
                }
            }
        }
    }

    visible_points
}
