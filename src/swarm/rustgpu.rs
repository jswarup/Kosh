//-- swarm/rustgpu.rs ----------------------------------------------------------------------------------------------------------------

use	std::sync::mpsc;
use	std::sync::Arc;
use	wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor,
    BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, DeviceDescriptor, Instance, MapMode,
    PipelineLayoutDescriptor, PollType, PowerPreference, Queue, RequestAdapterOptions,
    ShaderModuleDescriptor, ShaderSource as WgpuShaderSource, ShaderStages,
};
use	wgpu::util::{ BufferInitDescriptor, DeviceExt };
use	crate::silo::{ Buff, U64 };
use	crate::swarm::traits::{
    BackendKind, BufferUsage, IComputeBuffer, IComputeDevice, IComputeKernel,
    KernelSource, SwarmError, WorkgroupDim,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// WebGPU storage buffer on GPU hardware.
pub struct RustGpuBuffer
{
    _Label:  String,
    _Device: Arc< Device>,
    _Queue:  Arc< Queue>,
    _Buffer: Arc< Buffer>,
    _Size:   U64,
    _Usage:  BufferUsage,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl RustGpuBuffer
{
    pub fn	New(
        label: &str,
        device: Arc< Device>,
        queue: Arc< Queue>,
        buffer: Buffer,
        size: U64,
        usage: BufferUsage,
    ) -> Self
    {
        RustGpuBuffer {
            _Label: label.to_string(),
            _Device: device,
            _Queue: queue,
            _Buffer: Arc::new( buffer),
            _Size: size,
            _Usage: usage,
        }
    }

    pub fn	Usage( &self) -> BufferUsage
    {
        self._Usage
    }

    pub fn	RawBuffer( &self) -> &Buffer
    {
        &self._Buffer
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IComputeBuffer for RustGpuBuffer
{
    fn	Size( &self) -> U64
    {
        self._Size
    }

    fn	Label( &self) -> &str
    {
        &self._Label
    }

    fn	Write( &mut self, data: &[u8]) -> Result< (), SwarmError>
    {
        self._Queue.write_buffer( &self._Buffer, 0, data);
        Ok( ())
    }

    fn	Read( &self) -> Result< Buff< u8>, SwarmError>
    {
        let  	staging = self._Device.create_buffer( &BufferDescriptor {
            label: Some( "staging_read"),
            size: self._Size.AsUsize() as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let  	mut encoder = self._Device.create_command_encoder( &CommandEncoderDescriptor {
            label: Some( "readback_encoder"),
        });
        encoder.copy_buffer_to_buffer( &self._Buffer, 0, &staging, 0, self._Size.AsUsize() as u64);
        self._Queue.submit( std::iter::once( encoder.finish()));

        let  	slice = staging.slice( ..);
        let  	( tx, rx) = mpsc::channel();
        slice.map_async( MapMode::Read, move |result| {
            tx.send( result).unwrap();
        });
        self._Device.poll( PollType::Wait { submission_index: None, timeout: None }).map_err( |e| {
            SwarmError::BufferError( format!( "Device poll failed: {:?}", e))
        })?;
        rx.recv().map_err( |e| {
            SwarmError::BufferError( format!( "Channel receive failed: {}", e))
        })?.map_err( |e| {
            SwarmError::BufferError( format!( "Buffer mapping error: {:?}", e))
        })?;

        let  	view = slice.get_mapped_range();
        let  	result = Buff::from( &view.as_ref().unwrap()[..]);
        drop( view);
        staging.unmap();

        Ok( result)
    }

    fn	AsAny( &self) -> &dyn std::any::Any
    {
        self
    }

    fn	AsAnyMut( &mut self) -> &mut dyn std::any::Any
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// WebGPU compute pipeline compiled from WGSL or Rust-GPU SPIR-V bytecode.
pub struct RustGpuKernel
{
    _Name:               String,
    _EntryPoint:         String,
    _Pipeline:           Arc< ComputePipeline>,
    _BindGroupLayout:    Arc< BindGroupLayout>,
}

impl RustGpuKernel
{
    pub fn	New(
        name: &str,
        entryPoint: &str,
        pipeline: ComputePipeline,
        bindGroupLayout: BindGroupLayout,
    ) -> Self
    {
        RustGpuKernel {
            _Name: name.to_string(),
            _EntryPoint: entryPoint.to_string(),
            _Pipeline: Arc::new( pipeline),
            _BindGroupLayout: Arc::new( bindGroupLayout),
        }
    }

    pub fn	FromCached(
        name: &str,
        entryPoint: &str,
        pipeline: Arc< ComputePipeline>,
        bindGroupLayout: Arc< BindGroupLayout>,
    ) -> Self
    {
        RustGpuKernel {
            _Name: name.to_string(),
            _EntryPoint: entryPoint.to_string(),
            _Pipeline: pipeline,
            _BindGroupLayout: bindGroupLayout,
        }
    }

    pub fn	EntryPoint( &self) -> &str
    {
        &self._EntryPoint
    }

    pub fn	Pipeline( &self) -> &ComputePipeline
    {
        &self._Pipeline
    }

    pub fn	BindGroupLayout( &self) -> &BindGroupLayout
    {
        &self._BindGroupLayout
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IComputeKernel for RustGpuKernel
{
    fn	Name( &self) -> &str
    {
        &self._Name
    }

    fn	Backend( &self) -> BackendKind
    {
        BackendKind::RustGpu
    }

    fn	AsAny( &self) -> &dyn std::any::Any
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Rust-GPU compute device wrapping `wgpu` Device & Queue with compiled pipeline caching.
#[derive( Clone)]
pub struct RustGpuDevice
{
    _Device: Arc< Device>,
    _Queue:  Arc< Queue>,
    _Cache:  Arc< std::sync::Mutex< std::collections::HashMap< String, ( Arc< ComputePipeline>, Arc< BindGroupLayout>)>>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl RustGpuDevice
{
    pub fn	Init() -> Result< Self, SwarmError>
    {
        let  	instance = Instance::default();
        let  	adapter = pollster::block_on( async {
            instance.request_adapter( &RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                ..Default::default()
            }).await
        });
        let  	adapter = match adapter {
            Ok( a) => a,
            Err( e) => {
                return Err( SwarmError::DeviceUnavailable(
                    format!( "Failed to acquire GPU adapter: {:?}", e),
                ));
            }
        };

        let  	( device, queue) = pollster::block_on( async {
            adapter.request_device( &DeviceDescriptor {
                label: Some( "KoshRustGpuDevice"),
                ..Default::default()
            }).await
        }).map_err( |e| {
            SwarmError::DeviceUnavailable( format!( "Failed to create GPU device: {:?}", e))
        })?;

        Ok( RustGpuDevice {
            _Device: Arc::new( device),
            _Queue:  Arc::new( queue),
            _Cache:  Arc::new( std::sync::Mutex::new( std::collections::HashMap::new())),
        })
    }

    pub fn	EnumerateDevices() -> Buff< Self>
    {
        let  	instance = Instance::default();
        let  	adapters = pollster::block_on( instance.enumerate_adapters( wgpu::Backends::all()));
        let  	mut devicesVec = Vec::with_capacity( adapters.len().max( 1));

        for adapter in adapters {
            let  	res = pollster::block_on( async {
                adapter.request_device( &DeviceDescriptor {
                    label: Some( "KoshRustGpuDevice"),
                    ..Default::default()
                }).await
            });
            if let Ok( ( device, queue)) = res {
                devicesVec.push( RustGpuDevice {
                    _Device: Arc::new( device),
                    _Queue:  Arc::new( queue),
                    _Cache:  Arc::new( std::sync::Mutex::new( std::collections::HashMap::new())),
                });
            }
        }

        Buff::from_iter( devicesVec)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    
    pub fn	FromShared( device: Arc< Device>, queue: Arc< Queue>) -> Self
    {
        RustGpuDevice {
            _Device: device,
            _Queue:  queue,
            _Cache:  Arc::new( std::sync::Mutex::new( std::collections::HashMap::new())),
        }
    }

    pub fn	WgpuDevice( &self) -> &Arc< Device>
    {
        &self._Device
    }

    pub fn	WgpuQueue( &self) -> &Arc< Queue>
    {
        &self._Queue
    }

    pub fn	FromDeviceQueue( device: Device, queue: Queue) -> Self
    {
        RustGpuDevice {
            _Device: Arc::new( device),
            _Queue:  Arc::new( queue),
            _Cache:  Arc::new( std::sync::Mutex::new( std::collections::HashMap::new())),
        }
    }

    pub fn	RawDevice( &self) -> &Device
    {
        &self._Device
    }

    pub fn	RawQueue( &self) -> &Queue
    {
        &self._Queue
    }

    fn	ToWgpuUsage( usage: BufferUsage) -> BufferUsages
    {
        let  	mut u = BufferUsages::empty();
        if usage.Contains( BufferUsage::STORAGE) {
            u |= BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;
        }
        if usage.Contains( BufferUsage::UNIFORM) {
            u |= BufferUsages::UNIFORM | BufferUsages::COPY_DST;
        }
        if usage.Contains( BufferUsage::COPY_SRC) {
            u |= BufferUsages::COPY_SRC;
        }
        if usage.Contains( BufferUsage::COPY_DST) {
            u |= BufferUsages::COPY_DST;
        }
        if u.is_empty() {
            u = BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;
        }
        u
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IComputeDevice for RustGpuDevice
{
    fn	Backend( &self) -> BackendKind
    {
        BackendKind::RustGpu
    }

    fn	CreateBuffer(
        &self,
        label: &str,
        size: U64,
        usage: BufferUsage,
    ) -> Result< Box< dyn IComputeBuffer>, SwarmError>
    {
        let  	wgpuUsage = Self::ToWgpuUsage( usage);
        let  	buf = self._Device.create_buffer( &BufferDescriptor {
            label: Some( label),
            size: size.AsUsize() as u64,
            usage: wgpuUsage,
            mapped_at_creation: false,
        });

        Ok( Box::new( RustGpuBuffer::New(
            label,
            Arc::clone( &self._Device),
            Arc::clone( &self._Queue),
            buf,
            size,
            usage,
        )))
    }

    fn	CreateBufferInit(
        &self,
        label: &str,
        data: &[u8],
        usage: BufferUsage,
    ) -> Result< Box< dyn IComputeBuffer>, SwarmError>
    {
        let  	wgpuUsage = Self::ToWgpuUsage( usage);
        let  	buf = self._Device.create_buffer_init( &BufferInitDescriptor {
            label: Some( label),
            contents: data,
            usage: wgpuUsage,
        });

        Ok( Box::new( RustGpuBuffer::New(
            label,
            Arc::clone( &self._Device),
            Arc::clone( &self._Queue),
            buf,
            U64( data.len() as u64),
            usage,
        )))
    }

    fn	CompileKernel(
        &self,
        label: &str,
        entryPoint: &str,
        source: KernelSource,
    ) -> Result< Box< dyn IComputeKernel>, SwarmError>
    {
        let  	cacheKey = format!( "{}:{}", label, entryPoint);
        if let Ok( guard) = self._Cache.lock() {
            if let Some( ( cachedPipe, cachedBgl)) = guard.get( &cacheKey) {
                return Ok( Box::new( RustGpuKernel::FromCached(
                    label,
                    entryPoint,
                    Arc::clone( cachedPipe),
                    Arc::clone( cachedBgl),
                )));
            }
        }

        let  	shaderModule = match source {
            KernelSource::Wgsl( src) => {
                self._Device.create_shader_module( ShaderModuleDescriptor {
                    label: Some( label),
                    source: WgpuShaderSource::Wgsl( src.into()),
                })
            }
            KernelSource::SpirV( bytes) => {
                let  	spirv = std::borrow::Cow::Owned(
                    wgpu::util::make_spirv_raw( bytes).into_owned(),
                );
                self._Device.create_shader_module( ShaderModuleDescriptor {
                    label: Some( label),
                    source: WgpuShaderSource::SpirV( spirv),
                })
            }
            KernelSource::Ptx( _) | KernelSource::CpuClosure( _) => {
                return Err( SwarmError::UnsupportedBackend( BackendKind::RustGpu));
            }
        };

        // Determine number of buffer bindings based on kernel label/entryPoint
        let  	bindingCount = if label.contains( "vecadd") || label.contains( "camera_transform") || entryPoint.contains( "camera_transform") {
            3
        } else if label.contains( "collatz") {
            2
        } else {
            1
        };

        let  	mut entries: Vec< BindGroupLayoutEntry> = Vec::with_capacity( bindingCount);
        for i in 0..bindingCount {
            let  	readOnly = i < bindingCount - 1 && bindingCount > 1;
            entries.push( BindGroupLayoutEntry {
                binding: i as u32,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: readOnly },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }

        let  	bindGroupLayout = self._Device.create_bind_group_layout( &BindGroupLayoutDescriptor {
            label: Some( &format!( "{}_bgl", label)),
            entries: &entries,
        });

        let  	pipelineLayout = self._Device.create_pipeline_layout( &PipelineLayoutDescriptor {
            label: Some( &format!( "{}_pl", label)),
            bind_group_layouts: &[Some( &bindGroupLayout)],
            immediate_size: 0,
        });

        let  	pipeline = self._Device.create_compute_pipeline( &ComputePipelineDescriptor {
            label: Some( &format!( "{}_pipe", label)),
            layout: Some( &pipelineLayout),
            module: &shaderModule,
            entry_point: Some( entryPoint),
            compilation_options: Default::default(),
            cache: None,
        });

        let  	arcPipeline = Arc::new( pipeline);
        let  	arcBgl = Arc::new( bindGroupLayout);

        if let Ok( mut guard) = self._Cache.lock() {
            guard.insert( cacheKey, ( Arc::clone( &arcPipeline), Arc::clone( &arcBgl)));
        }

        Ok( Box::new( RustGpuKernel::FromCached(
            label,
            entryPoint,
            arcPipeline,
            arcBgl,
        )))
    }

    fn	Dispatch(
        &self,
        kernel: &dyn IComputeKernel,
        buffers: &[&dyn IComputeBuffer],
        dim: WorkgroupDim,
    ) -> Result< (), SwarmError>
    {
        let  	rustKernel = match kernel.AsAny().downcast_ref::< RustGpuKernel>() {
            Some( k) => k,
            None => {
                return Err( SwarmError::ExecutionError(
                    "Kernel is not a RustGpuKernel".to_string(),
                ));
            }
        };

        let  	mut entries: Vec< BindGroupEntry> = Vec::with_capacity( buffers.len());
        for ( i, b) in buffers.iter().enumerate() {
            if let Some( gpuBuf) = (*b).AsAny().downcast_ref::< RustGpuBuffer>() {
                entries.push( BindGroupEntry {
                    binding: i as u32,
                    resource: gpuBuf.RawBuffer().as_entire_binding(),
                });
            } else {
                return Err( SwarmError::BufferError(
                    format!( "Buffer {} is not a RustGpuBuffer", b.Label()),
                ));
            }
        }

        let  	bindGroup = self._Device.create_bind_group( &BindGroupDescriptor {
            label: Some( &format!( "{}_bg", rustKernel.Name())),
            layout: rustKernel.BindGroupLayout(),
            entries: &entries,
        });

        let  	mut encoder = self._Device.create_command_encoder( &CommandEncoderDescriptor {
            label: Some( "dispatch_encoder"),
        });
        {
            let  	mut pass = encoder.begin_compute_pass( &ComputePassDescriptor {
                label: Some( "compute_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline( rustKernel.Pipeline());
            pass.set_bind_group( 0, &bindGroup, &[]);
            pass.dispatch_workgroups( dim._X.AsU32(), dim._Y.AsU32(), dim._Z.AsU32());
        }
        self._Queue.submit( std::iter::once( encoder.finish()));

        Ok( ())
    }

    fn	Synchronize( &self) -> Result< (), SwarmError>
    {
        self._Device.poll( PollType::Wait { submission_index: None, timeout: None }).map_err( |e| {
            SwarmError::ExecutionError( format!( "Device synchronization failed: {:?}", e))
        })?;
        Ok( ())
    }

    fn	AsAny( &self) -> &dyn std::any::Any
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
