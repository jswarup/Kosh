//-- wxfrieze/pts_view.rs -------------------------------------------------------------------------------------------------------------
//! Native viewport for rendering 3D point clouds (`.pts`), with mouse orbit/pan and camera
//! reset/zoom controls. Projection math mirrors the CPU projection used by the egui viewport;
//! geometry loading/caching is shared via `crate::frieze::gpu_cache`.
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use wxdragon::color::Colour;
use wxdragon::dc::auto_buffered_paint_dc::AutoBufferedPaintDC;
use wxdragon::dc::{BrushStyle, DeviceContext, PenStyle};
use wxdragon::event::window_events::WindowEventData;
use wxdragon::prelude::*;
use wxdragon::timer::Timer;
use wxdragon::window::BackgroundStyle;

use crate::wxfrieze::state::SharedState;

struct DragState {
    left_down: Cell<bool>,
    right_down: Cell<bool>,
    last_x: Cell<i32>,
    last_y: Cell<i32>,
}

/// Builds a native point-cloud viewport panel (HUD toolbar + drawing surface) for `path`.
pub fn build_pts_view_panel(parent: &Notebook, state: SharedState, path: PathBuf) -> Panel {
    let panel = Panel::builder(parent).build();
    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();

    // HUD toolbar
    let toolbar = Panel::builder(&panel).build();
    let toolbar_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let title_label = StaticText::builder(&toolbar)
        .with_label(&format!(
            "POINT CLOUD — {}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("point_cloud.pts")
        ))
        .build();
    let zoom_in_btn = Button::builder(&toolbar).with_label("Zoom In").build();
    let zoom_out_btn = Button::builder(&toolbar).with_label("Zoom Out").build();
    let reset_btn = Button::builder(&toolbar).with_label("Reset Camera").build();
    toolbar_sizer.add(&title_label, 1, SizerFlag::AlignCenterVertical | SizerFlag::All, 6);
    toolbar_sizer.add(&zoom_in_btn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    toolbar_sizer.add(&zoom_out_btn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    toolbar_sizer.add(&reset_btn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    toolbar.set_sizer(toolbar_sizer, true);

    let canvas = Panel::builder(&panel).build();
    canvas.set_background_style(BackgroundStyle::Paint);
    canvas.on_erase_background(|_| {});

    root_sizer.add(&toolbar, 0, SizerFlag::Expand, 0);
    root_sizer.add(&canvas, 1, SizerFlag::Expand, 0);
    panel.set_sizer(root_sizer, true);

    let drag = Rc::new(DragState {
        left_down: Cell::new(false),
        right_down: Cell::new(false),
        last_x: Cell::new(0),
        last_y: Cell::new(0),
    });

    // Idle auto-rotate + redraw timer (mirrors the egui viewport's gentle idle spin).
    let timer = Timer::new(&canvas);
    {
        let state = state.clone();
        let drag = drag.clone();
        timer.on_tick(move |_| {
            if !drag.left_down.get() {
                state.borrow_mut()._PtsCamera.Rotate(0.002, 0.005);
            }
            canvas.refresh(false, None);
        });
    }
    timer.start(33, false);

    {
        let drag = drag.clone();
        canvas.on_mouse_left_down(move |evt| {
            if let WindowEventData::MouseButton(mb) = evt {
                if let Some(pos) = mb.get_position() {
                    drag.left_down.set(true);
                    drag.last_x.set(pos.x);
                    drag.last_y.set(pos.y);
                }
            }
        });
    }
    {
        let drag = drag.clone();
        canvas.on_mouse_left_up(move |_evt| drag.left_down.set(false));
    }
    {
        let drag = drag.clone();
        canvas.on_mouse_right_down(move |evt| {
            if let WindowEventData::MouseButton(mb) = evt {
                if let Some(pos) = mb.get_position() {
                    drag.right_down.set(true);
                    drag.last_x.set(pos.x);
                    drag.last_y.set(pos.y);
                }
            }
        });
    }
    {
        let drag = drag.clone();
        canvas.on_mouse_right_up(move |_evt| drag.right_down.set(false));
    }
    {
        let state = state.clone();
        let drag = drag.clone();
        canvas.on_mouse_motion(move |evt| {
            if let WindowEventData::MouseMotion(mm) = evt {
                if let Some(pos) = mm.get_position() {
                    let dx = (pos.x - drag.last_x.get()) as f32;
                    let dy = (pos.y - drag.last_y.get()) as f32;
                    if drag.right_down.get() {
                        state.borrow_mut()._PtsCamera.Pan(dx, dy);
                        canvas.refresh(false, None);
                    } else if drag.left_down.get() {
                        state.borrow_mut()._PtsCamera.Rotate(dy * 0.01, dx * 0.01);
                        canvas.refresh(false, None);
                    }
                    drag.last_x.set(pos.x);
                    drag.last_y.set(pos.y);
                }
            }
        });
    }

    zoom_in_btn.on_click({
        let state = state.clone();
        move |_| {
            state.borrow_mut()._PtsCamera.Zoom(1.1);
            canvas.refresh(false, None);
        }
    });
    zoom_out_btn.on_click({
        let state = state.clone();
        move |_| {
            state.borrow_mut()._PtsCamera.Zoom(0.9);
            canvas.refresh(false, None);
        }
    });
    reset_btn.on_click({
        let state = state.clone();
        move |_| {
            state.borrow_mut()._PtsCamera.Reset();
            canvas.refresh(false, None);
        }
    });

    {
        let state = state.clone();
        let path = path.clone();
        canvas.on_paint(move |_evt| {
            let dc = AutoBufferedPaintDC::new(&canvas);
            draw_pts_frame(&dc, &canvas, &state, &path);
        });
    }

    panel
}

fn draw_pts_frame(dc: &AutoBufferedPaintDC, canvas: &Panel, state: &SharedState, path: &Path) {
    let bg = state.borrow()._Theme.viewport_rgb();
    dc.set_background(Colour::rgb(bg.0, bg.1, bg.2));
    dc.clear();

    let size = canvas.get_client_size();
    let width = size.width as f32;
    let height = size.height as f32;

    let mesh = {
        let mut st = state.borrow_mut();
        st._MeshCache.GetOrLoad(None, path).cloned()
    };

    let camera = state.borrow()._PtsCamera;
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let cos_y = camera._RotY.cos();
    let sin_y = camera._RotY.sin();
    let cos_x = camera._RotX.cos();
    let sin_x = camera._RotX.sin();

    let project = |v: [f32; 3]| -> (f32, f32) {
        let x1 = v[0] * cos_y + v[2] * sin_y;
        let z1 = -v[0] * sin_y + v[2] * cos_y;
        let y2 = v[1] * cos_x - z1 * sin_x;
        let z2 = v[1] * sin_x + z1 * cos_x;
        let scale = (350.0 * camera._Zoom) / (300.0 + z2).max(10.0);
        (center_x + camera._PanX + x1 * scale, center_y + camera._PanY - y2 * scale)
    };

    let Some(mesh) = mesh else {
        dc.set_text_foreground(Colour::rgb(166, 173, 200));
        dc.draw_text("No point cloud data available", 12, 10);
        return;
    };

    let cx = mesh._Center[0];
    let cy = mesh._Center[1];
    let cz = mesh._Center[2];
    let scale_norm = mesh._ScaleNorm;
    let [min_x, min_y, min_z] = mesh._BboxMin;
    let [max_x, max_y, max_z] = mesh._BboxMax;

    let bbox_verts = [
        [(min_x - cx) * scale_norm, (min_y - cy) * scale_norm, (min_z - cz) * scale_norm],
        [(max_x - cx) * scale_norm, (min_y - cy) * scale_norm, (min_z - cz) * scale_norm],
        [(max_x - cx) * scale_norm, (max_y - cy) * scale_norm, (min_z - cz) * scale_norm],
        [(min_x - cx) * scale_norm, (max_y - cy) * scale_norm, (min_z - cz) * scale_norm],
        [(min_x - cx) * scale_norm, (min_y - cy) * scale_norm, (max_z - cz) * scale_norm],
        [(max_x - cx) * scale_norm, (min_y - cy) * scale_norm, (max_z - cz) * scale_norm],
        [(max_x - cx) * scale_norm, (max_y - cy) * scale_norm, (max_z - cz) * scale_norm],
        [(min_x - cx) * scale_norm, (max_y - cy) * scale_norm, (max_z - cz) * scale_norm],
    ];
    let proj_box: Vec<(f32, f32)> = bbox_verts.iter().map(|&v| project(v)).collect();
    let box_edges = [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4), (0, 4), (1, 5), (2, 6), (3, 7)];

    dc.set_pen(Colour::new(0, 243, 255, 90), 1, PenStyle::Solid);
    for &(a, b) in &box_edges {
        let (x1, y1) = proj_box[a];
        let (x2, y2) = proj_box[b];
        dc.draw_line(x1 as i32, y1 as i32, x2 as i32, y2 as i32);
    }

    dc.set_pen(Colour::rgb(137, 220, 235), 1, PenStyle::Solid);
    dc.set_brush(Colour::rgb(137, 220, 235), BrushStyle::Solid);
    for pt in mesh._Points.iter() {
        let local = [(pt[0] - cx) * scale_norm, (pt[1] - cy) * scale_norm, (pt[2] - cz) * scale_norm];
        let (x, y) = project(local);
        dc.draw_circle(x as i32, y as i32, 1);
    }

    dc.set_text_foreground(Colour::rgb(137, 220, 235));
    dc.draw_text(&format!("{} points", mesh.PointCount()), 12, 10);
}
