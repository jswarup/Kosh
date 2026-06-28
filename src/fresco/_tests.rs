//-- _tests.rs ------------------------------------------------------------------------------------------------------------------------
use	crate::fresco::exprrepos::ExprRepos;
use	crate::fresco::varexpr::{ VarKind, VarExpr };
use	crate::silo::U32;

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestExprRepos()
{
    let  	mut repos = ExprRepos::NewEmpty();
    
    assert_eq!( repos.SzVar(), U32( 0));
    assert_eq!( repos.Size(), U32( 0));

    let  	tag = repos.VarCreate( "TestVar".to_string(), false);
    let     var = repos.At::< VarExpr>( tag);
    assert_eq!( repos.SzVar(), U32( 1));
    assert_eq!( repos.Size(), U32( 1));
    assert_eq!( repos.VarNameAt( var.VarIndex()), "TestVar");

    let  	attr = repos.VarAttrAt( U32( 0));
    assert_eq!( attr.HasBits( VarKind::Scalar), false);
}

//---------------------------------------------------------------------------------------------------------------------------------
