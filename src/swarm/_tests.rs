//-- swarm/_tests.rs -----------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Buff, U32 };

//---------------------------------------------------------------------------------------------------------------------------------

const SHADER_DOUBLE: &str = r#"
@group(0) @binding(0)
var<storage, read_write> data: array<f32>;

@compute @workgroup_size( 64)
fn main( @builtin( global_invocation_id) gid: vec3<u32>)
{
    let  	idx = gid.x;
    if idx < arrayLength( &data) {
        data[idx] = data[idx] * 2.0;
    }
}
"#;

//---------------------------------------------------------------------------------------------------------------------------------

const SHADER_VECTOR_ADD: &str = r#"
@group(0) @binding(0)
var<storage, read> a: array<f32>;

@group(0) @binding(1)
var<storage, read> b: array<f32>;

@group(0) @binding(2)
var<storage, read_write> result: array<f32>;

@compute @workgroup_size( 64)
fn main( @builtin( global_invocation_id) gid: vec3<u32>)
{
    let  	idx = gid.x;
    if idx < arrayLength( &result) {
        result[idx] = a[idx] + b[idx];
    }
}
"#;

//---------------------------------------------------------------------------------------------------------------------------------

const SHADER_COLLATZ: &str = r#"
@group(0) @binding(0)
var<storage, read> input: array<u32>;

@group(0) @binding(1)
var<storage, read_write> output: array<u32>;

fn collatz_steps( n_in: u32) -> u32
{
    var  	n: u32 = n_in;
    var  	steps: u32 = 0u;
    while n != 1u {
        if (n % 2u) == 0u {
            n = n / 2u;
        } else {
            n = 3u * n + 1u;
        }
        steps = steps + 1u;
    }
    return steps;
}

@compute @workgroup_size( 64)
fn main( @builtin( global_invocation_id) gid: vec3<u32>)
{
    let  	idx = gid.x;
    if idx < arrayLength( &output) {
        output[idx] = collatz_steps( input[idx]);
    }
}
"#;

//---------------------------------------------------------------------------------------------------------------------------------

