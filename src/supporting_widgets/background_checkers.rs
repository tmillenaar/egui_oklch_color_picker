use egui::*;

/// In rust, % is the remainder, not the modulus
fn modulus(val: f32, modulus: f32) -> f32 {
    ((val % modulus) + modulus) % modulus
}

pub fn background_checkers(painter: &Painter, rect: Rect, ny: u32) {
    let dark_color = Color32::from_gray(90);
    let bright_color = Color32::from_gray(190);

    // Adjust rect size and checker size to match
    let target_checker_size = rect.height() / ny as f32;
    let checker_size = target_checker_size.floor();
    let overshoot_y = modulus(rect.height(), checker_size);
    let overshoot_x = modulus(rect.width(), checker_size);
    let rect = rect.shrink2(Vec2::new(overshoot_x / 2., overshoot_y / 2.));

    if !rect.is_positive() {
        return;
    }

    let nx = (rect.width() / checker_size).round() as u32;

    let mut mesh = Mesh::default();
    mesh.add_colored_rect(rect, dark_color);

    let mut top = true;
    for i_y in 0..ny {
        for mut i_x in (0..nx).step_by(2) {
            // Offset every other cell
            if modulus(i_x as f32 + i_y as f32, 2.) == 0. {
                i_x += 1;
                if i_x >= nx {
                    break;
                }
            }
            let x = lerp(rect.left()..=rect.right(), i_x as f32 / nx as f32);
            let cell_top = rect.top() + i_y as f32 * checker_size;
            let cell_rect = Rect::from_min_size(pos2(x, cell_top), Vec2::splat(checker_size));
            mesh.add_colored_rect(cell_rect, bright_color);
            top = !top;
        }
    }
    painter.add(Shape::mesh(mesh));
}
