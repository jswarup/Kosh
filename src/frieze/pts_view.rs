//-- frieze/pts_view.rs --------------------------------------------------------------------------------------------------------------
use	std::path::Path;
use	egui::{ Ui, Color32, Pos2, Stroke, Sense, RichText, Vec2 };
use	crate::frieze::state::AppState;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Viewport for rendering 3D point clouds in high performance native mode with GPU VRAM caching.
pub fn	RenderPtsView( ui: &mut Ui, path: &Path, state: &mut AppState)
{
    let  	fileName = path.file_name().and_then( |n| n.to_str()).unwrap_or( "point_cloud.pts");

    // Load or retrieve cached mesh
    let  	dev = state._Engine.as_ref().and_then( |e| e.WgpuDevice()).map( |d| d.as_ref());
    let  	meshOpt = state._MeshCache.GetOrLoad( dev, path).cloned();

    let  	pointCount = meshOpt.as_ref().map( |m| m.PointCount()).unwrap_or( 0);
    let  	bboxLabel = meshOpt.as_ref().map( |m| {
        format!( "[{:.2}, {:.2}, {:.2}]  [{:.2}, {:.2}, {:.2}]",
            m._BboxMin[0], m._BboxMin[1], m._BboxMin[2],
            m._BboxMax[0], m._BboxMax[1], m._BboxMax[2])
    }).unwrap_or_else( || "None".to_string());

    // Top HUD Bar
    ui.horizontal( |ui| {
        ui.spacing_mut().item_spacing = Vec2::new( 10.0, 0.0);
        ui.label( RichText::new( "POINT CLOUD").color( Color32::from_rgb( 137, 180, 250)).strong().size( 10.5));
        ui.label( RichText::new( fileName).strong().size( 13.0).color( Color32::from_rgb( 205, 214, 244)));
        ui.label( RichText::new( format!( "{} Points | BBox: {}", pointCount, bboxLabel)).size( 11.5).color( Color32::from_rgb( 137, 220, 235)));

        ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
            if ui.button( RichText::new( "Reset Camera").size( 11.0)).clicked() {
                state._PtsCamera.Reset();
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
            state._PtsCamera.Pan( delta.x, delta.y);
        } else {
            state._PtsCamera.Rotate( delta.y * 0.01, delta.x * 0.01);
        }
    } else {
        state._PtsCamera.Rotate( 0.002, 0.005);
    }

    // Handle zoom
    let  	scroll = ui.input( |i| i.smooth_scroll_delta.y);
    if scroll != 0.0 {
        let  	factor = if scroll > 0.0 { 1.1 } else { 0.9 };
        state._PtsCamera.Zoom( factor);
    }

    // Background fill (#0b0f19)
    painter.rect_filled( rect, 0.0, Color32::from_rgb( 11, 15, 25));

    // Update GPU uniforms if renderer is active
    if let Some( ref renderer) = state._ViewportRenderer {
        renderer.UpdateUniforms( &state._PtsCamera, rect.width(), rect.height(), [0.0, 0.95, 1.0, 1.0]);
    }

    let  	center = rect.center();
    let  	cosY = state._PtsCamera._RotY.cos();
    let  	sinY = state._PtsCamera._RotY.sin();
    let  	cosX = state._PtsCamera._RotX.cos();
    let  	sinX = state._PtsCamera._RotX.sin();

    let  	project = |v: [f32; 3]| -> (Pos2, f32) {
        let  	x1 = v[0] * cosY + v[2] * sinY;
        let  	z1 = -v[0] * sinY + v[2] * cosY;
        let  	y2 = v[1] * cosX - z1 * sinX;
        let  	z2 = v[1] * sinX + z1 * cosX;
        let  	scale = ( 350.0 * state._PtsCamera._Zoom) / ( 300.0 + z2).max( 10.0);
        ( Pos2::new( center.x + state._PtsCamera._PanX + x1 * scale, center.y + state._PtsCamera._PanY - y2 * scale), z2)
    };

    if let Some( ref mesh) = meshOpt {
        let  	cx = mesh._Center[0];
        let  	cy = mesh._Center[1];
        let  	cz = mesh._Center[2];
        let  	scaleNorm = mesh._ScaleNorm;
        let  	minX = mesh._BboxMin[0]; let  	maxX = mesh._BboxMax[0];
        let  	minY = mesh._BboxMin[1]; let  	maxY = mesh._BboxMax[1];
        let  	minZ = mesh._BboxMin[2]; let  	maxZ = mesh._BboxMax[2];

        // 1. Draw 3D Bounding Box Wireframe
        let  	bBoxVerts = [
            [(minX - cx) * scaleNorm, (minY - cy) * scaleNorm, (minZ - cz) * scaleNorm],
            [(maxX - cx) * scaleNorm, (minY - cy) * scaleNorm, (minZ - cz) * scaleNorm],
            [(maxX - cx) * scaleNorm, (maxY - cy) * scaleNorm, (minZ - cz) * scaleNorm],
            [(minX - cx) * scaleNorm, (maxY - cy) * scaleNorm, (minZ - cz) * scaleNorm],
            [(minX - cx) * scaleNorm, (minY - cy) * scaleNorm, (maxZ - cz) * scaleNorm],
            [(maxX - cx) * scaleNorm, (minY - cy) * scaleNorm, (maxZ - cz) * scaleNorm],
            [(maxX - cx) * scaleNorm, (maxY - cy) * scaleNorm, (maxZ - cz) * scaleNorm],
            [(minX - cx) * scaleNorm, (maxY - cy) * scaleNorm, (maxZ - cz) * scaleNorm],
        ];

        let  	projBox: Vec< Pos2> = bBoxVerts.iter().map( |&v| project( v).0).collect();
        let  	boxEdges = [
            (0,1),(1,2),(2,3),(3,0),
            (4,5),(5,6),(6,7),(7,4),
            (0,4),(1,5),(2,6),(3,7)
        ];

        let  	boxStroke = Stroke::new( 1.0, Color32::from_rgba_premultiplied( 0, 243, 255, 38));
        for (i, j) in boxEdges {
            painter.line_segment( [projBox[i], projBox[j]], boxStroke);
        }

        // 2. Draw Points with depth-shaded neon glow
        for pt in mesh._Points.iter() {
            let  	normPt = [
                (pt[0] - cx) * scaleNorm,
                (pt[1] - cy) * scaleNorm,
                (pt[2] - cz) * scaleNorm,
            ];
            let  	(p, z2) = project( normPt);
            if rect.contains( p) {
                let  	depthFactor = ((300.0 - z2) / 400.0).clamp( 0.3, 1.0);
                let  	alpha = (depthFactor * 255.0) as u8;
                let  	radius = 2.0 + depthFactor * 2.0;
                let  	color = Color32::from_rgba_premultiplied( 0, 243, 255, alpha);
                painter.circle_filled( p, radius, color);
            }
        }
    }

    // HUD Overlay
    let  	hudText = format!( "{} Points | In-Process WebGPU Cached | Zoom: {:.2}x | Pan: ({:.0}, {:.0})", pointCount, state._PtsCamera._Zoom, state._PtsCamera._PanX, state._PtsCamera._PanY);
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