/// Initializes the wgpu device and queue, returning None if no adapter is
/// available (e.g. headless CI without GPU).
fn	GpuInit() -> Option< ( wgpu::Device, wgpu::Queue)>
{
    pollster::block_on( async {
        let  	instance = wgpu::Instance::default();
        let  	adapter = instance
            .request_adapter( &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await;
        let  	adapter = match adapter {
            Ok( a) => a,
            Err( _) => {
                return None;
            }
        };
        let  	(device, queue) = adapter
            .request_device( &wgpu::DeviceDescriptor {
                label: Some( "KoshGpuTest"),
                ..Default::default()
            })
            .await
            .expect( "Failed to create GPU device");
        Some( ( device, queue))
    })
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Helper: create a GPU buffer pre-filled with the given byte data.
fn	GpuBufferInit(
    device: &wgpu::Device,
    label: &str,
    data: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer
{
    use	wgpu::util::DeviceExt;
    device.create_buffer_init( &wgpu::util::BufferInitDescriptor {
        label: Some( label),
        contents: data,
        usage,
    })
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Helper: read back a GPU buffer's contents into a Vec<u8>.
fn	GpuReadBuffer( device: &wgpu::Device, queue: &wgpu::Queue, buf: &wgpu::Buffer, size: u64) -> Vec< u8>
{
    let  	staging = device.create_buffer( &wgpu::BufferDescriptor {
        label: Some( "staging"),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let  	mut encoder = device.create_command_encoder( &wgpu::CommandEncoderDescriptor {
        label: Some( "readback"),
    });
    encoder.copy_buffer_to_buffer( buf, 0, &staging, 0, size);
    queue.submit( std::iter::once( encoder.finish()));

    let  	slice = staging.slice( ..);
    let  	(tx, rx) = std::sync::mpsc::channel();
    slice.map_async( wgpu::MapMode::Read, move |result| {
        tx.send( result).unwrap();
    });
    device.poll( wgpu::PollType::Wait { submission_index: None, timeout: None }).unwrap();
    rx.recv().unwrap().unwrap();

    let  	view = slice.get_mapped_range();
    let  	result = view.to_vec();
    drop( view);
    staging.unmap();
    result
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestGpuDoubleValues()
{
    let  	(device, queue) = match GpuInit() {
        Some( dq) => dq,
        None => {
            println!( "No GPU adapter found — skipping TestGpuDoubleValues");
            return;
        }
    };

    // Prepare input data using project Buff
    let  	szData = U32( 256);
    let  	input = Buff::Create( szData, |i| ( i.AsU32() + 1) as f32);
    let  	byteLen = (szData.AsUsize()) * std::mem::size_of::< f32>();

    // Upload to GPU
    let  	gpuBuf = GpuBufferInit(
        &device,
        "double_data",
        bytemuck_cast_slice::< f32>( &input),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );

    // Create shader + pipeline
    let  	shader = device.create_shader_module( wgpu::ShaderModuleDescriptor {
        label: Some( "double_shader"),
        source: wgpu::ShaderSource::Wgsl( SHADER_DOUBLE.into()),
    });
    let  	bindGroupLayout = device.create_bind_group_layout( &wgpu::BindGroupLayoutDescriptor {
        label: Some( "double_bgl"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let  	pipelineLayout = device.create_pipeline_layout( &wgpu::PipelineLayoutDescriptor {
        label: Some( "double_pl"),
        bind_group_layouts: &[Some( &bindGroupLayout)],
        immediate_size: 0,
    });
    let  	pipeline = device.create_compute_pipeline( &wgpu::ComputePipelineDescriptor {
        label: Some( "double_pipeline"),
        layout: Some( &pipelineLayout),
        module: &shader,
        entry_point: Some( "main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let  	bindGroup = device.create_bind_group( &wgpu::BindGroupDescriptor {
        label: Some( "double_bg"),
        layout: &bindGroupLayout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: gpuBuf.as_entire_binding(),
        }],
    });

    // Dispatch
    let  	workgroups = ( szData.AsU32() + 63) / 64;
    let  	mut encoder = device.create_command_encoder( &wgpu::CommandEncoderDescriptor {
        label: Some( "double_enc"),
    });
    {
        let  	mut pass = encoder.begin_compute_pass( &wgpu::ComputePassDescriptor {
            label: Some( "double_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline( &pipeline);
        pass.set_bind_group( 0, &bindGroup, &[]);
        pass.dispatch_workgroups( workgroups, 1, 1);
    }
    queue.submit( std::iter::once( encoder.finish()));

    // Read back and verify
    let  	raw = GpuReadBuffer( &device, &queue, &gpuBuf, byteLen as u64);
    let  	result: &[f32] = bytemuck_cast_slice_from::< f32>( &raw);
    for i in 0..szData.AsUsize() {
        let  	expected = (( i as u32) + 1) as f32 * 2.0;
        assert!(
            ( result[i] - expected).abs() < 1e-6,
            "Mismatch at {}: got {}, expected {}",
            i, result[i], expected,
        );
    }
    println!( "TestGpuDoubleValues: {} values doubled on GPU ✓", szData);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestGpuVectorAdd()
{
    let  	(device, queue) = match GpuInit() {
        Some( dq) => dq,
        None => {
            println!( "No GPU adapter found — skipping TestGpuVectorAdd");
            return;
        }
    };

    let  	szData = U32( 512);
    let  	buffA = Buff::Create( szData, |i| i.AsU32() as f32);
    let  	buffB = Buff::Create( szData, |i| ( i.AsU32() * 10) as f32);
    let  	buffOut = Buff::New( szData, 0.0f32);
    let  	byteLen = szData.AsUsize() * std::mem::size_of::< f32>();

    let  	gpuA = GpuBufferInit(
        &device, "vecadd_a",
        bytemuck_cast_slice::< f32>( &buffA),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let  	gpuB = GpuBufferInit(
        &device, "vecadd_b",
        bytemuck_cast_slice::< f32>( &buffB),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let  	gpuOut = GpuBufferInit(
        &device, "vecadd_out",
        bytemuck_cast_slice::< f32>( &buffOut),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );

    let  	shader = device.create_shader_module( wgpu::ShaderModuleDescriptor {
        label: Some( "vecadd_shader"),
        source: wgpu::ShaderSource::Wgsl( SHADER_VECTOR_ADD.into()),
    });
    let  	bindGroupLayout = device.create_bind_group_layout( &wgpu::BindGroupLayoutDescriptor {
        label: Some( "vecadd_bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let  	pipelineLayout = device.create_pipeline_layout( &wgpu::PipelineLayoutDescriptor {
        label: Some( "vecadd_pl"),
        bind_group_layouts: &[Some( &bindGroupLayout)],
        immediate_size: 0,
    });
    let  	pipeline = device.create_compute_pipeline( &wgpu::ComputePipelineDescriptor {
        label: Some( "vecadd_pipeline"),
        layout: Some( &pipelineLayout),
        module: &shader,
        entry_point: Some( "main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let  	bindGroup = device.create_bind_group( &wgpu::BindGroupDescriptor {
        label: Some( "vecadd_bg"),
        layout: &bindGroupLayout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: gpuA.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: gpuB.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: gpuOut.as_entire_binding(),
            },
        ],
    });

    let  	workgroups = ( szData.AsU32() + 63) / 64;
    let  	mut encoder = device.create_command_encoder( &wgpu::CommandEncoderDescriptor {
        label: Some( "vecadd_enc"),
    });
    {
        let  	mut pass = encoder.begin_compute_pass( &wgpu::ComputePassDescriptor {
            label: Some( "vecadd_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline( &pipeline);
        pass.set_bind_group( 0, &bindGroup, &[]);
        pass.dispatch_workgroups( workgroups, 1, 1);
    }
    queue.submit( std::iter::once( encoder.finish()));

    let  	raw = GpuReadBuffer( &device, &queue, &gpuOut, byteLen as u64);
    let  	result: &[f32] = bytemuck_cast_slice_from::< f32>( &raw);
    for i in 0..szData.AsUsize() {
        let  	expected = ( i as f32) + ( i as f32 * 10.0);
        assert!(
            ( result[i] - expected).abs() < 1e-6,
            "VectorAdd mismatch at {}: got {}, expected {}",
            i, result[i], expected,
        );
    }
    println!( "TestGpuVectorAdd: {} element vector add on GPU ✓", szData);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestGpuCollatz()
{
    let  	(device, queue) = match GpuInit() {
        Some( dq) => dq,
        None => {
            println!( "No GPU adapter found — skipping TestGpuCollatz");
            return;
        }
    };

    let  	szData = U32( 128);
    let  	inputBuff = Buff::Create( szData, |i| i.AsU32() + 1);
    let  	outputBuff = Buff::New( szData, 0u32);
    let  	byteLen = szData.AsUsize() * std::mem::size_of::< u32>();

    let  	gpuIn = GpuBufferInit(
        &device, "collatz_in",
        bytemuck_cast_slice::< u32>( &inputBuff),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let  	gpuOut = GpuBufferInit(
        &device, "collatz_out",
        bytemuck_cast_slice::< u32>( &outputBuff),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );

    let  	shader = device.create_shader_module( wgpu::ShaderModuleDescriptor {
        label: Some( "collatz_shader"),
        source: wgpu::ShaderSource::Wgsl( SHADER_COLLATZ.into()),
    });
    let  	bindGroupLayout = device.create_bind_group_layout( &wgpu::BindGroupLayoutDescriptor {
        label: Some( "collatz_bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let  	pipelineLayout = device.create_pipeline_layout( &wgpu::PipelineLayoutDescriptor {
        label: Some( "collatz_pl"),
        bind_group_layouts: &[Some( &bindGroupLayout)],
        immediate_size: 0,
    });
    let  	pipeline = device.create_compute_pipeline( &wgpu::ComputePipelineDescriptor {
        label: Some( "collatz_pipeline"),
        layout: Some( &pipelineLayout),
        module: &shader,
        entry_point: Some( "main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let  	bindGroup = device.create_bind_group( &wgpu::BindGroupDescriptor {
        label: Some( "collatz_bg"),
        layout: &bindGroupLayout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: gpuIn.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: gpuOut.as_entire_binding(),
            },
        ],
    });

    let  	workgroups = ( szData.AsU32() + 63) / 64;
    let  	mut encoder = device.create_command_encoder( &wgpu::CommandEncoderDescriptor {
        label: Some( "collatz_enc"),
    });
    {
        let  	mut pass = encoder.begin_compute_pass( &wgpu::ComputePassDescriptor {
            label: Some( "collatz_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline( &pipeline);
        pass.set_bind_group( 0, &bindGroup, &[]);
        pass.dispatch_workgroups( workgroups, 1, 1);
    }
    queue.submit( std::iter::once( encoder.finish()));

    let  	raw = GpuReadBuffer( &device, &queue, &gpuOut, byteLen as u64);
    let  	result: &[u32] = bytemuck_cast_slice_from::< u32>( &raw);

    // CPU reference: compute Collatz steps for each value
    for i in 0..szData.AsUsize() {
        let  	mut n = ( i as u32) + 1;
        let  	mut steps = 0u32;
        while n != 1 {
            if n % 2 == 0 {
                n /= 2;
            } else {
                n = 3 * n + 1;
            }
            steps += 1;
        }
        assert_eq!(
            result[i], steps,
            "Collatz mismatch at input {}: GPU={}, CPU={}",
            i + 1, result[i], steps,
        );
    }
    println!( "TestGpuCollatz: {} Collatz sequences computed on GPU ✓", szData);
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Safe cast from a typed slice to a byte slice (like bytemuck::cast_slice).
/// Only valid for `Copy` types with no padding concerns for f32/u32.
fn	bytemuck_cast_slice< T: Copy>( data: &[T]) -> &[u8]
{
    unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const u8,
            data.len() * std::mem::size_of::< T>(),
        )
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Safe cast from a byte slice back to a typed slice.
fn	bytemuck_cast_slice_from< T: Copy>( data: &[u8]) -> &[T]
{
    let  	szT = std::mem::size_of::< T>();
    assert!( szT > 0, "Cannot cast to ZST");
    assert_eq!( data.len() % szT, 0, "Byte slice length not aligned to target type");
    unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const T,
            data.len() / szT,
        )
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
