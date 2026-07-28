use egui::*;

pub trait CoordinateTransform {
    type Coord: Clone + Copy + PartialEq + Send + Sync;

    fn default_viewport() -> [f32; 4];

    /// Convert from world coordinates to the coordinate system used by this picker.
    fn world_to_coord(world: Pos2) -> Self::Coord;

    /// Convert from the picker's coordinate system back into world coordinates.
    fn coord_to_world(coord: Self::Coord) -> Pos2;

    fn draw_axes_on_background(
        painter: &Painter,
        world_to_pixel: impl Fn(Pos2) -> Pos2,
        rect: Rect,
        viewport: [f32; 4],
    ) {
        // default: nothing
    }

    fn clamp_viewport(viewport: &mut [f32; 4]) {
        let default_viewport = Self::default_viewport();

        let mut min_x = viewport[0];
        let mut min_y = viewport[1];
        let mut max_x = viewport[2];
        let mut max_y = viewport[3];

        let width = (max_x - min_x).min(default_viewport[2] - default_viewport[0]);
        let height = (max_y - min_y).min(default_viewport[3] - default_viewport[1]);

        let cx = (min_x + max_x) * 0.5;
        let cy = (min_y + max_y) * 0.5;

        min_x = cx - width * 0.5;
        max_x = cx + width * 0.5;

        min_y = cy - height * 0.5;
        max_y = cy + height * 0.5;

        if min_x < default_viewport[0] {
            let d = default_viewport[0] - min_x;
            min_x += d;
            max_x += d;
        }

        if max_x > default_viewport[2] {
            let d = default_viewport[2] - max_x;
            min_x += d;
            max_x += d;
        }

        if min_y < default_viewport[1] {
            let d = default_viewport[1] - min_y;
            min_y += d;
            max_y += d;
        }

        if max_y > default_viewport[3] {
            let d = default_viewport[3] - max_y;
            min_y += d;
            max_y += d;
        }

        viewport[0] = min_x;
        viewport[1] = min_y;
        viewport[2] = max_x;
        viewport[3] = max_y;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct CartesianCoord {
    pub x: f32,
    pub y: f32,
}

pub struct CartesianTransform;

impl CoordinateTransform for CartesianTransform {
    type Coord = CartesianCoord;

    fn default_viewport() -> [f32; 4] {
        [0., 0., 1., 1.]
    }

    fn world_to_coord(world: Pos2) -> CartesianCoord {
        CartesianCoord {
            x: world.x,
            y: world.y,
        }
    }

    fn coord_to_world(coord: CartesianCoord) -> Pos2 {
        Pos2::new(coord.x, coord.y)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct PolarCoord {
    pub radius: f32, // 0..1
    pub angle: f32,  // radians
}

pub struct PolarTransform;

impl CoordinateTransform for PolarTransform {
    type Coord = PolarCoord;

    fn default_viewport() -> [f32; 4] {
        [-1., -1., 1., 1.]
    }

    fn world_to_coord(world: Pos2) -> PolarCoord {
        let radius = (world.x * world.x + world.y * world.y).sqrt();

        let mut angle = (world.y).atan2(world.x);

        if angle < 0.0 {
            angle += std::f32::consts::TAU;
        }

        PolarCoord { radius, angle }
    }

    fn coord_to_world(coord: PolarCoord) -> Pos2 {
        Pos2 {
            x: coord.radius * coord.angle.cos(),
            y: coord.radius * coord.angle.sin(),
        }
    }

    fn draw_axes_on_background(
        painter: &Painter,
        world_to_pixel: impl Fn(Pos2) -> Pos2,
        rect: Rect,
        viewport: [f32; 4],
    ) {
        let center = world_to_pixel(Pos2::ZERO);

        let world_to_screen = rect.width() / (viewport[2] - viewport[0]);

        for i in 0..6 {
            let world_radius = 0.01 + 0.15 * i as f32;

            painter.circle(
                center + rect.min.to_vec2(),
                world_radius * world_to_screen,
                Color32::TRANSPARENT,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(200, 200, 200, 100)),
            );
        }
    }
}
