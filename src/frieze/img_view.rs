//-- frieze/img_view.rs -----------------------------------------------------------------------------------------------------------------

//! Native viewport for rendering raster images (JPEG/PNG).
use std::path::{Path, PathBuf};
use std::cell::RefCell;
use std::rc::Rc;

use wxdragon::bitmap::Bitmap;
use wxdragon::color::Colour;
use wxdragon::dc::auto_buffered_paint_dc::AutoBufferedPaintDC;
use wxdragon::dc::DeviceContext;
use wxdragon::prelude::*;
use wxdragon::window::BackgroundStyle;

use crate::frieze::state::SharedState;

pub fn build_img_view_panel(parent: &Notebook, _state: SharedState, path: PathBuf) -> Panel {
    let panel = Panel::builder(parent).build();
    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();

    let toolbar = Panel::builder(&panel).build();
    let toolbar_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let title_label = StaticText::builder(&toolbar)
        .with_label(&format!(
            "IMAGE 2D - {}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("image.png")
        ))
        .build();

    toolbar_sizer.add(&title_label, 1, SizerFlag::AlignCenterVertical | SizerFlag::All, 6);
    toolbar.set_sizer(toolbar_sizer, true);

    let canvas = Panel::builder(&panel).build();
    canvas.set_background_style(BackgroundStyle::Paint);
    canvas.set_background_color(Colour::rgb(30, 30, 46));
    canvas.on_erase_background(|_| {});

    let raw_img = load_image_as_rgba(&path);
    let cached_bmp: Rc<RefCell<Option<Bitmap>>> = Rc::new(RefCell::new(None));
    let cached_size: Rc<RefCell<(i32, i32)>> = Rc::new(RefCell::new((0, 0)));

    {
        let canvas = canvas.clone();
        canvas.on_size(move |evt| {
            canvas.refresh(true, None);
            evt.skip(true);
        });
    }

    {
        let canvas = canvas.clone();
        let raw_img = raw_img.clone();
        let cached_bmp = cached_bmp.clone();
        let cached_size = cached_size.clone();

        canvas.on_paint(move |_evt| {
            let dc = AutoBufferedPaintDC::new(&canvas);
            let size = canvas.get_client_size();
            dc.clear();

            if let Some(img) = &raw_img {
                let mut current_size = cached_size.borrow_mut();
                if current_size.0 != size.width || current_size.1 != size.height {
                    let img_w = img.width() as f32;
                    let img_h = img.height() as f32;
                    let canvas_w = size.width as f32;
                    let canvas_h = size.height as f32;

                    let scale = (canvas_w / img_w).min(canvas_h / img_h); // allow upscale
                    let new_w = (img_w * scale) as u32;
                    let new_h = (img_h * scale) as u32;

                    if new_w > 0 && new_h > 0 {
                        let scaled = image::imageops::resize(img, new_w, new_h, image::imageops::FilterType::Nearest);
                        *cached_bmp.borrow_mut() = Bitmap::from_rgba(&scaled.into_raw(), new_w, new_h);
                    }
                    *current_size = (size.width, size.height);
                }

                if let Some(bmp) = cached_bmp.borrow().as_ref() {
                    let x = (size.width - bmp.get_width()) / 2;
                    let y = (size.height - bmp.get_height()) / 2;
                    dc.draw_bitmap(bmp, x, y, false);
                }
            } else {
                dc.set_text_foreground(Colour::rgb(235, 100, 100));
                dc.draw_text("Failed to load or parse image.", 20, 20);
            }
        });
    }

    root_sizer.add(&toolbar, 0, SizerFlag::Expand, 0);
    root_sizer.add(&canvas, 1, SizerFlag::Expand, 0);
    panel.set_sizer(root_sizer, true);

    panel
}

fn load_image_as_rgba(path: &Path) -> Option<image::RgbaImage> {
    image::open(path).ok()?.into_rgba8().into()
}