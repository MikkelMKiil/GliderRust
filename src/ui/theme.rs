use std::sync::Arc;

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, Pos2, Shape, Stroke};

// ── Background layers ─────────────────────────────────────────────────────────
pub const BG_BASE: Color32 = Color32::from_rgb(7, 5, 15);

// ── Glass material ────────────────────────────────────────────────────────────
/// Primary glass fill: dark purple-tinted, mostly opaque
pub const GLASS_FILL: Color32 = Color32::from_rgba_premultiplied(28, 24, 55, 210);
/// Raised glass (cards on top of glass surfaces)
pub const GLASS_FILL_RAISED: Color32 = Color32::from_rgba_premultiplied(38, 33, 72, 215);
/// Inset / recessed glass (inputs, info areas)
pub const GLASS_FILL_INSET: Color32 = Color32::from_rgba_premultiplied(14, 12, 30, 200);
/// Glass panel border — white at low alpha (rim lighting)
pub const GLASS_BORDER: Color32 = Color32::from_rgba_premultiplied(65, 58, 105, 255);
/// Bright rim highlight at very top of panel (specular gleam)
pub const GLASS_SPECULAR: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 95);
/// Tab bar background
pub const TAB_BAR_FILL: Color32 = Color32::from_rgba_premultiplied(11, 9, 22, 240);

// ── Text ──────────────────────────────────────────────────────────────────────
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(232, 228, 255);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(130, 118, 175);
pub const TEXT_DIM: Color32 = Color32::from_rgb(72, 64, 104);

// ── Accent colours ────────────────────────────────────────────────────────────
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(90, 172, 255);
pub const ACCENT_BLUE_DIM: Color32 = Color32::from_rgba_premultiplied(18, 52, 112, 200);
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(60, 210, 150);
pub const ACCENT_RED: Color32 = Color32::from_rgb(255, 95, 95);
pub const ACCENT_YELLOW: Color32 = Color32::from_rgb(255, 200, 58);

// ── Tab button colours ────────────────────────────────────────────────────────
pub const TAB_ACTIVE_FILL: Color32 = Color32::from_rgba_premultiplied(22, 66, 160, 220);
pub const TAB_ACTIVE_BORDER: Color32 = Color32::from_rgba_premultiplied(90, 172, 255, 200);
pub const TAB_INACTIVE_FILL: Color32 = Color32::from_rgba_premultiplied(20, 17, 40, 180);
pub const TAB_INACTIVE_BORDER: Color32 = Color32::from_rgba_premultiplied(60, 52, 90, 180);

// ── Health bar colour helper ──────────────────────────────────────────────────
pub fn health_color(ratio: f32) -> Color32 {
    let ratio = ratio.clamp(0.0, 1.0);
    if ratio >= 0.5 {
        let t = (ratio - 0.5) * 2.0;
        lerp_color(Color32::from_rgb(255, 195, 48), Color32::from_rgb(60, 210, 150), t)
    } else {
        let t = ratio * 2.0;
        lerp_color(Color32::from_rgb(255, 78, 78), Color32::from_rgb(255, 195, 48), t)
    }
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        lerp_u8(a.r(), b.r(), t),
        lerp_u8(a.g(), b.g(), t),
        lerp_u8(a.b(), b.b(), t),
    )
}
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

// ── Frame constructors ────────────────────────────────────────────────────────
pub fn glass_frame() -> Frame {
    Frame::new()
        .fill(GLASS_FILL)
        .inner_margin(Margin::same(16))
        .corner_radius(CornerRadius::same(20))
        .stroke(Stroke::new(1.0, GLASS_BORDER))
}

pub fn glass_frame_raised() -> Frame {
    Frame::new()
        .fill(GLASS_FILL_RAISED)
        .inner_margin(Margin::same(16))
        .corner_radius(CornerRadius::same(20))
        .stroke(Stroke::new(1.0, GLASS_BORDER))
}

pub fn glass_frame_inset() -> Frame {
    Frame::new()
        .fill(GLASS_FILL_INSET)
        .inner_margin(Margin::same(12))
        .corner_radius(CornerRadius::same(12))
        .stroke(Stroke::new(1.0, GLASS_BORDER))
}

/// Draw a glass card and then paint a soft top glint to simulate
/// liquid-glass rim lighting without a hard white line.
pub fn glass_card<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let inner = glass_frame().show(ui, add_contents);
    paint_specular(ui, inner.response.rect, 20);
    inner.inner
}

