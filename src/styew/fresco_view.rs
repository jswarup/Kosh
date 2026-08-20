//-- styew/fresco_view.rs -----------------------------------------------------------------------------------------------------------
use	egui::{ Ui, Color32, RichText, Frame, Stroke, Margin, Vec2 };
use	crate::fresco::ExprRepos;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Viewport for inspecting Fresco symbolic AST expressions matching Aura's layout.
pub fn	RenderFrescoView( ui: &mut Ui, uri: &str)
{
    ui.vertical( |ui| {
        // Top Header
        ui.horizontal( |ui| {
            ui.spacing_mut().item_spacing = Vec2::new( 8.0, 0.0);
            ui.label( RichText::new( "ƒ FRESCO").color( Color32::from_rgb( 137, 220, 235)).strong().size( 10.5));
            ui.label( RichText::new( "Symbolic Expression Repository").strong().size( 13.0).color( Color32::from_rgb( 205, 214, 244)));
            ui.label( RichText::new( uri).size( 11.5).color( Color32::from_rgb( 108, 112, 134)));
        });

        ui.add_space( 2.0);
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink( [false, false])
            .show( ui, |ui| {
                ui.add_space( 6.0);

                Frame::new()
                    .fill( Color32::from_rgb( 24, 24, 37))
                    .stroke( Stroke::new( 1.0, Color32::from_rgb( 49, 50, 68)))
                    .corner_radius( 6)
                    .inner_margin( Margin::same( 14))
                    .show( ui, |ui| {
                        ui.label( RichText::new( "Algebraic Term Structure & AST Representation").strong().size( 13.0).color( Color32::from_rgb( 137, 180, 250)));
                        ui.add_space( 4.0);
                        ui.label( RichText::new( "Fresco AST terms (PolyExpr, VarExpr, RealExpr, TermTree) are evaluated symbolically in Rust memory and can be sampled on GPU/SIMD compute kernels.").size( 12.0).color( Color32::from_rgb( 166, 173, 200)));
                    });

                ui.add_space( 12.0);

                let  	_repos = ExprRepos::New();
                let  	expressions = [
                    ("Polynomial AST Term 1", "P₁(x) = 3x³ - 5x² + 2x - 7", "Degree: 3 | Monomials: 4"),
                    ("Polynomial AST Term 2", "P₂(x, y) = x² + 2xy + y²", "Binomial Expansion | Homogeneous"),
                    ("Rational Term Tree",   "R(x) = (x² - 1) / (x + 1)", "Roots: ±1 | Simplifiable to x - 1"),
                    ("Wavefront Surface",     "Z(u, v) = sin(u) · cos(v)", "Bivariate Sampling on GPU"),
                ];

                for (title, formula, meta) in expressions {
                    Frame::new()
                        .fill( Color32::from_rgb( 30, 30, 46))
                        .stroke( Stroke::new( 1.0, Color32::from_rgba_premultiplied( 137, 180, 250, 35)))
                        .corner_radius( 6)
                        .inner_margin( Margin::same( 12))
                        .show( ui, |ui| {
                            ui.horizontal( |ui| {
                                ui.label( RichText::new( title).strong().size( 12.5).color( Color32::from_rgb( 205, 214, 244)));
                                ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                                    ui.label( RichText::new( meta).size( 10.5).color( Color32::from_rgb( 108, 112, 134)));
                                });
                            });

                            ui.add_space( 4.0);
                            ui.label( RichText::new( formula).monospace().size( 12.5).color( Color32::from_rgb( 137, 220, 235)));
                        });

                    ui.add_space( 8.0);
                }
            });
    });
}

// ---------------------------------------------------------------------------------------------------------------------------------
