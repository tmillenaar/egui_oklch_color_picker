use egui::Pos2;

pub trait CoordinateTransform {
    type Coord: Clone + Copy + PartialEq + Send + Sync;

    /// Convert from world coordinates to the coordinate system used by this picker.
    fn world_to_coord(world: Pos2) -> Self::Coord;

    /// Convert from the picker's coordinate system back into world coordinates.
    fn coord_to_world(coord: Self::Coord) -> Pos2;
}

#[derive(Clone, Copy, PartialEq)]
pub struct CartesianCoord {
    pub x: f32,
    pub y: f32,
}

pub struct CartesianTransform;

impl CoordinateTransform for CartesianTransform {
    type Coord = CartesianCoord;

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

    fn world_to_coord(world: Pos2) -> PolarCoord {
        let radius = (world.x * world.x + world.y * world.y).sqrt();

        let mut angle = (-world.y).atan2(world.x);

        if angle < 0.0 {
            angle += std::f32::consts::TAU;
        }

        PolarCoord { radius, angle }
    }

    fn coord_to_world(coord: PolarCoord) -> Pos2 {
        Pos2 {
            x: coord.radius * coord.angle.cos(),
            y: -coord.radius * coord.angle.sin(),
        }
    }
}
