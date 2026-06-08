//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{ heist::atelier::Atelier, segue::shard::Shard, silo::uint::U32 };

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestShardBuds()
{
    let  	aShard = Shard::New( |_m| {
        print!( "{} ", 10);
    });
    let  	bShard = Shard::New( |_m| {
        print!( "{} ", 20);
    });
    let  	cShard = Shard::New( |_m| {
        print!( "{} ", 40);
    });
    let  	budTree = crate::SegueTree!(
        ( cShard < ( bShard | aShard | ( |_m| {
            print!( "{} ", 50);
        }) ))
    );
    budTree.Print();
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();
    budTree.Post( &mainMaestro);
    drop( mainMaestro);
    atelier.DoLaunch();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestShardFromCharAndString()
{
    let  	budTree = crate::SegueTree!( ( !"cShard" < !( 'b' | "aShard"[ |_m| {
                        print!( "{} ", 50);
                    }] )));
    budTree.Print();
    let  	atelier = Atelier::New( U32( 4));
    let  	mainMaestro = atelier.MainMaestro();
    budTree.Post( &mainMaestro);
    drop( mainMaestro);
    atelier.DoLaunch();
}

//---------------------------------------------------------------------------------------------------------------------------------
