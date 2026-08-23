//-- frieze/geom_view.rs ------------------------------------------------------------------------------------------------------------
//! Unified native 3D geometry viewport for rendering Point Clouds (.pts) and Wavefront meshes (.obj),
//! with mouse orbit/pan/zoom, reset/fit controls, auto-rotate idle timer, and selectable render modes.
use	std::cell::Cell;
use	std::path::{ Path, PathBuf };
use	std::rc::Rc;

use	wxdragon::color::Colour;
use	wxdragon::dc::auto_buffered_paint_dc::AutoBufferedPaintDC;
use	wxdragon::dc::{ BrushStyle, DeviceContext, PenStyle, Point as DcPoint, PolygonFillMode };
use	wxdragon::event::window_events::WindowEventData;
use	wxdragon::prelude::*;
use	wxdragon::timer::Timer;
use	wxdragon::window::BackgroundStyle;

use	crate::frieze::state::SharedState;
use	crate::swarm::viewport::ObjRenderMode;

// ---------------------------------------------------------------------------------------------------------------------------------

#[cfg( target_os = "windows")]
unsafe extern "system"
{
    fn	GetAsyncKeyState( vKey: i32) -> i16;
}

const	VK_SHIFT:   i32 = 0x10;
const	VK_CONTROL: i32 = 0x11;

fn	is_ctrl_pressed() -> bool
{
    #[cfg( target_os = "windows")]
    unsafe { ( GetAsyncKeyState( VK_CONTROL) as u16 & 0x8000) != 0 }
    #[cfg( not( target_os = "windows"))]
    false
}

fn	is_shift_pressed() -> bool
{
    #[cfg( target_os = "windows")]
    unsafe { ( GetAsyncKeyState( VK_SHIFT) as u16 & 0x8000) != 0 }
    #[cfg( not( target_os = "windows"))]
    false
}

// ---------------------------------------------------------------------------------------------------------------------------------

