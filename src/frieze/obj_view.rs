//-- frieze/obj_view.rs --------------------------------------------------------------------------------------------------------------
use	std::path::Path;
use	egui::{ Ui, Color32, Pos2, Stroke, Sense, RichText, Vec2, Mesh, Shape };
use	crate::frieze::state::{ AppState, ObjRenderMode };

// ---------------------------------------------------------------------------------------------------------------------------------

/// Viewport for rendering 3D Wavefront OBJ models supporting Points, Wireframe, Facets, and ShadedWire.
pub fn	RenderObjView( ui: &mut Ui, path: &Path, state: &mut AppState)
{
    let  	fileName = path.file_name().and_then( |n| n.to_str()).unwrap_or( "model.obj");

    // Load or retrieve cached mesh
    let  	dev = state._Engine.as_ref().and_then( |e| e.WgpuDevice()).map( |d| d.as_ref());
    let  	meshOpt = state._MeshCache.GetOrLoad( dev, path).cloned();

    let  	vertCount = meshOpt.as_ref().map( |m| m.PointCount()).unwrap_or( 0);
    let  	faceCount = meshOpt.as_ref().map( |m| m.FaceCount()).unwrap_or( 0);
    let  	bboxLabel = meshOpt.as_ref().map( |m| {
        format!( "[{:.2}, {:.2}, {:.2}]  [{:.2}, {:.2}, {:.2}]",
            m._BboxMin[0], m._BboxMin[1], m._BboxMin[2],
            m._BboxMax[0], m._BboxMax[1], m._BboxMax[2])
    }).unwrap_or_else( || "None".to_string());

    // Top HUD Bar & Render Mode Controls
    ui.horizontal( |ui| {
        ui.spacing_mut().item_spacing = Vec2::new( 8.0, 0.0);
        ui.label( RichText::new( "WAVEFRONT 3D").color( Color32::from_rgb( 203, 166, 247)).strong().size( 12.5));
        ui.label( RichText::new( fileName).strong().size( 15.0).color( Color32::from_rgb( 205, 214, 244)));

        ui.separator();
        for (m, label) in [
            (ObjRenderMode::Points, "Points"),
            (ObjRenderMode::Wireframe, "Wireframe"),
            (ObjRenderMode::Facets, "Facets"),
            (ObjRenderMode::ShadedWire, "Shaded + Wire"),
        ] {
            let  	selected = state._ActiveObjMode == m;
            let  	btn = ui.selectable_label(
                selected,
                RichText::new( label)
                    .size( 13.0)
                    .color( if selected { Color32::WHITE } else { Color32::from_rgb( 166, 173, 200) })
            );
            if btn.clicked() {
                state._ActiveObjMode = m;
            }
        }

        ui.label( RichText::new( format!( "{} Verts | {} Faces | BBox: {}", vertCount, faceCount, bboxLabel)).size( 13.5).color( Color32::from_rgb( 203, 166, 247)));

        ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
            if ui.button( RichText::new( "Reset Camera").size( 13.0)).clicked() {
                state._ObjCamera.Reset();
            }
        });
    });

    ui.add_space( 2.0);
    ui.separator();

    let  	(response, painter) = ui.allocate_painter( ui.available_size(), Sense::drag());
    let  	rect = response.rect;

    // Handle mouse drag orbit / pan
    if response.dragged() {
        let  	delta = response.drag_delta();
        if ui.input( |i| i.modifiers.shift || i.pointer.button_down( egui::PointerButton::Secondary)) {
            state._ObjCamera.Pan( delta.x, delta.y);
        } else {
            state._ObjCamera.Rotate( delta.y * 0.01, delta.x * 0.01);
        }
    } else {
        state._ObjCamera.Rotate( 0.001, 0.003);
    }

    // Handle zoom
    let  	scroll = ui.input( |i| i.smooth_scroll_delta.y);
    if scroll != 0.0 {
        let  	factor = if scroll > 0.0 { 1.1 } else { 0.9 };
        state._ObjCamera.Zoom( factor);
    }

    // Clear background (#0b0f19)
    painter.rect_filled( rect, 0.0, Color32::from_rgb( 11, 15, 25));

    // Update GPU uniforms if renderer is active
    if let Some( ref renderer) = state._ViewportRenderer {
        renderer.UpdateUniforms( &state._ObjCamera, rect.width(), rect.height(), [0.8, 0.7, 1.0, 1.0]);
    }

    let  	center = rect.center();
    let  	cosY = state._ObjCamera._RotY.cos();
    let  	sinY = state._ObjCamera._RotY.sin();
    let  	cosX = state._ObjCamera._RotX.cos();
    let  	sinX = state._ObjCamera._RotX.sin();

    let  	project = |v: [f32; 3]| -> (Pos2, f32) {
        let  	x1 = v[0] * cosY + v[2] * sinY;
        let  	z1 = -v[0] * sinY + v[2] * cosY;
        let  	y2 = v[1] * cosX - z1 * sinX;
        let  	z2 = v[1] * sinX + z1 * cosX;
        let  	scale = ( 350.0 * state._ObjCamera._Zoom) / ( 300.0 + z2).max( 10.0);
        ( Pos2::new( center.x + state._ObjCamera._PanX + x1 * scale, center.y + state._ObjCamera._PanY - y2 * scale), z2)
    };

    if let Some( ref mesh) = meshOpt {
        let  	points = &mesh._Points;
        let  	triangles = &mesh._Triangles;
        let  	cx = mesh._Center[0];
        let  	cy = mesh._Center[1];
        let  	cz = mesh._Center[2];
        let  	scaleNorm = mesh._ScaleNorm;
        let  	mode = state._ActiveObjMode;

        // Render Triangles / Facets
        if mode == ObjRenderMode::Facets || mode == ObjRenderMode::ShadedWire {
            for tri in triangles.iter() {
                let  	idx0 = tri[0] as usize;
                let  	idx1 = tri[1] as usize;
                let  	idx2 = tri[2] as usize;

                if idx0 < points.len() && idx1 < points.len() && idx2 < points.len() {
                    let  	norm0 = [
                        (points[idx0][0] - cx) * scaleNorm,
                        (points[idx0][1] - cy) * scaleNorm,
                        (points[idx0][2] - cz) * scaleNorm,
                    ];
                    let  	norm1 = [
                        (points[idx1][0] - cx) * scaleNorm,
                        (points[idx1][1] - cy) * scaleNorm,
                        (points[idx1][2] - cz) * scaleNorm,
                    ];
                    let  	norm2 = [
                        (points[idx2][0] - cx) * scaleNorm,
                        (points[idx2][1] - cy) * scaleNorm,
                        (points[idx2][2] - cz) * scaleNorm,
                    ];

                    let  	(p0, z0) = project( norm0);
                    let  	(p1, z1) = project( norm1);
                    let  	(p2, z2) = project( norm2);

                    // Normal & backface culling
                    let  	ux = p1.x - p0.x;
                    let  	uy = p1.y - p0.y;
                    let  	vx = p2.x - p0.x;
                    let  	vy = p2.y - p0.y;
                    let  	crossZ = ux * vy - uy * vx;

                    if crossZ > 0.0 {
                        let  	depthShade = (((z0 + z1 + z2) / 3.0) * 0.5 + 130.0).clamp( 50.0, 245.0) as u8;
                        let  	color = Color32::from_rgb( depthShade / 2 + 70, depthShade / 3 + 45, depthShade);

                        let  	mut eguiMesh = Mesh::default();
                        eguiMesh.add_triangle( 0, 1, 2);
                        eguiMesh.vertices.push( egui::epaint::Vertex { pos: p0, uv: egui::epaint::WHITE_UV, color });
                        eguiMesh.vertices.push( egui::epaint::Vertex { pos: p1, uv: egui::epaint::WHITE_UV, color });
                        eguiMesh.vertices.push( egui::epaint::Vertex { pos: p2, uv: egui::epaint::WHITE_UV, color });

                        painter.add( Shape::mesh( eguiMesh));

                        if mode == ObjRenderMode::ShadedWire {
                            let  	wireStroke = Stroke::new( 0.5, Color32::from_rgba_premultiplied( 255, 255, 255, 25));
                            painter.line_segment( [p0, p1], wireStroke);
                            painter.line_segment( [p1, p2], wireStroke);
                            painter.line_segment( [p2, p0], wireStroke);
                        }
                    }
                }
            }
        }

        // Render Wireframe only
        if mode == ObjRenderMode::Wireframe {
            let  	wireStroke = Stroke::new( 1.0, Color32::from_rgb( 203, 166, 247));
            for tri in triangles.iter() {
                let  	idx0 = tri[0] as usize;
                let  	idx1 = tri[1] as usize;
                let  	idx2 = tri[2] as usize;
                if idx0 < points.len() && idx1 < points.len() && idx2 < points.len() {
                    let  	norm0 = [(points[idx0][0] - cx) * scaleNorm, (points[idx0][1] - cy) * scaleNorm, (points[idx0][2] - cz) * scaleNorm];
                    let  	norm1 = [(points[idx1][0] - cx) * scaleNorm, (points[idx1][1] - cy) * scaleNorm, (points[idx1][2] - cz) * scaleNorm];
                    let  	norm2 = [(points[idx2][0] - cx) * scaleNorm, (points[idx2][1] - cy) * scaleNorm, (points[idx2][2] - cz) * scaleNorm];
                    let  	(p0, _) = project( norm0);
                    let  	(p1, _) = project( norm1);
                    let  	(p2, _) = project( norm2);
                    painter.line_segment( [p0, p1], wireStroke);
                    painter.line_segment( [p1, p2], wireStroke);
                    painter.line_segment( [p2, p0], wireStroke);
                }
            }
        }

        // Render Points mode
        if mode == ObjRenderMode::Points {
            let  	ptColor = Color32::from_rgb( 203, 166, 247);
            for pt in points.iter() {
                let  	norm = [(pt[0] - cx) * scaleNorm, (pt[1] - cy) * scaleNorm, (pt[2] - cz) * scaleNorm];
                let  	(p, _) = project( norm);
                if rect.contains( p) {
                    painter.circle_filled( p, 2.0, ptColor);
                }
            }
        }
    }

    // HUD Overlay
    let  	hudText = format!( "{} Vertices | {} Faces | Mode: {:?} | In-Process WebGPU Cached | Zoom: {:.2}x", vertCount, faceCount, state._ActiveObjMode, state._ObjCamera._Zoom);
    painter.text(
        rect.left_bottom() + Vec2::new( 16.0, -16.0),
        egui::Align2::LEFT_BOTTOM,
        hudText,
        egui::FontId::monospace( 11.5),
        Color32::from_rgb( 205, 214, 244),
    );

    ui.ctx().request_repaint();
}

// ---------------------------------------------------------------------------------------------------------------------------------
