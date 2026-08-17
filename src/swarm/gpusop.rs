//-- swarm/gpusop.rs -----------------------------------------------------------------------------------------------------------------

use	std::sync::mpsc;
use	wgpu::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Device,
    DeviceDescriptor, Instance, MapMode, PollType, PowerPreference, Queue,
    RequestAdapterOptions,
};
use	wgpu::util::{ BufferInitDescriptor, DeviceExt };
use	crate::silo::Buff;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGpuOp
{
    fn	Init() -> Option< ( Device, Queue)>;

    fn	BufferInit(
        &self,
        label: &str,
        data: &[u8],
        usage: BufferUsages,
    ) -> Buffer;

    fn	ReadBuffer(
        &self,
        queue: &Queue,
        buf: &Buffer,
        size: u64,
    ) -> Buff< u8>;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGpuOp for Device
{
    fn	Init() -> Option< ( Device, Queue)>
    {
        pollster::block_on( async {
            let  	instance = Instance::default();
            let  	adapter = instance
                .request_adapter( &RequestAdapterOptions {
                    power_preference: PowerPreference::HighPerformance,
                    ..Default::default()
                })
                .await;
            let  	adapter = match adapter {
                Ok( a) => a,
                Err( _) => {
                    return None;
                }
            };
            let  	( device, queue) = adapter
                .request_device( &DeviceDescriptor {
                    label: Some( "KoshGpu"),
                    ..Default::default()
                })
                .await
                .expect( "Failed to create GPU device");
            Some( ( device, queue))
        })
    }

    fn	BufferInit(
        &self,
        label: &str,
        data: &[u8],
        usage: BufferUsages,
    ) -> Buffer
    {
        self.create_buffer_init( &BufferInitDescriptor {
            label: Some( label),
            contents: data,
            usage,
        })
    }

    fn	ReadBuffer(
        &self,
        queue: &Queue,
        buf: &Buffer,
        size: u64,
    ) -> Buff< u8>
    {
        let  	staging = self.create_buffer( &BufferDescriptor {
            label: Some( "staging"),
            size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let  	mut encoder = self.create_command_encoder( &CommandEncoderDescriptor {
            label: Some( "readback"),
        });
        encoder.copy_buffer_to_buffer( buf, 0, &staging, 0, size);
        queue.submit( std::iter::once( encoder.finish()));

        let  	slice = staging.slice( ..);
        let  	( tx, rx) = mpsc::channel();
        slice.map_async( MapMode::Read, move |result| {
            tx.send( result).unwrap();
        });
        self.poll( PollType::Wait { submission_index: None, timeout: None }).unwrap();
        rx.recv().unwrap().unwrap();

        let  	view = slice.get_mapped_range();
        let  	result = Buff::from( &*view);
        drop( view);
        staging.unmap();
        result
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