struct DragState
{
    _LeftDown:  Cell< bool>,
    _RightDown: Cell< bool>,
    _LastX:     Cell< i32>,
    _LastY:     Cell< i32>,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Builds a native unified 3D geometry viewport panel for .pts or .obj files.
pub fn	build_geom_view_panel( parent: &Notebook, state: SharedState, path: PathBuf) -> Panel
{
    let  	panel = Panel::builder( parent).build();
    let  	rootSizer = BoxSizer::builder( Orientation::Vertical).build();

    let  	ext = path.extension().and_then( |e| e.to_str()).unwrap_or( "").to_lowercase();
    let  	fileName = path.file_name().and_then( |n| n.to_str()).unwrap_or( "geometry").to_string();
    let  	titlePrefix = if ext == "pts" { "POINT CLOUD" } else { "WAVEFRONT 3D" };

    let  	toolbar = Panel::builder( &panel).build();
    let  	toolbarSizer = BoxSizer::builder( Orientation::Horizontal).build();
    let  	titleLabel = StaticText::builder( &toolbar)
        .with_label( &format!( "{} {}", titlePrefix, fileName))
        .build();

    toolbarSizer.add( &titleLabel, 1, SizerFlag::AlignCenterVertical | SizerFlag::All, 6);

    let  	pointsBtn = ToggleButton::builder( &toolbar).with_label( "Points").build();
    let  	wireBtn = ToggleButton::builder( &toolbar).with_label( "Wireframe").build();
    let  	facetsBtn = ToggleButton::builder( &toolbar).with_label( "Facets").build();
    let  	shadedBtn = ToggleButton::builder( &toolbar).with_label( "Shaded+Wire").build();

    let  	syncModeButtons: Rc< dyn Fn()> = {
        let  	state = state.clone();
        let  	pointsBtn = pointsBtn.clone();
        let  	wireBtn = wireBtn.clone();
        let  	facetsBtn = facetsBtn.clone();
        let  	shadedBtn = shadedBtn.clone();
        Rc::new( move || {
            let  	mode = state.borrow()._ActiveObjMode;
            pointsBtn.set_value( mode == ObjRenderMode::Points);
            wireBtn.set_value( mode == ObjRenderMode::Wireframe);
            facetsBtn.set_value( mode == ObjRenderMode::Facets);
            shadedBtn.set_value( mode == ObjRenderMode::ShadedWire);
        })
    };
    syncModeButtons();

    toolbarSizer.add( &pointsBtn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 3);
    toolbarSizer.add( &wireBtn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 3);
    toolbarSizer.add( &facetsBtn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 3);
    toolbarSizer.add( &shadedBtn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 3);
    toolbar.set_sizer( toolbarSizer, true);

    let  	canvas = Panel::builder( &panel).build();
    canvas.set_background_style( BackgroundStyle::Paint);
    canvas.on_erase_background( |_| {});

    rootSizer.add( &toolbar, 0, SizerFlag::Expand, 0);
    rootSizer.add( &canvas, 1, SizerFlag::Expand, 0);
    panel.set_sizer( rootSizer, true);

    let  	drag = Rc::new( DragState {
        _LeftDown:  Cell::new( false),
        _RightDown: Cell::new( false),
        _LastX:     Cell::new( 0),
        _LastY:     Cell::new( 0),
    });

    let  	timer = Timer::new( &canvas);
    {
        let  	state = state.clone();
        let  	drag = drag.clone();
        let  	canvas = canvas.clone();
        timer.on_tick( move |_| {
            if !drag._LeftDown.get() && !drag._RightDown.get() {
                state.borrow_mut()._Camera.Rotate( 0.001, 0.003);
            }
            canvas.refresh( false, None);
        });
    }
    timer.start( 33, false);

    {
        let  	canvas = canvas.clone();
        canvas.on_size( move |evt| {
            canvas.refresh( true, None);
            evt.skip( true);
        });
    }

    {
        let  	state = state.clone();
        let  	canvas = canvas.clone();
        canvas.on_mouse_wheel( move |evt| {
            if let WindowEventData::General( e) = evt {
                let  	rot = e.get_wheel_rotation();
                let  	ctrl = e.control_down() || is_ctrl_pressed();
                if ctrl && rot != 0 {
                    let  	mut st = state.borrow_mut();
                    if rot > 0 {
                        st._Camera._Zoom *= 1.1;
                    } else {
                        st._Camera._Zoom /= 1.1;
                    }
                    if st._Camera._Zoom < 0.05 { st._Camera._Zoom = 0.05; }
                    canvas.refresh( false, None);
                }
            }
        });
    }

    {
        let  	drag = drag.clone();
        let  	state = state.clone();
        let  	canvas = canvas.clone();
        canvas.on_mouse_left_down( move |evt| {
            if let WindowEventData::MouseButton( mb) = evt {
                let  	shift = mb.event.event.shift_down() || is_shift_pressed();
                if shift {
                    state.borrow_mut()._Camera.Reset();
                    canvas.refresh( false, None);
                } else if let Some( pos) = mb.get_position() {
                    drag._LeftDown.set( true);
                    drag._LastX.set( pos.x);
                    drag._LastY.set( pos.y);
                }
            }
        });
    }

    {
        let  	drag = drag.clone();
        canvas.on_mouse_left_up( move |_evt| drag._LeftDown.set( false));
    }

    {
        let  	drag = drag.clone();
        let  	state = state.clone();
        let  	canvas = canvas.clone();
        canvas.on_mouse_right_down( move |evt| {
            if let WindowEventData::MouseButton( mb) = evt {
                let  	shift = mb.event.event.shift_down() || is_shift_pressed();
                if shift {
                    state.borrow_mut()._Camera.Fit();
                    canvas.refresh( false, None);
                } else if let Some( pos) = mb.get_position() {
                    drag._RightDown.set( true);
                    drag._LastX.set( pos.x);
                    drag._LastY.set( pos.y);
                }
            }
        });
    }

    {
        let  	drag = drag.clone();
        canvas.on_mouse_right_up( move |_evt| drag._RightDown.set( false));
    }

    {
        let  	state = state.clone();
        let  	drag = drag.clone();
        let  	canvas = canvas.clone();
        canvas.on_mouse_motion( move |evt| {
            if let WindowEventData::MouseMotion( mm) = evt {
                if let Some( pos) = mm.get_position() {
                    let  	dx = ( pos.x - drag._LastX.get()) as f32;
                    let  	dy = ( pos.y - drag._LastY.get()) as f32;
                    let  	ctrl = mm.event.event.control_down() || is_ctrl_pressed();

                    if drag._LeftDown.get() && drag._RightDown.get() {
                        let  	zoomFactor = 1.0 + ( dy * 0.01);
                        let  	mut st = state.borrow_mut();
                        st._Camera._Zoom *= zoomFactor;
                        if st._Camera._Zoom < 0.05 { st._Camera._Zoom = 0.05; }
                        canvas.refresh( false, None);
                    } else if drag._LeftDown.get() && ctrl {
                        let  	zoomFactor = 1.0 + ( dy * 0.01);
                        let  	mut st = state.borrow_mut();
                        st._Camera._Zoom *= zoomFactor;
                        if st._Camera._Zoom < 0.05 { st._Camera._Zoom = 0.05; }
                        canvas.refresh( false, None);
                    } else if drag._RightDown.get() {
                        state.borrow_mut()._Camera.Pan( dx, dy);
                        canvas.refresh( false, None);
                    } else if drag._LeftDown.get() {
                        state.borrow_mut()._Camera.Rotate( dy * 0.01, dx * 0.01);
                        canvas.refresh( false, None);
                    }
                    drag._LastX.set( pos.x);
                    drag._LastY.set( pos.y);
                }
            }
        });
    }

    let  	bindMode = |btn: ToggleButton, mode: ObjRenderMode| {
        let  	state = state.clone();
        let  	sync = syncModeButtons.clone();
        let  	canvas = canvas.clone();
        btn.on_click( move |_| {
            state.borrow_mut()._ActiveObjMode = mode;
            sync();
            canvas.refresh( false, None);
        });
    };
    bindMode( pointsBtn, ObjRenderMode::Points);
    bindMode( wireBtn, ObjRenderMode::Wireframe);
    bindMode( facetsBtn, ObjRenderMode::Facets);
    bindMode( shadedBtn, ObjRenderMode::ShadedWire);

    {
        let  	state = state.clone();
        let  	path = path.clone();
        let  	canvas = canvas.clone();
        canvas.on_paint( move |_evt| {
            let  	dc = AutoBufferedPaintDC::new( &canvas);
            draw_geom_frame( &dc, &canvas, &state, &path);
        });
    }

    return panel;
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	draw_geom_frame( dc: &AutoBufferedPaintDC, canvas: &Panel, state: &SharedState, path: &Path)
{
    let  	bg = state.borrow()._Theme.viewport_rgb();
    dc.set_background( Colour::rgb( bg.0, bg.1, bg.2));
    dc.clear();

    let  	size = canvas.get_client_size();
    let  	width = size.width as f32;
    let  	height = size.height as f32;

    let  	mesh = {
        let  	mut st = state.borrow_mut();
        st._MeshCache.GetOrLoad( None, path).cloned()
    };

    let  	( camera, mode) = {
        let  	st = state.borrow();
        ( st._Camera, st._ActiveObjMode)
    };

    let  	centerX = width / 2.0;
    let  	centerY = height / 2.0;
    let  	cosY = camera._RotY.cos();
    let  	sinY = camera._RotY.sin();
    let  	cosX = camera._RotX.cos();
    let  	sinX = camera._RotX.sin();

    let  	project = |v: [f32; 3]| -> ( f32, f32, f32) {
        let  	x1 = v[0] * cosY + v[2] * sinY;
        let  	z1 = -v[0] * sinY + v[2] * cosY;
        let  	y2 = v[1] * cosX - z1 * sinX;
        let  	z2 = v[1] * sinX + z1 * cosX;
        let  	scale = ( 350.0 * camera._Zoom) / ( 300.0 + z2).max( 10.0);
        ( centerX + camera._PanX + x1 * scale, centerY + camera._PanY - y2 * scale, z2)
    };

    let  	Some( mesh) = mesh else {
        dc.set_text_foreground( Colour::rgb( 166, 173, 200));
        dc.draw_text( "No geometry data available", 12, 10);
        return;
    };

    let  	cx = mesh._Center[0];
    let  	cy = mesh._Center[1];
    let  	cz = mesh._Center[2];
    let  	scaleNorm = mesh._ScaleNorm;

    let  	localOf = |p: [f32; 3]| -> [f32; 3] {
        [( p[0] - cx) * scaleNorm, ( p[1] - cy) * scaleNorm, ( p[2] - cz) * scaleNorm]
    };

    let  	hasFaces = mesh.FaceCount() > 0;
    let  	projectedVerts: Vec< ( f32, f32, f32)> = mesh._Points.iter().map( |&p| project( localOf( p))).collect();

    // If point cloud only, draw points and bounding box
    if !hasFaces {
        let  	[minX, minY, minZ] = mesh._BboxMin;
        let  	[maxX, maxY, maxZ] = mesh._BboxMax;
        let  	bboxVerts = [
            [( minX - cx) * scaleNorm, ( minY - cy) * scaleNorm, ( minZ - cz) * scaleNorm],
            [( maxX - cx) * scaleNorm, ( minY - cy) * scaleNorm, ( minZ - cz) * scaleNorm],
            [( maxX - cx) * scaleNorm, ( maxY - cy) * scaleNorm, ( minZ - cz) * scaleNorm],
            [( minX - cx) * scaleNorm, ( maxY - cy) * scaleNorm, ( minZ - cz) * scaleNorm],
            [( minX - cx) * scaleNorm, ( minY - cy) * scaleNorm, ( maxZ - cz) * scaleNorm],
            [( maxX - cx) * scaleNorm, ( minY - cy) * scaleNorm, ( maxZ - cz) * scaleNorm],
            [( maxX - cx) * scaleNorm, ( maxY - cy) * scaleNorm, ( maxZ - cz) * scaleNorm],
            [( minX - cx) * scaleNorm, ( maxY - cy) * scaleNorm, ( maxZ - cz) * scaleNorm],
        ];
        let  	projBox: Vec< ( f32, f32, f32)> = bboxVerts.iter().map( |&v| project( v)).collect();
        let  	boxEdges = [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4), (0, 4), (1, 5), (2, 6), (3, 7)];

        dc.set_pen( Colour::new( 0, 243, 255, 90), 1, PenStyle::Solid);
        for &( a, b) in &boxEdges {
            let  	( x1, y1, _) = projBox[a];
            let  	( x2, y2, _) = projBox[b];
            dc.draw_line( x1 as i32, y1 as i32, x2 as i32, y2 as i32);
        }

        dc.set_pen( Colour::rgb( 137, 220, 235), 1, PenStyle::Solid);
        dc.set_brush( Colour::rgb( 137, 220, 235), BrushStyle::Solid);
        for &( x, y, _) in &projectedVerts {
            dc.draw_circle( x as i32, y as i32, 1);
        }

        dc.set_text_foreground( Colour::rgb( 137, 220, 235));
        dc.draw_text( &format!( "{} points", mesh.PointCount()), 12, 10);
        return;
    }

    // Mesh rendering with triangles
    if mode == ObjRenderMode::Facets || mode == ObjRenderMode::ShadedWire {
        let  	mut tris: Vec< ( usize, f32)> = mesh
            ._Triangles
            .iter()
            .enumerate()
            .map( |( i, tri)| {
                let  	zAvg = ( projectedVerts[tri[0] as usize].2
                    + projectedVerts[tri[1] as usize].2
                    + projectedVerts[tri[2] as usize].2)
                    / 3.0;
                ( i, zAvg)
            })
            .collect();
        tris.sort_by( |a, b| b.1.partial_cmp( &a.1).unwrap_or( std::cmp::Ordering::Equal));

        dc.set_pen( Colour::rgb( 203, 166, 247), 1, PenStyle::Solid);
        for ( i, z) in tris {
            let  	tri = mesh._Triangles[i];
            let  	a = projectedVerts[tri[0] as usize];
            let  	b = projectedVerts[tri[1] as usize];
            let  	c = projectedVerts[tri[2] as usize];
            let  	shade = ( ( 200.0 - z).clamp( 60.0, 220.0)) as u8;
            dc.set_brush( Colour::rgb( shade, ( shade as f32 * 0.75) as u8, ( shade as f32 * 0.95) as u8), BrushStyle::Solid);
            let  	poly = [
                DcPoint { x: a.0 as i32, y: a.1 as i32 },
                DcPoint { x: b.0 as i32, y: b.1 as i32 },
                DcPoint { x: c.0 as i32, y: c.1 as i32 },
            ];
            dc.draw_polygon( &poly, 0, 0, PolygonFillMode::OddEven);
        }
    }

    if mode == ObjRenderMode::Wireframe || mode == ObjRenderMode::ShadedWire {
        dc.set_pen( Colour::rgb( 203, 166, 247), 1, PenStyle::Solid);
        for edge in mesh._Triangles.iter() {
            let  	a = projectedVerts[edge[0] as usize];
            let  	b = projectedVerts[edge[1] as usize];
            let  	c = projectedVerts[edge[2] as usize];
            dc.draw_line( a.0 as i32, a.1 as i32, b.0 as i32, b.1 as i32);
            dc.draw_line( b.0 as i32, b.1 as i32, c.0 as i32, c.1 as i32);
            dc.draw_line( c.0 as i32, c.1 as i32, a.0 as i32, a.1 as i32);
        }
    }

    if mode == ObjRenderMode::Points {
        dc.set_pen( Colour::rgb( 203, 166, 247), 1, PenStyle::Solid);
        dc.set_brush( Colour::rgb( 203, 166, 247), BrushStyle::Solid);
        for &( x, y, _) in &projectedVerts {
            dc.draw_circle( x as i32, y as i32, 1);
        }
    }

    dc.set_text_foreground( Colour::rgb( 203, 166, 247));
    dc.draw_text( &format!( "{} verts | {} faces", mesh.PointCount(), mesh.FaceCount()), 12, 10);
    return;
}