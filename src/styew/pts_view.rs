//-- styew/pts_view.rs --------------------------------------------------------------------------------------------------------------
use	std::path::Path;
use	egui::{ Ui, Color32, Pos2, Stroke, Sense, RichText, Vec2 };
use	crate::styew::state::CameraState;
use	crate::fenst::XplrParsePtsFile;
use	crate::silo::Buff;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Viewport for rendering 3D point clouds in native immediate mode.
pub fn	RenderPtsView( ui: &mut Ui, path: &Path, camera: &mut CameraState)
{
    // HUD Header
    ui.horizontal( |ui| {
        let  	fileName = path.file_name().and_then( |n| n.to_str()).unwrap_or( "point_cloud.pts");
        ui.label( RichText::new( fileName).strong().size( 14.0).color( Color32::WHITE));
        ui.label( RichText::new( "• SWARM-SYMPH NATIVE").color( Color32::from_rgb( 0, 243, 255)).strong().size( 11.0));

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

    // Clear background
    painter.rect_filled( rect, 0.0, Color32::from_rgb( 11, 15, 25));

    // Load points in-process
    let  	points: Buff< [f32; 3]> = if path.exists() {
        let  	pathStr = path.to_string_lossy().to_string();
        if let  	Ok( dto) = XplrParsePtsFile( &pathStr) {
            dto._Points
        } else {
            Buff::New()
        }
    } else {
        let  	mut mock = Buff::New();
        for i in 0..1000 {
            let  	t = i as f32 * 0.1;
            mock.Push( [t.sin() * 50.0, (t * 0.5).cos() * 50.0, (t * 0.2).sin() * 50.0]);
        }
        mock
    };

    let  	pointCount = points.len();
    let  	center = rect.center();
    let  	cosY = camera._RotY.cos();
    let  	sinY = camera._RotY.sin();
    let  	cosX = camera._RotX.cos();
    let  	sinX = camera._RotX.sin();

    // 1. Draw 3D Bounding Box Wireframe
    let  	boxHalf = 60.0 * camera._Zoom;
    let  	boxVerts = [
        [-boxHalf, -boxHalf, -boxHalf], [ boxHalf, -boxHalf, -boxHalf],
        [ boxHalf,  boxHalf, -boxHalf], [-boxHalf,  boxHalf, -boxHalf],
        [-boxHalf, -boxHalf,  boxHalf], [ boxHalf, -boxHalf,  boxHalf],
        [ boxHalf,  boxHalf,  boxHalf], [-boxHalf,  boxHalf,  boxHalf],
    ];

    let  	project = |v: [f32; 3]| -> Pos2 {
        let  	x1 = v[0] * cosY + v[2] * sinY;
        let  	z1 = -v[0] * sinY + v[2] * cosY;
        let  	y2 = v[1] * cosX - z1 * sinX;
        let  	z2 = v[1] * sinX + z1 * cosX;
        let  	scale = 400.0 / (400.0 + z2).max( 10.0);
        Pos2::new( center.x + camera._PanX + x1 * scale, center.y + camera._PanY - y2 * scale)
    };

    let  	projBox: Vec< Pos2> = boxVerts.iter().map( |&v| project( v)).collect();
    let  	boxEdges = [
        (0,1),(1,2),(2,3),(3,0),
        (4,5),(5,6),(6,7),(7,4),
        (0,4),(1,5),(2,6),(3,7)
    ];

    let  	boxStroke = Stroke::new( 1.0, Color32::from_rgba_premultiplied( 0, 243, 255, 35));
    for (i, j) in boxEdges {
        painter.line_segment( [projBox[i], projBox[j]], boxStroke);
    }

    // 2. Draw Points with depth-shaded glow
    let  	ptColor = Color32::from_rgb( 0, 243, 255);
    for pt in points.iter() {
        let  	scaledPt = [pt[0] * camera._Zoom, pt[1] * camera._Zoom, pt[2] * camera._Zoom];
        let  	p = project( scaledPt);
        if rect.contains( p) {
            painter.circle_filled( p, 2.0, ptColor);
        }
    }

    // 3. HUD Overlay
    let  	hudText = format!( "{} Points | Zoom: {:.2}x | Rot: ({:.2}, {:.2}) | Pan: ({:.0}, {:.0})", pointCount, camera._Zoom, camera._RotX, camera._RotY, camera._PanX, camera._PanY);
    painter.text(
        rect.left_bottom() + Vec2::new( 16.0, -16.0),
        egui::Align2::LEFT_BOTTOM,
        hudText,
        egui::FontId::monospace( 12.0),
        Color32::from_rgb( 148, 163, 184),
    );

    ui.ctx().request_repaint();
}

// ---------------------------------------------------------------------------------------------------------------------------------
