//-- _tests.rs ------------------------------------------------------------------------------------------------------------------------
use	crate::fresco::exprrepos::ExprRepos;
use	crate::fresco::varexpr::{ VarKind, VarExpr };
use	crate::fresco::term::Term;
use	crate::silo::U32;
use	crate::segue::{ JsonOutStream, XField };

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

#[test]
fn	TestTermTree()
{
    let  	x = 'x';
    let  	y = 'y';
    let  	z = "z";
    let  	nodeTree = crate::TermTree!(  x + y *( z + "a" +"b" +"x" +"d")); 
    
    let  	mut output = String::new();
    {
        let  	mut jsonStream = JsonOutStream::New( &mut output, true);
        jsonStream.KeyField( "nodeTree", XField::Fluxable( &nodeTree));
    }
    println!( "{}", output);
    
    let         mut exprRepos = ExprRepos::NewEmpty();
    exprRepos.PostTermTree( &nodeTree);

    let  	mut repoOutput = String::new();
    {
        let  	mut jsonStream = JsonOutStream::New( &mut repoOutput, true);
        jsonStream.KeyField( "exprRepos", XField::Fluxable( &exprRepos));
    }
    println!( "{}", repoOutput);
}

//---------------------------------------------------------------------------------------------------------------------------------
