use std::marker::PhantomData;
use std::sync::Arc;

use egui::*;

use crate::color_projection::*;
use crate::coordinate_transforms::*;
use crate::oklch::*;

pub type StableHuePicker = ColorPicker2d<CartesianTransform, StableHueProjection>;

pub type StableChromaPicker = ColorPicker2d<PolarTransform, StableChromaProjection>;

pub type StableLightnessPicker = ColorPicker2d<PolarTransform, StableLightnessProjection>;

pub struct ColorPicker2d<T, P>
where
    T: CoordinateTransform,
    P: ColorProjection<T::Coord>,
{
    pub size: Vec2,
    pub viewport: [f32; 4],

    pub stable_value_bits: u32,

    pub texture: TextureHandle,

    /// Cached coordinate corresponding to every pixel.
    pub coordinates: Arc<[T::Coord]>,

    /// Cached border pixels.
    pub border_pixels: Arc<[Pos2]>,

    /// Last fallback coordinate.
    pub previous_fallback: Option<T::Coord>,

    pub projection: P,

    _transform: PhantomData<T>,
}

impl<T, P> ColorPicker2d<T, P>
where
    T: CoordinateTransform,
    P: ColorProjection<T::Coord>,
{
    pub fn new(ui: &Ui, size: Vec2, projection: P) -> Self {
        let texture = ui.ctx().load_texture(
            "picker_texture",
            ColorImage::new([1, 1], vec![Color32::TRANSPARENT]),
            TextureOptions::LINEAR,
        );

        let mut picker = Self {
            size,

            viewport: T::default_viewport(),

            stable_value_bits: 0,

            texture,

            coordinates: Arc::new([]),

            border_pixels: Arc::new([]),

            previous_fallback: None,

            projection,

            _transform: PhantomData,
        };

        picker.update_coordinates();
        picker.update_texture();

        picker
    }

    pub fn set_projection(&mut self, projection: P)
    where
        P: PartialEq,
    {
        if self.projection != projection {
            self.projection = projection;
            self.update_texture();
        }
    }

    pub fn pixel_to_world(&self, pixel: Pos2) -> Pos2 {
        // Normalize to [0, 1]
        let nx = pixel.x / self.size.x;
        let ny = 1.0 - pixel.y / self.size.y;

        // Interpolate into the viewport
        Pos2 {
            x: self.viewport[0] + nx * (self.viewport[2] - self.viewport[0]),
            y: self.viewport[1] + ny * (self.viewport[3] - self.viewport[1]),
        }
    }

    pub fn world_to_pixel(&self, world: Pos2) -> Pos2 {
        // Normalize within the viewport
        let nx = (world.x - self.viewport[0]) / (self.viewport[2] - self.viewport[0]);

        let ny = (world.y - self.viewport[1]) / (self.viewport[3] - self.viewport[1]);

        Pos2 {
            x: nx * self.size.x,
            y: (1.0 - ny) * self.size.y,
        }
    }

    pub fn pixel_to_coord(&self, pixel: Pos2) -> T::Coord {
        let world = self.pixel_to_world(pixel);
        T::world_to_coord(world)
    }

    pub fn coord_to_pixel(&self, coord: T::Coord) -> Pos2 {
        let world = T::coord_to_world(coord);
        self.world_to_pixel(world)
    }

    pub fn world_to_coord(&self, world: Pos2) -> T::Coord {
        T::world_to_coord(world)
    }

    pub fn coord_to_world(&self, coord: T::Coord) -> Pos2 {
        T::coord_to_world(coord)
    }

    pub fn update_coordinates(&mut self) {
        let width = self.size.x as usize;
        let height = self.size.y as usize;

        let mut coordinates = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                // Note: We can sample at pixel centers to avoid bias by adding 0.5 to x and y here,
                //       but that would mean we have to add the same offset in other places in the code
                //       to stay consistent. This turns out to be hard to oversee so we accept the bias.
                //       The most important thing is that the color shown on the panel matches the rest
                //       of the widget. It being drawn half a pixel off is not as relevant.
                let pixel = Pos2 {
                    x: x as f32,
                    y: y as f32,
                };

                coordinates.push(self.pixel_to_coord(pixel));
            }
        }

        self.coordinates = Arc::from(coordinates);
    }

    pub fn resize(&mut self, new_size: Vec2) {
        if self.size == new_size {
            return;
        }

        self.size = new_size;

        self.update_coordinates();

        self.update_texture();
    }

    pub fn update_texture(&mut self) {
        let width = self.size.x as usize;
        let height = self.size.y as usize;

        let mut pixels = vec![0u8; width * height * 4];

        for (i, coord) in self.coordinates.iter().enumerate() {
            let oklch = self.projection.color_at(*coord);

            if oklch.is_valid_srgb() {
                let pixel_i = i * 4;

                let srgb = oklch.to_srgb();

                pixels[pixel_i] = srgb[0];
                pixels[pixel_i + 1] = srgb[1];
                pixels[pixel_i + 2] = srgb[2];
                pixels[pixel_i + 3] = 255;
            }
        }

        self.border_pixels = Arc::from(Self::calculate_border_from_pixels(self.size, &pixels));

        let image = egui::ColorImage::from_rgba_unmultiplied([width, height], &pixels);

        self.texture.set(image, egui::TextureOptions::LINEAR);
    }

    pub fn update_viewport(&mut self, response: &Response, rect: Rect) -> bool {
        let width = self.viewport[2] - self.viewport[0];
        let height = self.viewport[3] - self.viewport[1];

        let mut viewport_changed = false;

        // Pan
        let is_pan_button_down = response.ctx.input(|i| {
            i.pointer.button_down(PointerButton::Secondary)
                || i.pointer.button_down(PointerButton::Middle)
        });

        if is_pan_button_down && response.dragged() {
            viewport_changed = true;

            let drag = response.drag_delta();

            let dx_world = drag.x / rect.width() * width;
            let dy_world = drag.y / rect.height() * height;

            // World space has +Y upwards.
            self.viewport[0] -= dx_world;
            self.viewport[2] -= dx_world;

            self.viewport[1] += dy_world;
            self.viewport[3] += dy_world;
        }

        // Pan from touch
        if let Some(mt) = response.ctx.input(|i| i.multi_touch()) {
            let drag = mt.translation_delta;

            let dx_world = drag.x / rect.width() * width;
            let dy_world = drag.y / rect.height() * height;

            self.viewport[0] -= dx_world;
            self.viewport[2] -= dx_world;

            self.viewport[1] += dy_world;
            self.viewport[3] += dy_world;

            viewport_changed = true;
        }

        // Zoom
        let scroll = response.ctx.input(|i| i.smooth_scroll_delta().y);

        if scroll.abs() > 0.0 {
            if let Some(mouse) = response.hover_pos() {
                let local = Pos2 {
                    x: mouse.x - rect.left(),
                    y: mouse.y - rect.top(),
                };

                let world = self.pixel_to_world(local);

                let zoom_factor = if scroll > 0.0 { 0.9 } else { 1.1 };

                const MAX_ZOOM_FACTOR: f32 = 7.0;

                let world_width = T::default_viewport()[2] - T::default_viewport()[0];
                let world_height = T::default_viewport()[3] - T::default_viewport()[1];

                let min_width = world_width / MAX_ZOOM_FACTOR;
                let min_height = world_height / MAX_ZOOM_FACTOR;

                let new_width = width * zoom_factor;
                let new_height = height * zoom_factor;

                if scroll > 0.0 && (new_width < min_width || new_height < min_height) {
                    return viewport_changed;
                }

                viewport_changed = true;
                self.viewport[0] = world.x + (self.viewport[0] - world.x) * zoom_factor;
                self.viewport[2] = world.x + (self.viewport[2] - world.x) * zoom_factor;
                self.viewport[1] = world.y + (self.viewport[1] - world.y) * zoom_factor;
                self.viewport[3] = world.y + (self.viewport[3] - world.y) * zoom_factor;
            }
        }

        // Zoom from touch
        if let Some(mt) = response.ctx.input(|i| i.multi_touch()) {
            let zoom_factor = mt.zoom_delta;

            let local = Pos2 {
                x: mt.center_pos.x - rect.left(),
                y: mt.center_pos.y - rect.top(),
            };

            let world = self.pixel_to_world(local);

            self.viewport[0] = world.x + (self.viewport[0] - world.x) / zoom_factor;
            self.viewport[2] = world.x + (self.viewport[2] - world.x) / zoom_factor;
            self.viewport[1] = world.y + (self.viewport[1] - world.y) / zoom_factor;
            self.viewport[3] = world.y + (self.viewport[3] - world.y) / zoom_factor;

            viewport_changed = true;
        }

        if viewport_changed {
            T::clamp_viewport(&mut self.viewport);
        }

        viewport_changed
    }

    pub fn calculate_border_from_pixels(size: Vec2, pixels: &[u8]) -> Vec<Pos2> {
        let width = size.x as usize;
        let height = size.y as usize;

        let mut border_pixels = Vec::new();

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let pixel_index = (y * width + x) * 4;

                let is_valid = pixels[pixel_index + 3] != 0;

                if !is_valid {
                    continue;
                }

                let mut touches_invalid = false;

                'neighbors: for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;

                        let neighbor_index = (ny * width + nx) * 4;

                        if pixels[neighbor_index + 3] == 0 {
                            touches_invalid = true;
                            break 'neighbors;
                        }
                    }
                }

                if touches_invalid {
                    border_pixels.push(Pos2 {
                        x: x as f32,
                        y: y as f32,
                    });
                }
            }
        }

        border_pixels
    }

    pub fn nearest_valid_border_pixel(&self, pixel: Pos2) -> Option<Pos2> {
        let mut nearest = None;
        let mut nearest_distance_sq = f32::MAX;

        for &border_pixel in self.border_pixels.iter() {
            let dx = pixel.x - border_pixel.x;
            let dy = pixel.y - border_pixel.y;

            let distance_sq = dx * dx + dy * dy;

            if distance_sq < nearest_distance_sq {
                nearest_distance_sq = distance_sq;
                nearest = Some(border_pixel);
            }
        }

        nearest
    }

    pub fn nearest_valid_border_coord(&self, coord: T::Coord) -> Option<T::Coord> {
        let pixel = self.coord_to_pixel(coord);

        let border_pixel = self.nearest_valid_border_pixel(pixel)?;

        Some(self.pixel_to_coord(border_pixel))
    }

    pub fn nearest_valid_color(&self, coord: T::Coord) -> Oklch {
        let mut oklch = self.projection.color_at(coord);

        if oklch.is_valid_srgb() {
            return oklch;
        }

        let Some(border_coord) = self.nearest_valid_border_coord(coord) else {
            oklch.alpha = 0.;
            return oklch;
        };

        return self.projection.color_at(border_coord);
    }

    pub fn offscreen_indicator(&self, coord: T::Coord) -> Option<(Pos2, f32)> {
        let marker = self.coord_to_pixel(coord);

        // Already visible.
        if marker.x >= 0.0 && marker.x <= self.size.x && marker.y >= 0.0 && marker.y <= self.size.y
        {
            return None;
        }

        // Clamp to the nearest point on the viewport boundary.
        const MARGIN: f32 = 10.0;
        let indicator = Pos2 {
            x: marker.x.clamp(MARGIN, self.size.x - MARGIN),
            y: marker.y.clamp(MARGIN, self.size.y - MARGIN),
        };

        // Direction from the indicator towards the hidden marker.
        let dx = marker.x - indicator.x;
        let dy = marker.y - indicator.y;

        let angle = dy.atan2(dx);

        Some((indicator, angle))
    }
}

impl<T, P> Clone for ColorPicker2d<T, P>
where
    T: CoordinateTransform,
    P: ColorProjection<T::Coord> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            size: self.size,
            viewport: self.viewport,
            stable_value_bits: self.stable_value_bits,
            texture: self.texture.clone(),
            coordinates: self.coordinates.clone(),
            border_pixels: self.border_pixels.clone(),
            previous_fallback: self.previous_fallback,
            projection: self.projection.clone(),
            _transform: PhantomData,
        }
    }
}
