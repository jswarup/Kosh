//-- swarm/_tests.rs -----------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Buff, ISliceExt, U32 };
use	crate::swarm::IGpuOp;

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

#[test]
fn	TestGpuDoubleValues()
{
    let  	( device, queue) = match wgpu::Device::Init() {
        Some( dq) => dq,
        None => {
            println!( "No GPU adapter found — skipping TestGpuDoubleValues");
            return;
        }
    };

    // Prepare input data using project Buff
    let  	szData = U32( 256);
    let  	input = Buff::Create( szData, |i| ( i.AsU32() + 1) as f32);
    let  	byteLen = szData.AsUsize() * std::mem::size_of::< f32>();

    // Upload to GPU
    let  	gpuBuf = device.BufferInit(
        "double_data",
        input.CastSlice(),
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
    let  	raw = device.ReadBuffer( &queue, &gpuBuf, byteLen as u64);
    let  	result: &[f32] = raw.CastSliceFrom();
    for i in 0..szData.AsUsize() {
        let  	expected = ( i as f32 + 1.0) * 2.0;
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
    let  	( device, queue) = match wgpu::Device::Init() {
        Some( dq) => dq,
        None => {
            println!( "No GPU adapter found — skipping TestGpuVectorAdd");
            return;
        }
    };

    let  	szData = U32( 512);
    let  	buffA = Buff::Create( szData, |i| i.AsU32() as f32);
    let  	buffB = Buff::Create( szData, |i| ( i.AsU32() * 10) as f32);
    let  	buffOut = Buff::Create( szData, |_| 0.0f32);
    let  	byteLen = szData.AsUsize() * std::mem::size_of::< f32>();

    let  	gpuA = device.BufferInit(
        "vecadd_a",
        buffA.CastSlice(),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let  	gpuB = device.BufferInit(
        "vecadd_b",
        buffB.CastSlice(),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let  	gpuOut = device.BufferInit(
        "vecadd_out",
        buffOut.CastSlice(),
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

    let  	raw = device.ReadBuffer( &queue, &gpuOut, byteLen as u64);
    let  	result: &[f32] = raw.CastSliceFrom();
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
    let  	( device, queue) = match wgpu::Device::Init() {
        Some( dq) => dq,
        None => {
            println!( "No GPU adapter found — skipping TestGpuCollatz");
            return;
        }
    };

    let  	szData = U32( 128);
    let  	inputBuff = Buff::Create( szData, |i| i.AsU32() + 1);
    let  	outputBuff = Buff::Create( szData, |_| 0u32);
    let  	byteLen = szData.AsUsize() * std::mem::size_of::< u32>();

    let  	gpuIn = device.BufferInit(
        "collatz_in",
        inputBuff.CastSlice(),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let  	gpuOut = device.BufferInit(
        "collatz_out",
        outputBuff.CastSlice(),
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

    let  	raw = device.ReadBuffer( &queue, &gpuOut, byteLen as u64);
    let  	result: &[u32] = raw.CastSliceFrom();

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


//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestRustGpuComputeExample()
{
    let  	( device, queue) = match wgpu::Device::Init() {
        Some( dq) => dq,
        None => {
            println!( "No GPU adapter found — skipping TestRustGpuComputeExample");
            return;
        }
    };

    let  	szData = U32( 1048576);
    let  	inputBuff = Buff::Create( szData, |i| i.AsU32() + 1);
    let  	outputBuff = Buff::Create( szData, |_| 0u32);
    let  	byteLen = szData.AsUsize() * std::mem::size_of::< u32>();

    let  	gpuIn = device.BufferInit(
        "collatz_rec_in",
        inputBuff.CastSlice(),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let  	gpuOut = device.BufferInit(
        "collatz_rec_out",
        outputBuff.CastSlice(),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );

    let  	compileResult = spirv_builder::SpirvBuilder::new( "src/gcomp", "spirv-unknown-vulkan1.1")
        .build()
        .unwrap();
    let  	modulePath = match compileResult.module {
        spirv_builder::ModuleResult::SingleModule( p) => p,
        spirv_builder::ModuleResult::MultiModule( m) => m.into_iter().next().unwrap().1,
    };
    let  	spirvData = std::fs::read( modulePath).unwrap();
    let  	spirv = std::borrow::Cow::Owned( wgpu::util::make_spirv_raw( &spirvData).into_owned());

    let  	shader = device.create_shader_module( wgpu::ShaderModuleDescriptor {
        label: Some( "collatz_rec_shader"),
        source: wgpu::ShaderSource::SpirV( spirv),
    });
    let  	bindGroupLayout = device.create_bind_group_layout( &wgpu::BindGroupLayoutDescriptor {
        label: Some( "collatz_rec_bgl"),
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
        label: Some( "collatz_rec_pl"),
        bind_group_layouts: &[Some( &bindGroupLayout)],
        immediate_size: 0,
    });
    let  	pipeline = device.create_compute_pipeline( &wgpu::ComputePipelineDescriptor {
        label: Some( "collatz_rec_pipeline"),
        layout: Some( &pipelineLayout),
        module: &shader,
        entry_point: Some( "main_cs"),
        compilation_options: Default::default(),
        cache: None,
    });
    let  	bindGroup = device.create_bind_group( &wgpu::BindGroupDescriptor {
        label: Some( "collatz_rec_bg"),
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
        label: Some( "collatz_rec_enc"),
    });
    {
        let  	mut pass = encoder.begin_compute_pass( &wgpu::ComputePassDescriptor {
            label: Some( "collatz_rec_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline( &pipeline);
        pass.set_bind_group( 0, &bindGroup, &[]);
        pass.dispatch_workgroups( workgroups, 1, 1);
    }
    queue.submit( std::iter::once( encoder.finish()));

    let  	raw = device.ReadBuffer( &queue, &gpuOut, byteLen as u64);
    let  	result: &[u32] = raw.CastSliceFrom();

    println!( "1: 0");
    let  	mut max = 0;
    for i in 0..szData.AsUsize() {
        let  	src = ( i as u32) + 1;
        let  	out = result[i];
        if out == u32::MAX {
            println!( "{}: overflowed", src);
            break;
        } else if out > max {
            max = out;
            println!( "{}: {}", src, out);
        }
    }
    println!( "TestRustGpuComputeExample: Computed Collatz records up to {} ✓", szData.AsU32());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestCpuDoubleValues()
{
    use	crate::swarm::cpu::CpuDevice;
    use	crate::swarm::traits::{ BufferUsage, IComputeDevice, WorkgroupDim };

    let  	cpu = CpuDevice::New();
    let  	szData = U32( 256);
    let  	input = Buff::Create( szData, |i| ( i.AsU32() + 1) as f32);

    let  	kernel = CpuDevice::DoubleKernel();
    let  	buf = cpu.CreateBufferInit( "cpu_double", input.CastSlice(), BufferUsage::STORAGE).unwrap();

    let  	workgroups = ( szData.AsU32() + 63) / 64;
    cpu.Dispatch( &kernel, &[buf.as_ref()], WorkgroupDim::Linear( U32( workgroups))).unwrap();

    let  	raw = buf.Read().unwrap();
    let  	result: &[f32] = raw.CastSliceFrom();
    for i in 0..szData.AsUsize() {
        let  	expected = ( i as f32 + 1.0) * 2.0;
        assert!(
            ( result[i] - expected).abs() < 1e-6,
            "CPU mismatch at {}: got {}, expected {}",
            i, result[i], expected,
        );
    }
    println!( "TestCpuDoubleValues: {} values doubled on CPU ✓", szData);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestCpuVectorAdd()
{
    use	crate::swarm::cpu::CpuDevice;
    use	crate::swarm::traits::{ BufferUsage, IComputeDevice, WorkgroupDim };

    let  	cpu = CpuDevice::New();
    let  	szData = U32( 512);
    let  	buffA = Buff::Create( szData, |i| i.AsU32() as f32);
    let  	buffB = Buff::Create( szData, |i| ( i.AsU32() * 10) as f32);
    let  	buffOut = Buff::Create( szData, |_| 0.0f32);

    let  	kernel = CpuDevice::VectorAddKernel();
    let  	bufA = cpu.CreateBufferInit( "cpu_vec_a", buffA.CastSlice(), BufferUsage::STORAGE).unwrap();
    let  	bufB = cpu.CreateBufferInit( "cpu_vec_b", buffB.CastSlice(), BufferUsage::STORAGE).unwrap();
    let  	bufOut = cpu.CreateBufferInit( "cpu_vec_out", buffOut.CastSlice(), BufferUsage::STORAGE).unwrap();

    let  	workgroups = ( szData.AsU32() + 63) / 64;
    cpu.Dispatch( &kernel, &[bufA.as_ref(), bufB.as_ref(), bufOut.as_ref()], WorkgroupDim::Linear( U32( workgroups))).unwrap();

    let  	raw = bufOut.Read().unwrap();
    let  	result: &[f32] = raw.CastSliceFrom();
    for i in 0..szData.AsUsize() {
        let  	expected = ( i as f32) + ( i as f32 * 10.0);
        assert!(
            ( result[i] - expected).abs() < 1e-6,
            "CPU VectorAdd mismatch at {}: got {}, expected {}",
            i, result[i], expected,
        );
    }
    println!( "TestCpuVectorAdd: {} elements added on CPU ✓", szData);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestCpuCollatz()
{
    use	crate::swarm::cpu::CpuDevice;
    use	crate::swarm::traits::{ BufferUsage, IComputeDevice, WorkgroupDim };

    let  	cpu = CpuDevice::New();
    let  	szData = U32( 128);
    let  	inputBuff = Buff::Create( szData, |i| i.AsU32() + 1);
    let  	outputBuff = Buff::Create( szData, |_| 0u32);

    let  	kernel = CpuDevice::CollatzKernel();
    let  	bufIn = cpu.CreateBufferInit( "cpu_collatz_in", inputBuff.CastSlice(), BufferUsage::STORAGE).unwrap();
    let  	bufOut = cpu.CreateBufferInit( "cpu_collatz_out", outputBuff.CastSlice(), BufferUsage::STORAGE).unwrap();

    let  	workgroups = ( szData.AsU32() + 63) / 64;
    cpu.Dispatch( &kernel, &[bufIn.as_ref(), bufOut.as_ref()], WorkgroupDim::Linear( U32( workgroups))).unwrap();

    let  	raw = bufOut.Read().unwrap();
    let  	result: &[u32] = raw.CastSliceFrom();
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
        assert_eq!( result[i], steps, "Collatz mismatch at {}", i + 1);
    }
    println!( "TestCpuCollatz: {} Collatz sequences computed on CPU ✓", szData);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestCpuPointCloud()
{
    use	crate::swarm::cpu::CpuDevice;
    use	crate::swarm::traits::{ BufferUsage, IComputeDevice, WorkgroupDim };

    let  	cpu = CpuDevice::New();
    let  	numPoints = U32( 100);
    let  	numFloats = U32( ( numPoints.AsUsize() * 4) as u32);
    let  	zeroBuff = Buff::Create( numFloats, |_| 0.0f32);

    let  	kernel = CpuDevice::PointCloudKernel();
    let  	bufOut = cpu.CreateBufferInit( "cpu_pointcloud_out", zeroBuff.CastSlice(), BufferUsage::STORAGE).unwrap();

    let  	workgroups = ( numPoints.AsU32() + 63) / 64;
    cpu.Dispatch( &kernel, &[bufOut.as_ref()], WorkgroupDim::Linear( U32( workgroups))).unwrap();

    let  	raw = bufOut.Read().unwrap();
    let  	floats: &[f32] = raw.CastSliceFrom();
    assert_eq!( floats.len(), 400);

    for i in 0..numPoints.AsUsize() {
        let  	base = i * 4;
        let  	x = floats[base + 0];
        let  	y = floats[base + 1];
        let  	z = floats[base + 2];
        let  	w = floats[base + 3];
        assert_eq!( w, 1.0);
        assert!( x >= -20.0 && x <= 20.0, "x out of range: {}", x);
        assert!( y >= -20.0 && y <= 20.0, "y out of range: {}", y);
        assert!( z >= -20.0 && z <= 20.0, "z out of range: {}", z);
    }
    println!( "TestCpuPointCloud: 100 3D points generated on CPU ✓");
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestCudaOxidePtxExecution()
{
    use	crate::swarm::cudaoxide::CudaOxideDevice;
    use	crate::swarm::traits::{ BufferUsage, IComputeDevice, KernelSource, WorkgroupDim };

    let  	cuda = CudaOxideDevice::Init().unwrap();
    let  	szData = U32( 256);
    let  	input = Buff::Create( szData, |i| ( i.AsU32() + 1) as f32);

    let  	ptxSource = ".version 7.0\n.target sm_70\n.entry double_kernel";
    let  	kernel = cuda.CompileKernel( "double_kernel", "double_kernel", KernelSource::Ptx( ptxSource)).unwrap();
    let  	buf = cuda.CreateBufferInit( "cuda_double", input.CastSlice(), BufferUsage::STORAGE).unwrap();

    let  	workgroups = ( szData.AsU32() + 63) / 64;
    cuda.Dispatch( kernel.as_ref(), &[buf.as_ref()], WorkgroupDim::Linear( U32( workgroups))).unwrap();

    let  	raw = buf.Read().unwrap();
    let  	result: &[f32] = raw.CastSliceFrom();
    for i in 0..szData.AsUsize() {
        let  	expected = ( i as f32 + 1.0) * 2.0;
        assert!(
            ( result[i] - expected).abs() < 1e-6,
            "Cuda-Oxide mismatch at {}: got {}, expected {}",
            i, result[i], expected,
        );
    }
    println!( "TestCudaOxidePtxExecution: {} values doubled via Cuda-Oxide PTX ✓", szData);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestSwarmEngineBackendSwitching()
{
    use	crate::swarm::engine::SwarmEngine;
    use	crate::swarm::traits::BackendKind;

    let  	testData: Buff< f32> = ( 1..=64).map( |x| x as f32).collect();

    // 1. Execute on CPU backend
    let  	cpuEngine = SwarmEngine::New( BackendKind::Cpu).unwrap();
    assert_eq!( cpuEngine.Backend(), BackendKind::Cpu);
    let  	cpuResult = cpuEngine.RunDouble( &testData).unwrap();

    // 2. Execute on Cuda-Oxide backend
    let  	cudaEngine = SwarmEngine::New( BackendKind::CudaOxide).unwrap();
    assert_eq!( cudaEngine.Backend(), BackendKind::CudaOxide);
    let  	cudaResult = cudaEngine.RunDouble( &testData).unwrap();

    // 3. Execute on Rust-GPU backend (if hardware adapter is available)
    let  	rustGpuResult = match SwarmEngine::New( BackendKind::RustGpu) {
        Ok( engine) => {
            assert_eq!( engine.Backend(), BackendKind::RustGpu);
            engine.RunDouble( &testData).ok()
        }
        Err( _) => None,
    };

    // Verify consistency across all active backends
    for i in 0..testData.len() {
        let  	expected = testData[i] * 2.0;
        assert_eq!( cpuResult[i], expected, "CPU backend output mismatch at index {}", i);
        assert_eq!( cudaResult[i], expected, "Cuda-Oxide backend output mismatch at index {}", i);
        if let Some( ref gpuRes) = rustGpuResult {
            assert!(
                ( gpuRes[i] - expected).abs() < 1e-6,
                "Rust-GPU backend output mismatch at index {}", i,
            );
        }
    }

    // Test Vector Add switching
    let  	vecA = Buff![1.0f32, 2.0, 3.0, 4.0];
    let  	vecB = Buff![10.0f32, 20.0, 30.0, 40.0];
    let  	cpuAdd = cpuEngine.RunVectorAdd( &vecA, &vecB).unwrap();
    let  	cudaAdd = cudaEngine.RunVectorAdd( &vecA, &vecB).unwrap();
    assert_eq!( cpuAdd, Buff![11.0, 22.0, 33.0, 44.0]);
    assert_eq!( cudaAdd, Buff![11.0, 22.0, 33.0, 44.0]);

    // Test Collatz switching
    let  	collatzIn = Buff![1u32, 2, 3, 4, 5, 6, 7];
    let  	cpuCollatz = cpuEngine.RunCollatz( &collatzIn).unwrap();
    let  	cudaCollatz = cudaEngine.RunCollatz( &collatzIn).unwrap();
    assert_eq!( cpuCollatz, cudaCollatz);

    // Test Point Cloud switching
    let  	cpuPoints = cpuEngine.RunPointCloud( U32( 100), None).unwrap();
    let  	cudaPoints = cudaEngine.RunPointCloud( U32( 100), None).unwrap();
    assert_eq!( cpuPoints.len(), 100);
    assert_eq!( cudaPoints.len(), 100);

    println!( "TestSwarmEngineBackendSwitching: Unified compute verified across CPU, Cuda-Oxide, and Rust-GPU ✓");
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestSwarmEngineAuto()
{
    use	crate::swarm::engine::SwarmEngine;

    let  	engine = SwarmEngine::Auto();
    println!( "TestSwarmEngineAuto: Auto-selected backend is {} ✓", engine.Backend());

    let  	data = Buff![5.0f32, 10.0, 15.0];
    let  	result = engine.RunDouble( &data).unwrap();
    assert_eq!( result, Buff![10.0, 20.0, 30.0]);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestSharedGcompKernelParity()
{
    use	crate::swarm::traits::BackendKind;
    use	crate::swarm::engine::SwarmEngine;

    // 1. Direct invocation of shared gcomp kernels
    let  	mut testData = Buff![1.0f32, 2.0, 3.0, 4.0];
    for i in 0..testData.len() {
        gcomp::double_elem( i, &mut testData);
    }
    assert_eq!( testData, Buff![2.0, 4.0, 6.0, 8.0]);

    let  	a = Buff![10.0f32, 20.0, 30.0];
    let  	b = Buff![1.0f32, 2.0, 3.0];
    let  	mut sumOut = Buff![0.0f32, 0.0, 0.0];
    for i in 0..3 {
        gcomp::vector_add_elem( i, &a, &b, &mut sumOut);
    }
    assert_eq!( sumOut, Buff![11.0, 22.0, 33.0]);

    let  	collatzIn = Buff![6u32, 11, 27];
    let  	mut collatzOut = Buff![0u32, 0, 0];
    for i in 0..3 {
        gcomp::collatz_elem( i, &collatzIn, &mut collatzOut);
    }
    assert_eq!( collatzOut, Buff![8, 14, 111]);

    // 2. Cross-backend parity between CPU and Cuda-Oxide running shared gcomp kernels
    let  	cpuEngine = SwarmEngine::New( BackendKind::Cpu).unwrap();
    let  	cudaEngine = SwarmEngine::New( BackendKind::CudaOxide).unwrap();

    let  	cpuDouble = cpuEngine.RunDouble( &a).unwrap();
    let  	cudaDouble = cudaEngine.RunDouble( &a).unwrap();
    assert_eq!( cpuDouble, cudaDouble);

    let  	cpuAdd = cpuEngine.RunVectorAdd( &a, &b).unwrap();
    let  	cudaAdd = cudaEngine.RunVectorAdd( &a, &b).unwrap();
    assert_eq!( cpuAdd, cudaAdd);

    let  	cpuCollatz = cpuEngine.RunCollatz( &collatzIn).unwrap();
    let  	cudaCollatz = cudaEngine.RunCollatz( &collatzIn).unwrap();
    assert_eq!( cpuCollatz, cudaCollatz);

    // 3. Test swarm_kernel! macro definition
    crate::swarm_kernel!( CustomScaleKernel, |idx, input: &[f32], output: &mut [f32]| {
        if idx < output.len() && idx < input.len() {
            output[idx] = input[idx] * 10.0;
        }
    });

    let  	inVec = Buff![1.0f32, 2.0, 3.0];
    let  	mut outVec = Buff![0.0f32, 0.0, 0.0];
    for i in 0..3 {
        CustomScaleKernel( i, &inVec, &mut outVec);
    }
    assert_eq!( outVec, Buff![10.0, 20.0, 30.0]);

    println!( "TestSharedGcompKernelParity: CPU, Cuda-Oxide, and gcomp shared kernel parity verified ✓");
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestSwarmCameraTransform()
{
    use	crate::swarm::engine::SwarmEngine;
    use	crate::swarm::traits::BackendKind;

    let  	points = [
        [ 0.0f32, 0.0, 0.0 ],
        [ 10.0, 10.0, 10.0 ],
        [ -10.0, -10.0, -10.0 ],
    ];

    let  	camParams: [f32; 13] = [
        0.4,   // rotX
        0.6,   // rotY
        1.0,   // zoom
        0.0,   // panX
        0.0,   // panY
        350.0, // fov
        250.0, // distance
        800.0, // width
        600.0, // height
        0.0,   // cx
        0.0,   // cy
        0.0,   // cz
        1.0,   // scaleNorm
    ];

    let  	cpuEngine = SwarmEngine::New( BackendKind::Cpu).unwrap();
    let  	resCpu = cpuEngine.RunCameraTransform( &points, &camParams, None);
    assert!( resCpu.is_ok());

    let  	projectedCpu = resCpu.unwrap();
    assert_eq!( projectedCpu.len(), 3);

    // Verify first point (0,0,0) projects to screen center (400, 300)
    assert!( ( projectedCpu[0][0] - 400.0).abs() < 1e-4);
    assert!( ( projectedCpu[0][1] - 300.0).abs() < 1e-4);
    assert!( projectedCpu[0][2] > 0.0); // radius
    assert!( projectedCpu[0][3] > 0.0); // core_radius
    assert!( projectedCpu[0][4] > 0.0); // alpha

    let  	autoEngine = SwarmEngine::Auto();
    let  	resAuto = autoEngine.RunCameraTransform( &points, &camParams, None);
    assert!( resAuto.is_ok());
    let  	projectedAuto = resAuto.unwrap();
    assert_eq!( projectedAuto.len(), 3);

    println!( "TestSwarmCameraTransform: Camera transformation verified on Swarm (backend: {}) ✓", autoEngine.Backend());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestSwarmClusterSharding()
{
    use	crate::swarm::engine::SwarmCluster;

    let  	cluster = SwarmCluster::Auto();
    assert!( cluster.DeviceCount() >= 1);

    let  	mut points = Buff::New();
    for i in 0..200 {
        let  	f = i as f32;
        points.Push( [ f, f * 2.0, f * 0.5 ]);
    }

    let  	camParams: [f32; 13] = [
        0.0, 0.0, 1.0, 0.0, 0.0, 350.0, 250.0, 800.0, 600.0, 0.0, 0.0, 0.0, 1.0,
    ];

    let  	res = cluster.RunCameraTransformSharded( &points, &camParams, None);
    assert!( res.is_ok());

    let  	projected = res.unwrap();
    assert_eq!( projected.len(), 200);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestGcompFrustumCulling()
{
    let  	inPoints = [
        0.0f32, 0.0, 0.0,      // Inside (center)
        1000.0, 0.0, 0.0,     // Outside (far right)
        0.0, 1000.0, 0.0,     // Outside (far top)
    ];

    // 6 simple frustum bounding planes centered at origin with half-extent 100
    // Right, Left, Top, Bottom, Far, Near
    let  	planes = [
        -1.0f32, 0.0, 0.0, 100.0,  // x <= 100
         1.0, 0.0, 0.0, 100.0,     // x >= -100
        0.0, -1.0, 0.0, 100.0,     // y <= 100
        0.0,  1.0, 0.0, 100.0,     // y >= -100
        0.0, 0.0, -1.0, 100.0,     // z <= 100
        0.0, 0.0,  1.0, 100.0,     // z >= -100
    ];

    let  	mut outVisible = [0u32; 3];
    for i in 0..3 {
        gcomp::frustum_cull_elem( i, &inPoints, &planes, &mut outVisible);
    }

    assert_eq!( outVisible[0], 1); // inside
    assert_eq!( outVisible[1], 0); // culled
    assert_eq!( outVisible[2], 0); // culled
}

//---------------------------------------------------------------------------------------------------------------------------------

