//-- _tests.rs ------------------------------------------------------------------------------------------------------------------------
use	crate::fresco::exprrepos::{ ExprRepos, VarKind, VarExpr };
use	crate::silo::U32;

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestExprRepos()
{
    let  	mut repos = ExprRepos::NewEmpty();
    
    assert_eq!( repos.SzVar(), U32( 0));
    assert_eq!( repos.Size(), U32( 0));

    let  	tag = repos.VarCreate( "TestVar".to_string(), false);
    let     _var = repos.At::< VarExpr>( tag);
    assert_eq!( repos.SzVar(), U32( 1));
    assert_eq!( repos.Size(), U32( 1));
    assert_eq!( repos.VarNameAt( U32( 0)), "TestVar");

    let  	attr = repos.VarAttrAt( U32( 0));
    assert_eq!( attr.HasBits( VarKind::Scalar), false);
}

//---------------------------------------------------------------------------------------------------------------------------------
