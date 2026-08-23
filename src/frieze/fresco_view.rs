//-- frieze/fresco_view.rs ----------------------------------------------------------------------------------------------------------
//! Viewport for inspecting Fresco symbolic AST expressions (static informational panel).
use wxdragon::prelude::*;
use wxdragon::color::Colour;

use crate::fresco::ExprRepos;

/// Builds a static panel describing the Fresco symbolic expression repository.
pub fn build_fresco_view_panel(parent: &Notebook, uri: &str) -> Panel {
    let panel = Panel::builder(parent).build();
    panel.set_background_color(Colour::rgb(24, 24, 37));

    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let header = StaticText::builder(&panel)
        .with_label("Æ’ FRESCO â€” Symbolic Expression Repository")
        .build();
    header.set_foreground_color(Colour::rgb(137, 220, 235));
    sizer.add(&header, 0, SizerFlag::All, 10);

    let uri_label = StaticText::builder(&panel).with_label(uri).build();
    uri_label.set_foreground_color(Colour::rgb(108, 112, 134));
    sizer.add(&uri_label, 0, SizerFlag::Left | SizerFlag::Bottom, 10);

    // Touching ExprRepos::New() keeps parity with the egui panel (symbolic term evaluation
    // happens in Rust memory and can be sampled on GPU/SIMD compute kernels).
    let _repos = ExprRepos::New();

    let expressions = [
        ("Polynomial AST Term 1", "P1(x) = 3x^3 - 5x^2 + 2x - 7", "Degree: 3 | Monomials: 4"),
        ("Polynomial AST Term 2", "P2(x, y) = x^2 + 2xy + y^2", "Binomial Expansion | Homogeneous"),
        ("Rational Term Tree", "R(x) = (x^2 - 1) / (x + 1)", "Roots: +/-1 | Simplifiable to x - 1"),
        ("Wavefront Surface", "Z(u, v) = sin(u) * cos(v)", "Bivariate Sampling on GPU"),
    ];

    for (title, formula, meta) in expressions {
        let card = Panel::builder(&panel).build();
        card.set_background_color(Colour::rgb(30, 30, 46));
        let card_sizer = BoxSizer::builder(Orientation::Vertical).build();

        let title_label = StaticText::builder(&card).with_label(title).build();
        title_label.set_foreground_color(Colour::rgb(205, 214, 244));
        card_sizer.add(&title_label, 0, SizerFlag::All, 6);

        let formula_label = StaticText::builder(&card).with_label(formula).build();
        formula_label.set_foreground_color(Colour::rgb(137, 220, 235));
        card_sizer.add(&formula_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom, 6);

        let meta_label = StaticText::builder(&card).with_label(meta).build();
        meta_label.set_foreground_color(Colour::rgb(108, 112, 134));
        card_sizer.add(&meta_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom, 6);

        card.set_sizer(card_sizer, true);
        sizer.add(&card, 0, SizerFlag::Expand | SizerFlag::All, 8);
    }

    panel.set_sizer(sizer, true);
    panel
}
