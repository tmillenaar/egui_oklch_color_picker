use egui::*;

pub fn draw_triangle(
    painter: &Painter,
    tip: Pos2,
    angle: f32, // radians, 0 = +X
    size: f32,  // side length of equilateral triangle
    fill: Color32,
    stroke: Stroke,
) {
    let height = size * (3.0_f32).sqrt() * 0.5;

    // Direction the triangle points.
    let dir = Vec2::new(angle.cos(), angle.sin());

    // Perpendicular to the direction.
    let perp = Vec2::new(-dir.y, dir.x);

    // Base is behind the tip.
    let base_center = tip - dir * height;

    let left = base_center + perp * (size * 0.5);
    let right = base_center - perp * (size * 0.5);

    painter.add(Shape::convex_polygon(vec![tip, left, right], fill, stroke));
}

pub fn draw_inward_triangles(
    painter: &Painter,
    center: Pos2,
    radius: f32, // distance from center to inner tip
    size: f32,   // side length of equilateral triangle
    count: usize,
    offset_deg: f32,
    fill: Color32,
    stroke: Stroke,
) {
    if count == 0 {
        return;
    }

    let height = size * (3.0_f32).sqrt() * 0.5;

    let base_angle = std::f32::consts::TAU * 0.75; // bottom
    let offset = offset_deg.to_radians();
    let step = std::f32::consts::TAU / count as f32;

    for i in 0..count {
        let angle = base_angle + offset + step * i as f32;

        let outward = Vec2::new(angle.cos(), angle.sin());

        // Inner tip sits exactly radius units from center.
        let tip = center + outward * radius;

        let base_center = tip + outward * height;

        let perp = Vec2::new(-outward.y, outward.x);

        let left = base_center + perp * (size * 0.5);
        let right = base_center - perp * (size * 0.5);

        painter.add(Shape::convex_polygon(vec![tip, left, right], fill, stroke));
    }
}
