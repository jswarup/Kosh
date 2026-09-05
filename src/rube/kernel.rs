use	crate::rube::{
    interface::ParameterInterface,
    reg::Reg,
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct KernelSignature
{
    pub _InputPorts:  usize,
    pub _OutputPorts: usize,
    pub _Parameters:  &'static [ParameterInterface],
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug, PartialEq, Eq)]
pub enum KernelError
{
    InputPortMismatch,
    OutputPortMismatch,
    ExecutionFailed( String),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IKernel: Send + Sync
{
    fn	Name( &self) -> &'static str;
    fn	Version( &self) -> &'static str;
    fn	Signature( &self) -> &'static KernelSignature;

    fn	Execute( &self, inputs: &[Reg], outputs: &mut [Reg]) -> Result< (), KernelError>;

    fn	ValidateSignature( &self, inputs: &[Reg], outputs: &mut [Reg]) -> Result< (), KernelError>
    {
        if inputs.len() != self.Signature()._InputPorts {
            return Err( KernelError::InputPortMismatch);
        }
        if outputs.len() != self.Signature()._OutputPorts {
            return Err( KernelError::OutputPortMismatch);
        }
        Ok( ())
    }
}
