//-- wxfrieze/img_view.rs -----------------------------------------------------------------------------------------------------------------

//! Native viewport for rendering raster images (JPEG/PNG).
use	std::path::{Path, PathBuf};

use	wxdragon::bitmap::Bitmap;
use	wxdragon::color::Colour;
use	wxdragon::dc::auto_buffered_paint_dc::AutoBufferedPaintDC;
use	wxdragon::dc::DeviceContext;
use	wxdragon::prelude::*;
use	wxdragon::window::BackgroundStyle;

use	crate::wxfrieze::state::SharedState;

/// Builds a native Image viewport panel for path.
pub fn	build_img_view_panel( parent: &Notebook, _state: SharedState, path: PathBuf) -> Panel
{
    let  	panel = Panel::builder( parent).build();
    let  	rootSizer = BoxSizer::builder( Orientation::Vertical).build();

    let  	toolbar = Panel::builder( &panel).build();
    let  	toolbarSizer = BoxSizer::builder( Orientation::Horizontal).build();
    let  	titleLabel = StaticText::builder( &toolbar)
        .with_label( &format!(
            "IMAGE 2D 📷 {}",
            path.file_name().and_then( |n| n.to_str()).unwrap_or( "image.png")
        ))
        .build();

    toolbarSizer.add( &titleLabel, 1, SizerFlag::AlignCenterVertical | SizerFlag::All, 6);
    toolbar.set_sizer( toolbarSizer, true);

    let  	canvas = Panel::builder( &panel).build();
    canvas.set_background_style( BackgroundStyle::Paint);
    canvas.set_background_color( Colour::rgb( 30, 30, 46));
    canvas.on_erase_background( |_| {});

    let  	bmp = load_image_as_bitmap( &path);

    {
        let  	canvas = canvas.clone();
        canvas.on_paint( move |_evt| {
            let  	dc = AutoBufferedPaintDC::new( &canvas);
            let  	size = canvas.get_client_size();

            dc.clear();

            if let Some( bitmap) = &bmp {
                let  	x = ( size.width - bitmap.get_width()) / 2;
                let  	y = ( size.height - bitmap.get_height()) / 2;
                dc.draw_bitmap( bitmap, x, y, false);
            } else {
                dc.set_text_foreground( Colour::rgb( 235, 100, 100));
                dc.draw_text( "Failed to load or parse image.", 20, 20);
            }
        });
    }

    rootSizer.add( &toolbar, 0, SizerFlag::Expand, 0);
    rootSizer.add( &canvas, 1, SizerFlag::Expand, 0);
    panel.set_sizer( rootSizer, true);

    return panel;
}

fn	load_image_as_bitmap( path: &Path) -> Option< Bitmap>
{
    let  	img = image::open( path).ok()?.into_rgba8();
    let  	width = img.width();
    let  	height = img.height();

    return Bitmap::from_rgba( &img.into_raw(), width, height);
}
