//-- frieze/wave_view.rs -----------------------------------------------------------------------------------------------------------
//! Interactive digital waveform viewer panel for Value Change Dump (.vcd) files using native wxDragon DC drawing.
use	std::cell::RefCell;
use	std::path::PathBuf;
use	std::rc::Rc;

use	wxdragon::color::Colour;
use	wxdragon::dc::auto_buffered_paint_dc::AutoBufferedPaintDC;
use	wxdragon::dc::{ BrushStyle, DeviceContext, PenStyle };
use	wxdragon::event::window_events::WindowEventData;
use	wxdragon::prelude::*;
use	wxdragon::widgets::treectrl::{ TreeCtrl, TreeCtrlStyle, TreeItemId };
use	wxdragon::window::BackgroundStyle;
use	wxdragon::HasItemData;

use	crate::frieze::state::SharedState;
use	crate::rube::{ VcdDisplayModel, VcdScope, VcdSignal };
use	crate::silo::{ Buff, IAccess, U32, USeg };

// ---------------------------------------------------------------------------------------------------------------------------------

const	NAME_COL_WIDTH: f32 = 180.0;
const	RULER_HEIGHT:   f32 = 28.0;
const	ROW_HEIGHT:     f32 = 26.0;

// ---------------------------------------------------------------------------------------------------------------------------------

#[cfg( target_os = "windows")]
unsafe extern "system"
{
    fn	GetAsyncKeyState( vKey: i32) -> i16;
}

const	VK_CONTROL: i32 = 0x11;

fn	is_ctrl_pressed() -> bool
{
    #[cfg( target_os = "windows")]
    unsafe { ( GetAsyncKeyState( VK_CONTROL) as u16 & 0x8000) != 0 }
    #[cfg( not( target_os = "windows"))]
    false
}

// ---------------------------------------------------------------------------------------------------------------------------------

struct WaveViewInner
{
    _Model:       VcdDisplayModel,
    _ViewStart:   f64,
    _ViewEnd:     f64,
    _ScrollY:     i32,
    _SelectedSig: Option< U32>,
    _CursorTime:  Option< u64>,
    _LeftDown:    bool,
    _LastX:       i32,
    _LastY:       i32,
}

impl WaveViewInner
{
    fn	New( model: VcdDisplayModel) -> Self
    {
        let  	timeMin = model._TimeMin as f64;
        let  	timeMax = ( model._TimeMax as f64).max( timeMin + 10.0);
        Self {
            _Model:       model,
            _ViewStart:   timeMin,
            _ViewEnd:     timeMax,
            _ScrollY:     0,
            _SelectedSig: None,
            _CursorTime:  None,
            _LeftDown:    false,
            _LastX:       0,
            _LastY:       0,
        }
    }

    fn	Fit( &mut self)
    {
        self._ViewStart = self._Model._TimeMin as f64;
        self._ViewEnd = ( self._Model._TimeMax as f64).max( self._Model._TimeMin as f64 + 1.0);
        self._ScrollY = 0;
    }

    fn	Zoom( &mut self, factor: f64, pivotRatio: f64)
    {
        let  	span = self._ViewEnd - self._ViewStart;
        let  	newSpan = ( span * factor).max( 1.0);
        let  	pivotTime = self._ViewStart + span * pivotRatio.clamp( 0.0, 1.0);
        self._ViewStart = pivotTime - newSpan * pivotRatio.clamp( 0.0, 1.0);
        self._ViewEnd = self._ViewStart + newSpan;
    }

    fn	Pan( &mut self, dt: f64)
    {
        self._ViewStart += dt;
        self._ViewEnd += dt;
    }

    #[inline]
    fn	TimeToX( &self, time: f64, waveAreaWidth: f64) -> f64
    {
        let  	span = ( self._ViewEnd - self._ViewStart).max( 1.0);
        let  	ratio = ( time - self._ViewStart) / span;
        return NAME_COL_WIDTH as f64 + ratio * waveAreaWidth;
    }

