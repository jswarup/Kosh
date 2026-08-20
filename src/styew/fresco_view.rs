//-- styew/fresco_view.rs -----------------------------------------------------------------------------------------------------------
use	egui::{ Ui, Color32, RichText, Frame, Stroke, Margin };
use	crate::fresco::ExprRepos;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Viewport for inspecting Fresco symbolic AST expressions and function graphs.
pub fn	RenderFrescoView( ui: &mut Ui, uri: &str)
{
    ui.vertical( |ui| {
        // Header
        ui.horizontal( |ui| {
            ui.label( RichText::new( "ƒ FRESCO SYMBOLIC ENGINE").strong().size( 14.0).color( Color32::from_rgb( 59, 130, 246)));
            ui.label( RichText::new( uri).size( 11.0).color( Color32::from_rgb( 100, 116, 139)));
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink( [false, false])
            .show( ui, |ui| {
                Frame::new()
                    .fill( Color32::from_rgb( 17, 24, 39))
                    .stroke( Stroke::new( 1.0, Color32::from_rgba_premultiplied( 255, 255, 255, 15)))
                    .corner_radius( 6)
                    .inner_margin( Margin::same( 16))
                    .show( ui, |ui| {
                        ui.label( RichText::new( "Algebraic Term Structure & AST Representation").strong().size( 13.0).color( Color32::from_rgb( 96, 165, 250)));
                        ui.add_space( 6.0);
                        ui.label( RichText::new( "Expressions in Fresco (PolyExpr, VarExpr, RealExpr, TermTree) are evaluated symbolically and can be evaluated on CPU SIMD or Swarm GPU compute kernels.").size( 12.0).color( Color32::from_rgb( 148, 163, 184)));
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
                        .fill( Color32::from_rgba_premultiplied( 15, 23, 42, 200))
                        .stroke( Stroke::new( 1.0, Color32::from_rgba_premultiplied( 59, 130, 246, 50)))
                        .corner_radius( 6)
                        .inner_margin( Margin::same( 12))
                        .show( ui, |ui| {
                            ui.horizontal( |ui| {
                                ui.label( RichText::new( title).strong().size( 12.0).color( Color32::WHITE));
                                ui.with_layout( egui::Layout::right_to_left( egui::Align::Center), |ui| {
                                    ui.label( RichText::new( meta).size( 10.0).color( Color32::from_rgb( 100, 116, 139)));
                                });
                            });

                            ui.add_space( 4.0);
                            ui.label( RichText::new( formula).monospace().size( 13.0).color( Color32::from_rgb( 0, 243, 255)));
                        });

                    ui.add_space( 8.0);
                }
            });
    });
}

// ---------------------------------------------------------------------------------------------------------------------------------
