//-- wxfrieze/obj_view.rs -------------------------------------------------------------------------------------------------------------
//! Native viewport for rendering Wavefront OBJ meshes with Points / Wireframe / Facets /
//! ShadedWire modes, mouse orbit/pan, and zoom controls.
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use wxdragon::color::Colour;
use wxdragon::dc::auto_buffered_paint_dc::AutoBufferedPaintDC;
use wxdragon::dc::{BrushStyle, DeviceContext, PenStyle, Point as DcPoint, PolygonFillMode};
use wxdragon::event::window_events::WindowEventData;
use wxdragon::prelude::*;
use wxdragon::timer::Timer;
use wxdragon::window::BackgroundStyle;

use crate::frieze::state::ObjRenderMode;
use crate::wxfrieze::state::SharedState;

struct DragState {
    left_down: Cell<bool>,
    right_down: Cell<bool>,
    last_x: Cell<i32>,
    last_y: Cell<i32>,
}

/// Builds a native OBJ mesh viewport panel (HUD toolbar + drawing surface) for `path`.
pub fn build_obj_view_panel(parent: &Notebook, state: SharedState, path: PathBuf) -> Panel {
    let panel = Panel::builder(parent).build();
    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();

    let toolbar = Panel::builder(&panel).build();
    let toolbar_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let title_label = StaticText::builder(&toolbar)
        .with_label(&format!(
            "WAVEFRONT 3D — {}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("model.obj")
        ))
        .build();

    let points_btn = ToggleButton::builder(&toolbar).with_label("Points").build();
    let wire_btn = ToggleButton::builder(&toolbar).with_label("Wireframe").build();
    let facets_btn = ToggleButton::builder(&toolbar).with_label("Facets").build();
    let shaded_btn = ToggleButton::builder(&toolbar).with_label("Shaded+Wire").build();
    let reset_btn = Button::builder(&toolbar).with_label("Reset Camera").build();

    let sync_mode_buttons: Rc<dyn Fn()> = {
        let state = state.clone();
        Rc::new(move || {
            let mode = state.borrow()._ActiveObjMode;
            points_btn.set_value(mode == ObjRenderMode::Points);
            wire_btn.set_value(mode == ObjRenderMode::Wireframe);
            facets_btn.set_value(mode == ObjRenderMode::Facets);
            shaded_btn.set_value(mode == ObjRenderMode::ShadedWire);
        })
    };
    sync_mode_buttons();

    toolbar_sizer.add(&title_label, 1, SizerFlag::AlignCenterVertical | SizerFlag::All, 6);
    toolbar_sizer.add(&points_btn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 3);
    toolbar_sizer.add(&wire_btn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 3);
    toolbar_sizer.add(&facets_btn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 3);
    toolbar_sizer.add(&shaded_btn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 3);
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

    let timer = Timer::new(&canvas);
    {
        let state = state.clone();
        let drag = drag.clone();
        timer.on_tick(move |_| {
            if !drag.left_down.get() {
                state.borrow_mut()._ObjCamera.Rotate(0.001, 0.003);
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
                        state.borrow_mut()._ObjCamera.Pan(dx, dy);
                        canvas.refresh(false, None);
                    } else if drag.left_down.get() {
                        state.borrow_mut()._ObjCamera.Rotate(dy * 0.01, dx * 0.01);
                        canvas.refresh(false, None);
                    }
                    drag.last_x.set(pos.x);
                    drag.last_y.set(pos.y);
                }
            }
        });
    }

    let bind_mode = |btn: ToggleButton, mode: ObjRenderMode| {
        let state = state.clone();
        let sync = sync_mode_buttons.clone();
        btn.on_click(move |_| {
            state.borrow_mut()._ActiveObjMode = mode;
            sync();
            canvas.refresh(false, None);
        });
    };
    bind_mode(points_btn, ObjRenderMode::Points);
    bind_mode(wire_btn, ObjRenderMode::Wireframe);
    bind_mode(facets_btn, ObjRenderMode::Facets);
    bind_mode(shaded_btn, ObjRenderMode::ShadedWire);

    reset_btn.on_click({
        let state = state.clone();
        move |_| {
            state.borrow_mut()._ObjCamera.Reset();
            canvas.refresh(false, None);
        }
    });

    {
        let state = state.clone();
        let path = path.clone();
        canvas.on_paint(move |_evt| {
            let dc = AutoBufferedPaintDC::new(&canvas);
            draw_obj_frame(&dc, &canvas, &state, &path);
        });
    }

    panel
}