pub fn glass_card_raised<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let inner = glass_frame_raised().show(ui, add_contents);
    paint_specular(ui, inner.response.rect, 20);
    inner.inner
}

pub fn paint_card_top_glint(ui: &egui::Ui, rect: egui::Rect, _corner_r: u8) {
    paint_top_glint(ui, rect, 0.52, 2.8, 6.8, 1.8, 18, 24);
}

pub fn paint_button_top_glint(ui: &egui::Ui, rect: egui::Rect) {
    paint_top_glint(ui, rect, 0.46, 6.2, 5.0, 1.4, 14, 20);
}

pub fn paint_top_glint(
    ui: &egui::Ui,
    rect: egui::Rect,
    width_ratio: f32,
    top_offset: f32,
    soft_height: f32,
    core_height: f32,
    soft_alpha: u8,
    core_alpha: u8,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }

    let max_width = (rect.width() - 6.0).max(10.0);
    let glint_width = (rect.width() * width_ratio).clamp(10.0, max_width);
    if glint_width <= 0.0 {
        return;
    }

    let center_x = rect.center().x;
    let left = center_x - glint_width * 0.5;
    let right = center_x + glint_width * 0.5;
    let soft_top = rect.top() + top_offset;

    paint_glint_strip(ui, rect, left, right, soft_top, soft_height, soft_alpha, 0.24);

    let core_inset = glint_width * 0.16;
    let core_top = soft_top + ((soft_height - core_height) * 0.25).clamp(0.0, 2.0);
    paint_glint_strip(
        ui,
        rect,
        left + core_inset,
        right - core_inset,
        core_top,
        core_height,
        core_alpha,
        0.35,
    );
}

fn paint_glint_strip(
    ui: &egui::Ui,
    clip_rect: egui::Rect,
    mut left: f32,
    mut right: f32,
    y_top: f32,
    height: f32,
    alpha: u8,
    edge_fade_ratio: f32,
) {
    if alpha == 0 || height <= 0.0 {
        return;
    }

    left = left.max(clip_rect.left());
    right = right.min(clip_rect.right());
    if right - left < 2.0 {
        return;
    }

    let y0 = y_top.max(clip_rect.top());
    let y1 = (y_top + height).min(clip_rect.bottom());
    if y1 <= y0 {
        return;
    }

    let fade = ((right - left) * edge_fade_ratio).clamp(2.0, (right - left) * 0.45);
    let x1 = (left + fade).min(right);
    let x2 = (right - fade).max(x1);

    let top_color = Color32::from_rgba_premultiplied(255, 255, 255, alpha);
    let transparent = Color32::from_rgba_premultiplied(0, 0, 0, 0);

    let mut mesh = egui::epaint::Mesh::default();
    let top_row = [
        egui::pos2(left, y0),
        egui::pos2(x1, y0),
        egui::pos2(x2, y0),
        egui::pos2(right, y0),
    ];
    let bottom_row = [
        egui::pos2(left, y1),
        egui::pos2(x1, y1),
        egui::pos2(x2, y1),
        egui::pos2(right, y1),
    ];

    for (i, pos) in top_row.into_iter().enumerate() {
        let color = if i == 0 || i == 3 { transparent } else { top_color };
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv: egui::pos2(0.0, 0.0),
            color,
        });
    }

    for pos in bottom_row {
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv: egui::pos2(0.0, 0.0),
            color: transparent,
        });
    }

    for i in 0..3u32 {
        let t0 = i;
        let t1 = i + 1;
        let b0 = i + 4;
        let b1 = i + 5;
        mesh.indices.extend_from_slice(&[t0, t1, b1, t0, b1, b0]);
    }

    ui.painter().add(Shape::Mesh(Arc::new(mesh)));
}

fn paint_specular(ui: &egui::Ui, rect: egui::Rect, corner_r: u8) {
    paint_card_top_glint(ui, rect, corner_r);
}

