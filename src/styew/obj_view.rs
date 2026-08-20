//-- styew/obj_view.rs --------------------------------------------------------------------------------------------------------------
use	std::path::Path;
use	egui::{ Ui, Color32, Pos2, Stroke, Sense, RichText, Vec2, Shape };
use	egui::epaint::Mesh;
use	crate::styew::state::{ CameraState, ObjRenderMode };
use	crate::fenst::XplrParseWaveObjFile;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Viewport for rendering 3D Wavefront OBJ meshes in native immediate mode.
pub fn	RenderObjView( ui: &mut Ui, path: &Path, camera: &mut CameraState, mode: &mut ObjRenderMode)
{
    // HUD Header with Mode Buttons
    ui.horizontal( |ui| {
        let  	fileName = path.file_name().and_then( |n| n.to_str()).unwrap_or( "mesh.obj");
        ui.label( RichText::new( fileName).strong().size( 14.0).color( Color32::WHITE));
        ui.label( RichText::new( "◬ SWARM-SYMPH MESH").color( Color32::from_rgb( 168, 85, 247)).strong().size( 11.0));

        ui.add_space( 16.0);

        // Mode switchers
        for (m, label) in [
            (ObjRenderMode::Points, "Points"),
            (ObjRenderMode::Wireframe, "Wireframe"),
            (ObjRenderMode::Facets, "Facets"),
            (ObjRenderMode::ShadedWire, "Shaded + Wire"),
        ] {
            let  	selected = *mode == m;
            let  	btn = ui.selectable_label( selected, RichText::new( label).size( 11.0));
            if btn.clicked() {
                *mode = m;
            }
        }

        ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
            if ui.button( "↺ Reset Camera").clicked() {
                *camera = CameraState::default();
            }
        });
    });

    ui.separator();

    // Canvas Allocation
    let  	(response, painter) = ui.allocate_painter( ui.available_size(), Sense::drag());
    let  	rect = response.rect;

    // Handle mouse drag orbit / pan
    if response.dragged() {
        let  	delta = response.drag_delta();
        if ui.input( |i| i.modifiers.shift || i.pointer.button_down( egui::PointerButton::Secondary)) {
            camera._PanX += delta.x;
            camera._PanY += delta.y;
        } else {
            camera._RotY += delta.x * 0.01;
            camera._RotX += delta.y * 0.01;
        }
    } else {
        camera._RotY += 0.003;
        camera._RotX += 0.001;
    }

    // Handle zoom
    let  	scroll = ui.input( |i| i.smooth_scroll_delta.y);
    if scroll != 0.0 {
        if scroll > 0.0 {
            camera._Zoom *= 1.1;
        } else {
            camera._Zoom *= 0.9;
        }
        camera._Zoom = camera._Zoom.clamp( 0.05, 50.0);
    }

    // Clear background
    painter.rect_filled( rect, 0.0, Color32::from_rgb( 11, 15, 25));

    // Load mesh data in-process
    let  	meshDto = if path.exists() {
        let  	pathStr = path.to_string_lossy().to_string();
        XplrParseWaveObjFile( &pathStr).ok()
    } else {
        None
    };

    let  	center = rect.center();
    let  	cosY = camera._RotY.cos();
    let  	sinY = camera._RotY.sin();
    let  	cosX = camera._RotX.cos();
    let  	sinX = camera._RotX.sin();

    let  	project = |v: [f32; 3]| -> (Pos2, f32) {
        let  	x1 = v[0] * cosY + v[2] * sinY;
        let  	z1 = -v[0] * sinY + v[2] * cosY;
        let  	y2 = v[1] * cosX - z1 * sinX;
        let  	z2 = v[1] * sinX + z1 * cosX;
        let  	scale = (350.0 * camera._Zoom) / (300.0 + z2).max( 10.0);
        (Pos2::new( center.x + camera._PanX + x1 * scale, center.y + camera._PanY - y2 * scale), z2)
    };

    let  	mut vertCount = 0;
    let  	mut faceCount = 0;

    if let  	Some( mesh) = meshDto {
        vertCount = mesh._Points.len();
        faceCount = mesh._Triangles.len();

        let  	points = &mesh._Points;
        let  	triangles = &mesh._Triangles;

        // Render Triangles / Facets
        if *mode == ObjRenderMode::Facets || *mode == ObjRenderMode::ShadedWire {
            for tri in triangles.iter() {
                let  	idx0 = tri[0] as usize;
                let  	idx1 = tri[1] as usize;
                let  	idx2 = tri[2] as usize;

                if idx0 < points.len() && idx1 < points.len() && idx2 < points.len() {
                    let  	(p0, z0) = project( points[idx0]);
                    let  	(p1, z1) = project( points[idx1]);
                    let  	(p2, z2) = project( points[idx2]);

                    let  	ux = p1.x - p0.x;
                    let  	uy = p1.y - p0.y;
                    let  	vx = p2.x - p0.x;
                    let  	vy = p2.y - p0.y;
                    let  	crossZ = ux * vy - uy * vx;

                    if crossZ > 0.0 {
                        let  	depthShade = (((z0 + z1 + z2) / 3.0) * 0.5 + 128.0).clamp( 40.0, 240.0) as u8;
                        let  	color = Color32::from_rgb( depthShade / 2 + 60, depthShade / 3 + 40, depthShade);

                        let  	mut eguiMesh = Mesh::default();
                        eguiMesh.add_triangle( 0, 1, 2);
                        eguiMesh.vertices.push( egui::epaint::Vertex { pos: p0, uv: egui::epaint::WHITE_UV, color });
                        eguiMesh.vertices.push( egui::epaint::Vertex { pos: p1, uv: egui::epaint::WHITE_UV, color });
                        eguiMesh.vertices.push( egui::epaint::Vertex { pos: p2, uv: egui::epaint::WHITE_UV, color });

                        painter.add( Shape::mesh( eguiMesh));

                        if *mode == ObjRenderMode::ShadedWire {
                            let  	wireStroke = Stroke::new( 0.5, Color32::from_rgba_premultiplied( 255, 255, 255, 30));
                            painter.line_segment( [p0, p1], wireStroke);
                            painter.line_segment( [p1, p2], wireStroke);
                            painter.line_segment( [p2, p0], wireStroke);
                        }
                    }
                }
            }
        }

        // Render Wireframe only
        if *mode == ObjRenderMode::Wireframe {
            let  	wireStroke = Stroke::new( 1.0, Color32::from_rgb( 168, 85, 247));
            for tri in triangles.iter() {
                let  	idx0 = tri[0] as usize;
                let  	idx1 = tri[1] as usize;
                let  	idx2 = tri[2] as usize;
                if idx0 < points.len() && idx1 < points.len() && idx2 < points.len() {
                    let  	(p0, _) = project( points[idx0]);
                    let  	(p1, _) = project( points[idx1]);
                    let  	(p2, _) = project( points[idx2]);
                    painter.line_segment( [p0, p1], wireStroke);
                    painter.line_segment( [p1, p2], wireStroke);
                    painter.line_segment( [p2, p0], wireStroke);
                }
            }
        }

        // Render Points mode
        if *mode == ObjRenderMode::Points {
            let  	ptColor = Color32::from_rgb( 192, 132, 252);
            for pt in points.iter() {
                let  	(p, _) = project( *pt);
                if rect.contains( p) {
                    painter.circle_filled( p, 2.0, ptColor);
                }
            }
        }
    }

    // HUD Overlay
    let  	hudText = format!( "{} Vertices | {} Faces | Zoom: {:.2}x | Rot: ({:.2}, {:.2})", vertCount, faceCount, camera._Zoom, camera._RotX, camera._RotY);
    painter.text(
        rect.left_bottom() + Vec2::new( 16.0, -16.0),
        egui::Align2::LEFT_BOTTOM,
        hudText,
        egui::FontId::monospace( 12.0),
        Color32::from_rgb( 168, 85, 247),
    );

    ui.ctx().request_repaint();
}

// ---------------------------------------------------------------------------------------------------------------------------------