fn draw_obj_frame(dc: &AutoBufferedPaintDC, canvas: &Panel, state: &SharedState, path: &Path) {
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

    let (camera, mode) = {
        let st = state.borrow();
        (st._ObjCamera, st._ActiveObjMode)
    };

    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let cos_y = camera._RotY.cos();
    let sin_y = camera._RotY.sin();
    let cos_x = camera._RotX.cos();
    let sin_x = camera._RotX.sin();

    let project = |v: [f32; 3]| -> (f32, f32, f32) {
        let x1 = v[0] * cos_y + v[2] * sin_y;
        let z1 = -v[0] * sin_y + v[2] * cos_y;
        let y2 = v[1] * cos_x - z1 * sin_x;
        let z2 = v[1] * sin_x + z1 * cos_x;
        let scale = (350.0 * camera._Zoom) / (300.0 + z2).max(10.0);
        (center_x + camera._PanX + x1 * scale, center_y + camera._PanY - y2 * scale, z2)
    };

    let Some(mesh) = mesh else {
        dc.set_text_foreground(Colour::rgb(166, 173, 200));
        dc.draw_text("No mesh data available", 12, 10);
        return;
    };

    let cx = mesh._Center[0];
    let cy = mesh._Center[1];
    let cz = mesh._Center[2];
    let scale_norm = mesh._ScaleNorm;

    let local_of = |p: [f32; 3]| -> [f32; 3] {
        [(p[0] - cx) * scale_norm, (p[1] - cy) * scale_norm, (p[2] - cz) * scale_norm]
    };

    let projected_verts: Vec<(f32, f32, f32)> = mesh._Points.iter().map(|&p| project(local_of(p))).collect();

    if mode == ObjRenderMode::Facets || mode == ObjRenderMode::ShadedWire {
        // Painter's algorithm: sort triangles back-to-front by average depth.
        let mut tris: Vec<(usize, f32)> = mesh
            ._Triangles
            .iter()
            .enumerate()
            .map(|(i, tri)| {
                let z_avg = (projected_verts[tri[0] as usize].2
                    + projected_verts[tri[1] as usize].2
                    + projected_verts[tri[2] as usize].2)
                    / 3.0;
                (i, z_avg)
            })
            .collect();
        tris.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        dc.set_pen(Colour::rgb(203, 166, 247), 1, PenStyle::Solid);
        for (i, _z) in tris {
            let tri = mesh._Triangles[i];
            let a = projected_verts[tri[0] as usize];
            let b = projected_verts[tri[1] as usize];
            let c = projected_verts[tri[2] as usize];
            let shade = ((200.0 - _z).clamp(60.0, 220.0)) as u8;
            dc.set_brush(Colour::rgb(shade, (shade as f32 * 0.75) as u8, (shade as f32 * 0.95) as u8), BrushStyle::Solid);
            let poly = [
                DcPoint { x: a.0 as i32, y: a.1 as i32 },
                DcPoint { x: b.0 as i32, y: b.1 as i32 },
                DcPoint { x: c.0 as i32, y: c.1 as i32 },
            ];
            dc.draw_polygon(&poly, 0, 0, PolygonFillMode::OddEven);
        }
    }

    if mode == ObjRenderMode::Wireframe || mode == ObjRenderMode::ShadedWire {
        dc.set_pen(Colour::rgb(203, 166, 247), 1, PenStyle::Solid);
        for edge in mesh._Triangles.iter() {
            let a = projected_verts[edge[0] as usize];
            let b = projected_verts[edge[1] as usize];
            let c = projected_verts[edge[2] as usize];
            dc.draw_line(a.0 as i32, a.1 as i32, b.0 as i32, b.1 as i32);
            dc.draw_line(b.0 as i32, b.1 as i32, c.0 as i32, c.1 as i32);
            dc.draw_line(c.0 as i32, c.1 as i32, a.0 as i32, a.1 as i32);
        }
    }

    if mode == ObjRenderMode::Points {
        dc.set_pen(Colour::rgb(203, 166, 247), 1, PenStyle::Solid);
        dc.set_brush(Colour::rgb(203, 166, 247), BrushStyle::Solid);
        for &(x, y, _z) in &projected_verts {
            dc.draw_circle(x as i32, y as i32, 1);
        }
    }

    dc.set_text_foreground(Colour::rgb(203, 166, 247));
    dc.draw_text(&format!("{} verts | {} faces", mesh.PointCount(), mesh.FaceCount()), 12, 10);
}