// ── Ambient background ────────────────────────────────────────────────────────
/// Paints the dynamic colour-blob background behind all glass panels.
pub fn paint_background(ui: &egui::Ui, rect: egui::Rect) {
    let p = ui.painter_at(rect);

    // Base fill
    p.rect_filled(rect, CornerRadius::ZERO, BG_BASE);

    let w = rect.width();
    let h = rect.height();

    // Electric-blue blob — upper-left quadrant
    paint_blob(
        &p,
        rect.min + egui::vec2(w * 0.12, h * 0.22),
        w * 0.38,
        Color32::from_rgba_premultiplied(10, 48, 180, 140),
    );

    // Deep-purple blob — lower-right
    paint_blob(
        &p,
        rect.min + egui::vec2(w * 0.82, h * 0.78),
        w * 0.35,
        Color32::from_rgba_premultiplied(110, 18, 190, 120),
    );

    // Teal accent — upper-right
    paint_blob(
        &p,
        rect.min + egui::vec2(w * 0.88, h * 0.14),
        w * 0.22,
        Color32::from_rgba_premultiplied(0, 150, 185, 75),
    );

    // Ambient indigo wash — dead-centre, very large, very faint
    paint_blob(
        &p,
        rect.center(),
        w * 0.55,
        Color32::from_rgba_premultiplied(50, 24, 110, 35),
    );
}

/// Draws a radial gradient circle using a triangle-fan mesh:
/// fully-coloured centre fading to transparent edge.
fn paint_blob(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    use egui::epaint::{Mesh, Vertex};

    let mut mesh = Mesh::default();
    let segs = 48u32;

    // Centre vertex — full colour
    mesh.vertices.push(Vertex {
        pos: center,
        uv: egui::pos2(0.0, 0.0),
        color,
    });

    // Ring vertices — transparent edge
    let edge = Color32::from_rgba_premultiplied(0, 0, 0, 0);
    for i in 0..segs {
        let a = i as f32 * std::f32::consts::TAU / segs as f32;
        mesh.vertices.push(Vertex {
            pos: center + egui::vec2(a.cos() * radius, a.sin() * radius),
            uv: egui::pos2(0.0, 0.0),
            color: edge,
        });
    }

    // Triangle fan from centre
    for i in 0..segs {
        mesh.indices.push(0);
        mesh.indices.push(1 + i);
        mesh.indices.push(1 + (i + 1) % segs);
    }

    painter.add(Shape::Mesh(Arc::new(mesh)));
}

// ── Global visuals ────────────────────────────────────────────────────────────
pub fn apply(ctx: &egui::Context) {
    let mut vis = egui::Visuals::dark();

    vis.panel_fill = BG_BASE;
    vis.window_fill = GLASS_FILL;
    vis.extreme_bg_color = Color32::from_rgb(5, 4, 12);
    vis.faint_bg_color = GLASS_FILL_INSET;
    vis.window_corner_radius = CornerRadius::same(20);
    vis.window_stroke = Stroke::new(1.0, GLASS_BORDER);

    // Noninteractive (labels, separators)
    vis.widgets.noninteractive.bg_fill = GLASS_FILL;
    vis.widgets.noninteractive.bg_stroke = Stroke::new(1.0, GLASS_BORDER);
    vis.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    vis.widgets.noninteractive.corner_radius = CornerRadius::same(8);

    // Inactive buttons / text edits
    vis.widgets.inactive.bg_fill = GLASS_FILL_RAISED;
    vis.widgets.inactive.bg_stroke = Stroke::new(1.0, GLASS_BORDER);
    vis.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    vis.widgets.inactive.corner_radius = CornerRadius::same(10);

    // Hovered
    vis.widgets.hovered.bg_fill = Color32::from_rgba_premultiplied(55, 48, 100, 220);
    vis.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_BLUE);
    vis.widgets.hovered.fg_stroke = Stroke::new(1.0, ACCENT_BLUE);
    vis.widgets.hovered.corner_radius = CornerRadius::same(10);

    // Pressed
    vis.widgets.active.bg_fill = ACCENT_BLUE_DIM;
    vis.widgets.active.bg_stroke = Stroke::new(2.0, ACCENT_BLUE);
    vis.widgets.active.fg_stroke = Stroke::new(1.0, ACCENT_BLUE);
    vis.widgets.active.corner_radius = CornerRadius::same(10);

    // Open (combo-box / drop-down)
    vis.widgets.open.bg_fill = GLASS_FILL_RAISED;
    vis.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT_BLUE);
    vis.widgets.open.fg_stroke = Stroke::new(1.0, ACCENT_BLUE);
    vis.widgets.open.corner_radius = CornerRadius::same(10);

    vis.selection.bg_fill = ACCENT_BLUE_DIM;
    vis.selection.stroke = Stroke::new(1.0, ACCENT_BLUE);
    vis.hyperlink_color = ACCENT_BLUE;
    vis.override_text_color = Some(TEXT_PRIMARY);

    ctx.set_visuals(vis);

    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.window_margin = Margin::same(0);
    ctx.set_global_style(style);
}
