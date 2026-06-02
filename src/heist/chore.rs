//-- chore.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::silo::uint::U32;
use	crate::stalks::work::{ IWorker, IWork };
use	crate::stalks::bud::{ Bud, IntoBud, TraversalEvent };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Chore
{
    pub Ind: U32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Chore
{
    pub fn	New( ind: U32) -> Self
    {
        Self {
            Ind: ind,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IWork for Chore
{
    fn	DoWork( &mut self, _worker: &dyn IWorker)
    {
        println!( "{}", self.Ind);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Bud< Chore> for Chore
{
    fn	Val( &self) -> Chore
    {
        *self
    }

    fn	Left( &self) -> Option< &dyn Bud< Chore>>
    {
        None
    }

    fn	Right( &self) -> Option< &dyn Bud< Chore>>
    {
        None
    }

    fn	Op( &self) -> &str
    {
        ""
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IntoBud< Chore> for Chore
{
    fn	IntoBud( self) -> Box< dyn Bud< Chore>>
    {
        Box::new( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::fmt::Display for Chore
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        write!( f, "{}", self.Ind)
    }
}


//---------------------------------------------------------------------------------------------------------------------------------

impl dyn Bud<Chore>+ '_
{
    pub fn	Post( &self, worker:  &dyn IWorker )
    {
        let  	mut childCounts = Vec::new();
        self.TraverseDFS( &mut |node, event| {
            match event {
                TraversalEvent::Entry => {
                    if let Some( count) = childCounts.last_mut() {
                        if *count > 0 {
                            print!( " ");
                        }
                        *count += 1;
                    }
                    if node.Left().is_some() || node.Right().is_some() {
                        print!( "[{} ", node.Op());
                        childCounts.push( 0);
                    } else {
                        print!( "{}", node.Val());
                    }
                }
                TraversalEvent::Exit => {
                    childCounts.pop();
                    print!( "]");
                }
            }
        });
        println!();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------


