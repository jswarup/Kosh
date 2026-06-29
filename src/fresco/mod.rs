//-- fresco/mod.rs ---------------------------------------------------------------------------------------------------------------------
pub mod exprrepos;
pub mod varexpr;
pub mod realexpr;
pub mod polyexpr;
pub mod sumexpr;
pub mod prodexpr;
pub mod term;

pub use	exprrepos::{ BaseExpr, ExprEntry, ExprRepos };
pub use	varexpr::{ VarAttrib, VarExpr, VarKind };
pub use	realexpr::RealExpr;
pub use	polyexpr::PolyExpr;
pub use	sumexpr::SumExpr;
pub use	prodexpr::ProdExpr;
pub use	term::Term;

//---------------------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
pub mod _tests;

//---------------------------------------------------------------------------------------------------------------------------------
