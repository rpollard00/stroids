use bevy::{prelude::*, render::render_resource::TextureFormat};

#[derive(Component, Clone, Default)]
pub struct Hull {
    path: Vec<Vec2>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct HullCollision;

#[derive(Clone, Copy, Debug)]
struct Projection(f32, f32);
impl Projection {
    fn overlaps(&self, other: &Projection) -> bool {
        self.0 <= other.1 && self.1 >= other.0
    }
}

impl Hull {
    pub fn new(image: &Image) -> Hull {
        let image_vec = extract_visible_pixels(&image);

        convex_hull(&image_vec)
    }

    pub fn draw_as_lines(
        self: &Self,
        gizmos: &mut Gizmos,
        color: Color,
        position: &Vec2,
        rotation: &Quat,
    ) {
        let Hull { path: pts } = self;
        for line in pts.windows(2) {
            let start = (*rotation * line[0].extend(0.0)).truncate() + *position;
            let end = (*rotation * line[1].extend(0.0)).truncate() + *position;
            gizmos.line_2d(start, end, color);
        }
    }

    pub fn from_bb(tr: Vec2, tl: Vec2, br: Vec2, bl: Vec2) -> Hull {
        let mut path: Vec<Vec2> = Vec::new();

        path.push(tr);
        path.push(tl);
        path.push(bl);
        path.push(br);

        Hull { path }
    }
}

pub fn check_for_collision(
    hull_a: &Hull,
    hull_a_transform: &Transform,
    hull_b: &Hull,
    hull_b_transform: &Transform,
) -> Option<HullCollision> {
    let hull_a = hull_at_orientation(&hull_a, &hull_a_transform);
    let hull_b = hull_at_orientation(&hull_b, &hull_b_transform);

    let hull_a_axes = get_all_normals(&hull_a)
        .into_iter()
        .chain(get_all_normals(&hull_b));

    for axis in hull_a_axes {
        let proj_a = project(&hull_a, &axis);
        let proj_b = project(&hull_b, &axis);

        if !proj_a.overlaps(&proj_b) {
            return None;
        }
    }

    Some(HullCollision)
}

pub fn project(shape: &Vec<Vec2>, axis: &Vec2) -> Projection {
    let mut max_p = f32::MIN;
    let mut min_p = f32::MAX;

    for v in shape.iter() {
        let p = axis.dot(*v);

        min_p = min_p.min(p);
        max_p = max_p.max(p);
    }

    return Projection(min_p, max_p);
}

pub fn hull_at_orientation(hull: &Hull, transform: &Transform) -> Vec<Vec2> {
    let Hull { path: raw_vertices } = hull;

    // vertices need to be transformed by their rotation and position
    let mut vertices = Vec::<Vec2>::new();
    let translation = &transform.translation;
    let rotation = &transform.rotation;

    for v in raw_vertices {
        let v_oriented = (*rotation * v.extend(0.0)) + *translation;
        vertices.push(v_oriented.truncate());
    }

    vertices
}

pub fn normalized_normal(v1: Vec2, v2: Vec2) -> Vec2 {
    let edge = v1 - v2;
    edge.perp().normalize()
}

pub fn get_all_normals(pts: &Vec<Vec2>) -> Vec<Vec2> {
    let mut normals = Vec::<Vec2>::new();
    for edge in pts.windows(2) {
        normals.push(normalized_normal(edge[0], edge[1]))
    }
    normals
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
    let mut origin: Vec2 = Vec2::MAX;

    // scan and find the minimum origin
    for pt in pixel_data {
        if pt.y < origin.y || (pt.y == origin.y && pt.x < origin.x) {
            origin = *pt;
        }
    }

    let mut sorted_pts: Vec<Vec2> = pixel_data
        .iter()
        .filter(|&&pt| pt != origin)
        .cloned()
        .collect();

    sorted_pts.sort_by(|a, b| {
        let angle_a = angle_from_vec(a.x - origin.x, a.y - origin.y);
        let angle_b = angle_from_vec(b.x - origin.x, b.y - origin.y);

        if angle_a == angle_b {
            let dist_a = (*a - origin).length_squared();
            let dist_b = (*b - origin).length_squared();

            dist_a.partial_cmp(&dist_b).unwrap()
        } else {
            angle_a.partial_cmp(&angle_b).unwrap()
        }
    });

    let mut hull: Vec<Vec2> = vec![origin];

    for pt in sorted_pts {
        while hull.len() > 1 {
            let last_pt = hull[hull.len() - 1];
            let second_last_pt = hull[hull.len() - 2];

            match orientation(second_last_pt, last_pt, pt) {
                Orientation::CounterClockwise => break,
                Orientation::Clockwise => {
                    hull.pop();
                }
                Orientation::Collinear => {
                    if (pt - second_last_pt).length_squared()
                        > (last_pt - second_last_pt).length_squared()
                    {
                        hull.pop();
                    } else {
                        break;
                    }
                }
            };
        }
        hull.push(pt);
    }

    hull.push(*hull.first().clone().unwrap());

    Hull { path: hull }
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

                // appears y coord needs a flip for some reason
                // for some reason these are constructed differently from the textures so they were
                // not matching
                if pixel_alpha > 0 {
                    visible_points.push(Vec2::new(x_f as f32, -y_f as f32))
                }
            }
        }
    }

    visible_points
}
