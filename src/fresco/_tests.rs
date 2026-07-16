//-- _tests.rs ------------------------------------------------------------------------------------------------------------------------
use	crate::fresco::exprrepos::ExprRepos;
use	crate::flux::{ IFluxExportSource };
use	crate::fresco::varexpr::{ VarKind, VarExpr };
use	crate::fresco::termtree::ITermNode;
use	crate::silo::U32;
use	crate::flux::{ JsonOutStream, FieldExp };

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

fn	TestTermTreeHelper( ) -> impl ITermNode + IFluxExportSource
{
    crate::TermTree!(  "a" +"b" +"x" +"d")
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestTermTree()
{
    let  	q = TestTermTreeHelper();

    let  	x = 'x';
    let  	y = 'y';
    let  	z = "z";
    let  	nodeTree = crate::TermTree!( q + x + y *( z + "a" +"b" +"x" +"d"));

    let  	nodeTree = crate::TermTree!( q + x + y *( z + "a" +"b" +"x" +"d"));
    let  	mut output = String::new();
    {
        let  	mut jsonStream = JsonOutStream::New( &mut output, true);
        jsonStream.KeyField( "nodeTree", FieldExp::FluxSource( &nodeTree));
    }
    println!( "{}", output);

    let         mut exprRepos = ExprRepos::NewEmpty();
    exprRepos.PostTermTree( &nodeTree);

    let  	mut repoOutput = String::new();
    {
        let  	mut jsonStream = JsonOutStream::New( &mut repoOutput, true);
        jsonStream.KeyField( "exprRepos", FieldExp::FluxSource( &exprRepos));
    }
    println!( "{}", repoOutput);
}

//---------------------------------------------------------------------------------------------------------------------------------
