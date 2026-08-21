//-- frieze/pts_view.rs --------------------------------------------------------------------------------------------------------------
use	std::path::Path;
use	egui::{ Ui, Color32, Pos2, Stroke, Sense, RichText, Vec2 };
use	crate::frieze::state::CameraState;
use	crate::fenst::XplrParsePtsFile;
use	crate::silo::{ Buff, U32 };

// ---------------------------------------------------------------------------------------------------------------------------------

/// Viewport for rendering 3D point clouds in native immediate mode with auto-fit centering.
pub fn	RenderPtsView( ui: &mut Ui, path: &Path, camera: &mut CameraState)
{
    let  	fileName = path.file_name().and_then( |n| n.to_str()).unwrap_or( "point_cloud.pts");

    // Load points in-process
    let  	points: Buff< [f32; 3]> = if path.exists() {
        let  	pathStr = path.to_string_lossy().to_string();
        if let  	Ok( dto) = XplrParsePtsFile( &pathStr) {
            dto._Points
        } else {
            Buff::New()
        }
    } else {
        Buff::Create( U32( 1000), |i| {
            let  	t = i.AsU32() as f32 * 0.1;
            [t.sin() * 50.0, (t * 0.5).cos() * 50.0, (t * 0.2).sin() * 50.0]
        })
    };

    let  	pointCount = points.len();

    // Compute 3D bounding box and auto-fit normalization
    let  	mut minX = f32::MAX; let  	mut maxX = f32::MIN;
    let  	mut minY = f32::MAX; let  	mut maxY = f32::MIN;
    let  	mut minZ = f32::MAX; let  	mut maxZ = f32::MIN;

    for pt in points.iter() {
        minX = minX.min( pt[0]); maxX = maxX.max( pt[0]);
        minY = minY.min( pt[1]); maxY = maxY.max( pt[1]);
        minZ = minZ.min( pt[2]); maxZ = maxZ.max( pt[2]);
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

    // Top HUD Bar
    ui.horizontal( |ui| {
        ui.spacing_mut().item_spacing = Vec2::new( 10.0, 0.0);
        ui.label( RichText::new( "POINT CLOUD").color( Color32::from_rgb( 137, 180, 250)).strong().size( 10.5));
        ui.label( RichText::new( fileName).strong().size( 13.0).color( Color32::from_rgb( 205, 214, 244)));
        ui.label( RichText::new( format!( "{} Points | BBox: {}", pointCount, bboxLabel)).size( 11.5).color( Color32::from_rgb( 137, 220, 235)));

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
        camera._RotY += 0.005;
        camera._RotX += 0.002;
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
    for pt in points.iter() {
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

    // 3. HUD Overlay
    let  	hudText = format!( "{} Points | Zoom: {:.2}x | Pan: ({:.0}, {:.0})", pointCount, camera._Zoom, camera._PanX, camera._PanY);
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
