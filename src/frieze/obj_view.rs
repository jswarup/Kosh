//-- frieze/obj_view.rs --------------------------------------------------------------------------------------------------------------
use	std::path::Path;
use	egui::{ Ui, Color32, Pos2, Stroke, Sense, RichText, Vec2, Shape };
use	egui::epaint::Mesh;
use	crate::frieze::state::{ CameraState, ObjRenderMode };
use	crate::fenst::XplrParseWaveObjFile;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Viewport for rendering 3D Wavefront OBJ meshes in native immediate mode with auto-fit centering.
pub fn	RenderObjView( ui: &mut Ui, path: &Path, camera: &mut CameraState, mode: &mut ObjRenderMode)
{
    let  	fileName = path.file_name().and_then( |n| n.to_str()).unwrap_or( "mesh.obj");

    // Load mesh data in-process
    let  	meshDto = if path.exists() {
        let  	pathStr = path.to_string_lossy().to_string();
        XplrParseWaveObjFile( &pathStr).ok()
    } else {
        None
    };

    let  	mut vertCount = 0;
    let  	mut faceCount = 0;
    let  	mut minX = f32::MAX; let  	mut maxX = f32::MIN;
    let  	mut minY = f32::MAX; let  	mut maxY = f32::MIN;
    let  	mut minZ = f32::MAX; let  	mut maxZ = f32::MIN;

    if let  	Some( ref mesh) = meshDto {
        vertCount = mesh._Points.len();
        faceCount = mesh._Triangles.len();

        for pt in mesh._Points.iter() {
            minX = minX.min( pt[0]); maxX = maxX.max( pt[0]);
            minY = minY.min( pt[1]); maxY = maxY.max( pt[1]);
            minZ = minZ.min( pt[2]); maxZ = maxZ.max( pt[2]);
        }
    }

    if minX == f32::MAX {
        minX = -50.0; maxX = 50.0;
        minY = -50.0; maxY = 50.0;
        minZ = -50.0; maxZ = 50.0;
    }

    let  	cx = (minX + maxX) * 0.5;
    let  	cy = (minY + maxY) * 0.5;
    let  	cz = (minZ + maxZ) * 0.5;
    let  	dx = maxX - minX;
    let  	dy = maxY - minY;
    let  	dz = maxZ - minZ;
    let  	maxDim = dx.max( dy).max( dz);
    let  	scaleNorm = if maxDim > 1e-4 { 240.0 / maxDim } else { 1.0 };

    let  	bboxLabel = format!( "[{:.2}, {:.2}, {:.2}]  [{:.2}, {:.2}, {:.2}]", minX, minY, minZ, maxX, maxY, maxZ);

    // Top Header with Mode Buttons
    ui.horizontal( |ui| {
        ui.spacing_mut().item_spacing = Vec2::new( 8.0, 0.0);

        ui.label( RichText::new( "WAVEFRONT OBJ").color( Color32::from_rgb( 203, 166, 247)).strong().size( 10.5));
        ui.label( RichText::new( fileName).strong().size( 13.0).color( Color32::from_rgb( 205, 214, 244)));

        // Mode switchers
        for (m, label) in [
            (ObjRenderMode::Points, "Points"),
            (ObjRenderMode::Wireframe, "Wireframe"),
            (ObjRenderMode::Facets, "Facets"),
            (ObjRenderMode::ShadedWire, "Shaded + Wire"),
        ] {
            let  	selected = *mode == m;
            let  	btn = ui.selectable_label(
                selected,
                RichText::new( label)
                    .size( 11.0)
                    .color( if selected { Color32::WHITE } else { Color32::from_rgb( 166, 173, 200) })
            );
            if btn.clicked() {
                *mode = m;
            }
        }

        ui.label( RichText::new( format!( "{} Verts | {} Faces | BBox: {}", vertCount, faceCount, bboxLabel)).size( 11.5).color( Color32::from_rgb( 203, 166, 247)));

        ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
            if ui.button( RichText::new( "↺ Reset Camera").size( 11.0)).clicked() {
                *camera = CameraState::default();
            }
        });
    });

    ui.add_space( 2.0);
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

    // Clear background (#0b0f19)
    painter.rect_filled( rect, 0.0, Color32::from_rgb( 11, 15, 25));

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

    if let  	Some( mesh) = meshDto {
        let  	points = &mesh._Points;
        let  	triangles = &mesh._Triangles;

        // Render Triangles / Facets
        if *mode == ObjRenderMode::Facets || *mode == ObjRenderMode::ShadedWire {
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

                        if *mode == ObjRenderMode::ShadedWire {
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
        if *mode == ObjRenderMode::Wireframe {
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
        if *mode == ObjRenderMode::Points {
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
    let  	hudText = format!( "{} Vertices | {} Faces | Zoom: {:.2}x | Rot: ({:.2}, {:.2})", vertCount, faceCount, camera._Zoom, camera._RotX, camera._RotY);
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