    #[inline]
    fn	XToTime( &self, x: f64, waveAreaWidth: f64) -> f64
    {
        let  	relX = ( x - NAME_COL_WIDTH as f64).max( 0.0);
        let  	span = ( self._ViewEnd - self._ViewStart).max( 1.0);
        let  	ratio = if waveAreaWidth > 0.0 { relX / waveAreaWidth } else { 0.0 };
        return self._ViewStart + ratio * span;
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Builds the native interactive waveform viewport panel for VCD files.
pub fn	build_wave_view_panel(
    parent: &Notebook,
    state: SharedState,
    displayModel: VcdDisplayModel,
    path: PathBuf,
) -> Panel
{
    let  	panel = Panel::builder( parent).build();
    let  	rootSizer = BoxSizer::builder( Orientation::Vertical).build();

    let  	fileName = path.file_name().and_then( |n| n.to_str()).unwrap_or( "trace.vcd").to_string();
    let  	sigCount = displayModel.SignalCount().0;
    let  	timescale = displayModel._Timescale.clone();

    // 1. Toolbar
    let  	toolbar = Panel::builder( &panel).build();
    let  	toolbarSizer = BoxSizer::builder( Orientation::Horizontal).build();

    let  	titleLabel = StaticText::builder( &toolbar)
        .with_label( &format!( "WAVEFORM  {}  ({} signals, timescale: {})", fileName, sigCount, timescale))
        .build();
    toolbarSizer.add( &titleLabel, 1, SizerFlag::AlignCenterVertical | SizerFlag::All, 6);

    let  	fitBtn = Button::builder( &toolbar).with_label( "Fit").build();
    let  	zoomInBtn = Button::builder( &toolbar).with_label( "Zoom +").build();
    let  	zoomOutBtn = Button::builder( &toolbar).with_label( "Zoom -").build();
    let  	statusLabel = StaticText::builder( &toolbar).with_label( "Cursor: --").build();

    toolbarSizer.add( &statusLabel, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 6);
    toolbarSizer.add( &zoomInBtn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 3);
    toolbarSizer.add( &zoomOutBtn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 3);
    toolbarSizer.add( &fitBtn, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 3);
    toolbar.set_sizer( toolbarSizer, true);

    // 2. Main Area: Sidebar tree on left + Waveform canvas on right
    let  	bodyPanel = Panel::builder( &panel).build();
    let  	bodySizer = BoxSizer::builder( Orientation::Horizontal).build();

    let  	sidebar = TreeCtrl::builder( &bodyPanel)
        .with_style( TreeCtrlStyle::HasButtons | TreeCtrlStyle::LinesAtRoot)
        .build();

    let  	canvas = Panel::builder( &bodyPanel).build();
    canvas.set_background_style( BackgroundStyle::Paint);
    canvas.on_erase_background( |_| {});

    bodySizer.add( &sidebar, 0, SizerFlag::Expand, 0);
    bodySizer.add( &canvas, 1, SizerFlag::Expand, 0);
    bodyPanel.set_sizer( bodySizer, true);

    rootSizer.add( &toolbar, 0, SizerFlag::Expand, 0);
    rootSizer.add( &bodyPanel, 1, SizerFlag::Expand, 0);
    panel.set_sizer( rootSizer, true);

    let  	inner = Rc::new( RefCell::new( WaveViewInner::New( displayModel)));

    // Populate Sidebar Scope Tree
    populate_scope_tree( &sidebar, &inner.borrow()._Model);

    // Sidebar selection handler: highlight signal and scroll into view
    {
        let  	inner = inner.clone();
        let  	canvas = canvas.clone();
        let  	statusLabel = statusLabel.clone();
        sidebar.on_selection_changed( move |evt| {
            let  	Some( itemId) = evt.get_item() else { return };
            let  	Some( data) = sidebar.get_custom_data( &itemId) else { return };
            if let Some( &sigIdx) = data.downcast_ref::< u32>() {
                let  	mut inn = inner.borrow_mut();
                inn._SelectedSig = Some( U32( sigIdx));
                // Scroll into view if needed
                let  	targetY = sigIdx as i32 * ROW_HEIGHT as i32;
                inn._ScrollY = ( targetY - 100).max( 0);
                if let Some( sig) = inn._Model.Signal( U32( sigIdx)) {
                    statusLabel.set_label( &format!( "Selected: {} [{}]", sig._FullName, sig._Bits.0));
                }
                canvas.refresh( false, None);
            }
        });
    }

    // Button event handlers
    {
        let  	inner = inner.clone();
        let  	canvas = canvas.clone();
        fitBtn.on_click( move |_| {
            inner.borrow_mut().Fit();
            canvas.refresh( false, None);
        });
    }

    {
        let  	inner = inner.clone();
        let  	canvas = canvas.clone();
        zoomInBtn.on_click( move |_| {
            inner.borrow_mut().Zoom( 0.7, 0.5);
            canvas.refresh( false, None);
        });
    }

    {
        let  	inner = inner.clone();
        let  	canvas = canvas.clone();
        zoomOutBtn.on_click( move |_| {
            inner.borrow_mut().Zoom( 1.4, 0.5);
            canvas.refresh( false, None);
        });
    }

    // Canvas size event
    {
        let  	canvas = canvas.clone();
        canvas.on_size( move |evt| {
            canvas.refresh( true, None);
            evt.skip( true);
        });
    }

    // Mouse wheel: Ctrl+wheel zooms time, regular wheel scrolls vertically
    {
        let  	inner = inner.clone();
        let  	canvas = canvas.clone();
        canvas.on_mouse_wheel( move |evt| {
            if let WindowEventData::General( e) = evt {
                let  	rot = e.get_wheel_rotation();
                let  	ctrl = e.control_down() || is_ctrl_pressed();
                if rot != 0 {
                    let  	mut inn = inner.borrow_mut();
                    if ctrl {
                        let  	pivotRatio = 0.5;
                        if rot > 0 {
                            inn.Zoom( 0.8, pivotRatio);
                        } else {
                            inn.Zoom( 1.25, pivotRatio);
                        }
                    } else {
                        inn._ScrollY = ( inn._ScrollY - rot / 2).max( 0);
                    }
                    canvas.refresh( false, None);
                }
            }
        });
    }

    // Mouse left down: set cursor time, start drag, or select row
    {
        let  	inner = inner.clone();
        let  	canvas = canvas.clone();
        let  	statusLabel = statusLabel.clone();
        canvas.on_mouse_left_down( move |evt| {
            if let WindowEventData::MouseButton( mb) = evt {
                if let Some( pos) = mb.get_position() {
                    let  	mut inn = inner.borrow_mut();
                    inn._LeftDown = true;
                    inn._LastX = pos.x;
                    inn._LastY = pos.y;

                    let  	size = canvas.get_client_size();
                    let  	waveAreaW = ( size.width as f64 - NAME_COL_WIDTH as f64).max( 1.0);

                    if pos.x as f32 >= NAME_COL_WIDTH {
                        let  	t = inn.XToTime( pos.x as f64, waveAreaW).round() as u64;
                        inn._CursorTime = Some( t);
                        statusLabel.set_label( &format!( "Time: #{} {}", t, inn._Model._Timescale));
                    }

                    // Check which signal row was clicked
                    if pos.y as f32 >= RULER_HEIGHT {
                        let  	clickedRow = ( ( pos.y as f32 - RULER_HEIGHT + inn._ScrollY as f32) / ROW_HEIGHT) as u32;
                        if clickedRow < inn._Model.SignalCount().0 {
                            inn._SelectedSig = Some( U32( clickedRow));
                            if let Some( sig) = inn._Model.Signal( U32( clickedRow)) {
                                let  	valStr = if let Some( ct) = inn._CursorTime {
                                    sig.ValueAt( ct)
                                } else {
                                    "--"
                                };
                                statusLabel.set_label( &format!( "{} = {}", sig._FullName, valStr));
                            }
                        }
                    }
                    canvas.refresh( false, None);
                }
            }
        });
    }

    // Mouse left up
    {
        let  	inner = inner.clone();
        canvas.on_mouse_left_up( move |_evt| {
            inner.borrow_mut()._LeftDown = false;
        });
    }

    // Mouse motion: pan time or scrub cursor
    {
        let  	inner = inner.clone();
        let  	canvas = canvas.clone();
        let  	statusLabel = statusLabel.clone();
        canvas.on_mouse_motion( move |evt| {
            if let WindowEventData::MouseMotion( mm) = evt {
                if let Some( pos) = mm.get_position() {
                    let  	mut inn = inner.borrow_mut();
                    if inn._LeftDown {
                        let  	dx = ( pos.x - inn._LastX) as f64;
                        let  	size = canvas.get_client_size();
                        let  	waveAreaW = ( size.width as f64 - NAME_COL_WIDTH as f64).max( 1.0);
                        let  	span = inn._ViewEnd - inn._ViewStart;
                        let  	dt = -dx * ( span / waveAreaW);
                        inn.Pan( dt);

                        if pos.x as f32 >= NAME_COL_WIDTH {
                            let  	t = inn.XToTime( pos.x as f64, waveAreaW).round() as u64;
                            inn._CursorTime = Some( t);
                            statusLabel.set_label( &format!( "Time: #{} {}", t, inn._Model._Timescale));
                        }

                        inn._LastX = pos.x;
                        inn._LastY = pos.y;
                        canvas.refresh( false, None);
                    }
                }
            }
        });
    }

    // Paint event
    {
        let  	inner = inner.clone();
        let  	state = state.clone();
        let  	canvas = canvas.clone();
        canvas.on_paint( move |_evt| {
            let  	dc = AutoBufferedPaintDC::new( &canvas);
            let  	size = canvas.get_client_size();
            let  	width = size.width as f32;
            let  	height = size.height as f32;

            draw_waveform_canvas( &dc, &inner.borrow(), &state, width, height);
        });
    }

    return panel;
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	populate_scope_tree( tree: &TreeCtrl, model: &VcdDisplayModel)
{
    tree.delete_all_items();
    let  	rootName = "Signals";

    if let Some( rootItem) = tree.add_root_with_data( rootName, 0u32, None, None) {
        let  	mut curSigIdx = 0u32;
        populate_scope_nodes( tree, &rootItem, &model._Scopes, &mut curSigIdx);
        tree.expand( &rootItem);
    }
}

fn	populate_scope_nodes( tree: &TreeCtrl, parentItem: &TreeItemId, scopes: &Buff< VcdScope>, curSigIdx: &mut u32)
{
    scopes.Arr().Traverse( |scope| {
        let  	scopeLabel = format!( "{} ({})", scope._Name, scope._Type);
        if let Some( scopeItem) = tree.append_item_with_data( parentItem, &scopeLabel, 0u32, None, None) {
            // Append signals in this scope
            scope._Vars.Arr().Traverse( |var| {
                let  	sigLabel = format!( "{} [{}]", var._Name, var._Bits.0);
                let  	thisIdx = *curSigIdx;
                *curSigIdx += 1;
                tree.append_item_with_data( &scopeItem, &sigLabel, thisIdx, None, None);
            });
            // Recurse child scopes
            populate_scope_nodes( tree, &scopeItem, &scope._Scopes, curSigIdx);
            tree.expand( &scopeItem);
        }
    });
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	draw_waveform_canvas(
    dc: &AutoBufferedPaintDC,
    inner: &WaveViewInner,
    state: &SharedState,
    width: f32,
    height: f32,
)
{
    let  	bg = state.borrow()._Theme.viewport_rgb();
    dc.set_background( Colour::rgb( bg.0, bg.1, bg.2));
    dc.clear();

    let  	waveAreaW = ( width - NAME_COL_WIDTH).max( 1.0) as f64;

    // 1. Draw Time Ruler at top
    draw_time_ruler( dc, inner, width, waveAreaW);

    // 2. Draw Signal Rows
    draw_signal_rows( dc, inner, width, height, waveAreaW);

    // 3. Draw Cursor Line
    draw_cursor_marker( dc, inner, width, height, waveAreaW);

    // 4. Draw Header Column separator line
    dc.set_pen( Colour::rgb( 69, 71, 90), 1, PenStyle::Solid);
    dc.draw_line( NAME_COL_WIDTH as i32, 0, NAME_COL_WIDTH as i32, height as i32);
    return;
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	draw_time_ruler( dc: &AutoBufferedPaintDC, inner: &WaveViewInner, width: f32, waveAreaW: f64)
{
    // Ruler background
    dc.set_pen( Colour::rgb( 49, 50, 68), 1, PenStyle::Solid);
    dc.set_brush( Colour::rgb( 24, 24, 37), BrushStyle::Solid);
    dc.draw_rectangle( 0, 0, width as i32, RULER_HEIGHT as i32);

    // Corner title
    dc.set_text_foreground( Colour::rgb( 137, 220, 235));
    dc.draw_text( "Signals / Time", 8, 6);

    let  	span = ( inner._ViewEnd - inner._ViewStart).max( 1.0);
    let  	rawStep = span / 10.0;
    let  	magnitude = 10.0f64.powf( rawStep.log10().floor());
    let  	normalized = rawStep / magnitude;
    let  	step = if normalized < 2.0 {
        1.0 * magnitude
    } else if normalized < 5.0 {
        2.0 * magnitude
    } else {
        5.0 * magnitude
    };

    let  	firstTick = ( inner._ViewStart / step).floor() * step;
    let  	mut curTick = firstTick;

    dc.set_pen( Colour::rgb( 88, 91, 112), 1, PenStyle::Solid);
    dc.set_text_foreground( Colour::rgb( 166, 173, 200));

    while curTick <= inner._ViewEnd + step {
        let  	x = inner.TimeToX( curTick, waveAreaW) as f32;
        if x >= NAME_COL_WIDTH && x <= width {
            dc.draw_line( x as i32, ( RULER_HEIGHT - 8.0) as i32, x as i32, RULER_HEIGHT as i32);
            let  	label = format!( "{}", curTick as u64);
            dc.draw_text( &label, ( x + 3.0) as i32, 6);
        }
        curTick += step;
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	draw_signal_rows(
    dc: &AutoBufferedPaintDC,
    inner: &WaveViewInner,
    width: f32,
    height: f32,
    waveAreaW: f64,
)
{
    let  	sigCount = inner._Model.SignalCount().0;

    USeg::New( 0, sigCount).Traverse( |sigIdx| {
        let  	y = RULER_HEIGHT + ( sigIdx.0 as f32 * ROW_HEIGHT) - inner._ScrollY as f32;
        if y + ROW_HEIGHT < RULER_HEIGHT || y > height {
            return;
        }

        let  	sig = inner._Model.Signal( sigIdx).unwrap();
        let  	isSelected = inner._SelectedSig == Some( sigIdx);

        // Row background
        if isSelected {
            dc.set_pen( Colour::rgb( 69, 71, 90), 1, PenStyle::Solid);
            dc.set_brush( Colour::rgb( 49, 50, 68), BrushStyle::Solid);
        } else if sigIdx.0 % 2 == 1 {
            dc.set_pen( Colour::rgb( 24, 24, 37), 1, PenStyle::Solid);
            dc.set_brush( Colour::rgb( 20, 20, 32), BrushStyle::Solid);
        } else {
            dc.set_pen( Colour::rgb( 17, 17, 27), 1, PenStyle::Solid);
            dc.set_brush( Colour::rgb( 17, 17, 27), BrushStyle::Solid);
        }
        dc.draw_rectangle( 0, y as i32, width as i32, ROW_HEIGHT as i32);

        // Row grid line
        dc.set_pen( Colour::rgb( 30, 30, 46), 1, PenStyle::Solid);
        dc.draw_line( 0, ( y + ROW_HEIGHT) as i32, width as i32, ( y + ROW_HEIGHT) as i32);

        // Signal name column
        let  	textCol = if isSelected {
            Colour::rgb( 245, 194, 231)
        } else if sig.IsSingleBit() {
            Colour::rgb( 166, 227, 161)
        } else {
            Colour::rgb( 250, 179, 135)
        };
        dc.set_text_foreground( textCol);
        let  	displayName = if sig._Bits.0 > 1 {
            format!( "{} [{}]", sig._Name, sig._Bits.0)
        } else {
            sig._Name.clone()
        };
        dc.draw_text( &displayName, 8, ( y + 5.0) as i32);

        // Waveform Drawing
        draw_single_signal_trace( dc, inner, sig, y, width, waveAreaW);
    });
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	draw_single_signal_trace(
    dc: &AutoBufferedPaintDC,
    inner: &WaveViewInner,
    sig: &VcdSignal,
    rowY: f32,
    canvasWidth: f32,
    waveAreaW: f64,
)
{
    let  	changes = sig._Changes.Arr();
    if changes.IsEmpty() {
        // Unknown constant rail
        let  	midY = rowY + ROW_HEIGHT * 0.5;
        dc.set_pen( Colour::rgb( 243, 139, 168), 1, PenStyle::Dot);
        dc.draw_line( NAME_COL_WIDTH as i32, midY as i32, canvasWidth as i32, midY as i32);
        return;
    }

    let  	isSingleBit = sig.IsSingleBit();
    let  	highY = rowY + 5.0;
    let  	lowY = rowY + ROW_HEIGHT - 6.0;
    let  	midY = rowY + ROW_HEIGHT * 0.5;

    let  	count = changes.Size().0;

    USeg::New( 0, count).Traverse( |cIdx| {
        let  	time = changes[cIdx].0 as f64;
        let  	val = &changes[cIdx].1;
        let  	nextTime = if cIdx.0 + 1 < count {
            changes[cIdx + U32( 1)].0 as f64
        } else {
            inner._ViewEnd.max( time + 1.0)
        };

        if nextTime < inner._ViewStart || time > inner._ViewEnd {
            return;
        }

        let  	x1 = ( inner.TimeToX( time, waveAreaW) as f32).max( NAME_COL_WIDTH);
        let  	x2 = ( inner.TimeToX( nextTime, waveAreaW) as f32).min( canvasWidth);

        if x2 < x1 {
            return;
        }

        if isSingleBit {
            // 1-bit digital rails
            let  	( lineY, col) = if val == "1" {
                ( highY, Colour::rgb( 166, 227, 161) )
            } else if val == "0" {
                ( lowY, Colour::rgb( 148, 156, 187) )
            } else {
                ( midY, Colour::rgb( 243, 139, 168) )
            };

            dc.set_pen( col, 1, PenStyle::Solid);
            // Horizontal segment
            dc.draw_line( x1 as i32, lineY as i32, x2 as i32, lineY as i32);

            // Vertical transition to next value if within view
            if cIdx.0 + 1 < count {
                let  	nextVal = &changes[cIdx + U32( 1)].1;
                let  	nextY = if nextVal == "1" { highY } else if nextVal == "0" { lowY } else { midY };
                if ( x2 - x1).abs() > 0.1 && x2 >= NAME_COL_WIDTH && x2 <= canvasWidth {
                    dc.draw_line( x2 as i32, lineY as i32, x2 as i32, nextY as i32);
                }
            }
        } else {
            // Multi-bit Bus
            let  	busCol = if val.contains( 'x') || val.contains( 'X') {
                Colour::rgb( 243, 139, 168)
            } else {
                Colour::rgb( 137, 180, 250)
            };
            dc.set_pen( busCol, 1, PenStyle::Solid);

            // Top rail
            dc.draw_line( x1 as i32, highY as i32, x2 as i32, highY as i32);
            // Bottom rail
            dc.draw_line( x1 as i32, lowY as i32, x2 as i32, lowY as i32);

            // Bus crossover transition
            dc.draw_line( x1 as i32, highY as i32, ( x1 + 3.0) as i32, lowY as i32);
            dc.draw_line( x1 as i32, lowY as i32, ( x1 + 3.0) as i32, highY as i32);

            // Bus value label centered inside if there's enough space
            let  	segWidth = x2 - x1;
            if segWidth > 28.0 {
                dc.set_text_foreground( Colour::rgb( 205, 214, 244));
                let  	displayVal = if val.len() > 8 {
                    format!( "{}...", &val[..6])
                } else {
                    val.clone()
                };
                let  	textX = ( x1 + 6.0) as i32;
                dc.draw_text( &displayVal, textX, ( rowY + 5.0) as i32);
            }
        }
    });
}

// ---------------------------------------------------------------------------------------------------------------------------------

fn	draw_cursor_marker(
    dc: &AutoBufferedPaintDC,
    inner: &WaveViewInner,
    width: f32,
    height: f32,
    waveAreaW: f64,
)
{
    let  	Some( cursorTime) = inner._CursorTime else { return };
    let  	cursorX = inner.TimeToX( cursorTime as f64, waveAreaW) as f32;

    if cursorX < NAME_COL_WIDTH || cursorX > width {
        return;
    }

    // Vertical cursor line
    dc.set_pen( Colour::rgb( 243, 139, 168), 1, PenStyle::Solid);
    dc.draw_line( cursorX as i32, 0, cursorX as i32, height as i32);

    // Cursor badge on ruler
    let  	badgeText = format!( "#{}", cursorTime);
    dc.set_brush( Colour::rgb( 243, 139, 168), BrushStyle::Solid);
    dc.draw_rectangle( ( cursorX - 18.0) as i32, 2, 36, ( RULER_HEIGHT - 4.0) as i32);
    dc.set_text_foreground( Colour::rgb( 17, 17, 27));
    dc.draw_text( &badgeText, ( cursorX - 14.0) as i32, 6);
}

// ---------------------------------------------------------------------------------------------------------------------------------
