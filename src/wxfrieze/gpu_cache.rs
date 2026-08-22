//-- frieze/gpu_cache.rs -------------------------------------------------------------------------------------------------------------
use	std::collections::HashMap;
use	std::path::{ Path, PathBuf };
use	wgpu::Device;
use	crate::silo::Buff;
use	crate::swarm::viewport::{ GpuMesh, ViewportVertex };
use	crate::fenst::{ XplrParsePtsFile, XplrParseWaveObjFile };

// ---------------------------------------------------------------------------------------------------------------------------------

/// Cached 3D geometry data holding both memory geometry buffers and GPU VRAM resources.
#[derive( Clone)]
pub struct LoadedMesh
{
    pub _Points:        Buff< [f32; 3]>,
    pub _Normals:       Buff< [f32; 3]>,
    pub _Triangles:     Buff< [u32; 3]>,
    pub _BboxMin:       [f32; 3],
    pub _BboxMax:       [f32; 3],
    pub _Center:        [f32; 3],
    pub _ScaleNorm:     f32,
    pub _GpuMesh:       Option< GpuMesh>,
}

impl LoadedMesh
{
    pub fn	PointCount( &self) -> usize
    {
        self._Points.len()
    }

    pub fn	FaceCount( &self) -> usize
    {
        self._Triangles.len()
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// GPU Mesh Cache preventing re-parsing of geometry files every frame.
#[derive( Default)]
pub struct GpuMeshCache
{
    _Cache: HashMap< PathBuf, LoadedMesh>,
}

impl GpuMeshCache
{
    pub fn	New() -> Self
    {
        Self {
            _Cache: HashMap::new(),
        }
    }

    /// Gets or loads a geometry model for the given file path.
    pub fn	GetOrLoad( &mut self, device: Option< &Device>, path: &Path) -> Option< &LoadedMesh>
    {
        if self._Cache.contains_key( path) {
            return self._Cache.get( path);
        }

        let  	mesh = Self::LoadMesh( device, path)?;
        self._Cache.insert( path.to_path_buf(), mesh);
        self._Cache.get( path)
    }

    /// Loads and parses a geometry file once into memory.
    fn	LoadMesh( device: Option< &Device>, path: &Path) -> Option< LoadedMesh>
    {
        let  	pathStr = path.to_string_lossy().to_string();

        if pathStr.ends_with( ".pts") {
            let  	dto = XplrParsePtsFile( &pathStr).ok()?;
            let  	pts = dto._Points;

            let  	mut minX = f32::MAX; let  	mut maxX = f32::MIN;
            let  	mut minY = f32::MAX; let  	mut maxY = f32::MIN;
            let  	mut minZ = f32::MAX; let  	mut maxZ = f32::MIN;

            for p in pts.iter() {
                minX = minX.min( p[0]); maxX = maxX.max( p[0]);
                minY = minY.min( p[1]); maxY = maxY.max( p[1]);
                minZ = minZ.min( p[2]); maxZ = maxZ.max( p[2]);
            }

            if minX == f32::MAX {
                minX = -50.0; maxX = 50.0;
                minY = -50.0; maxY = 50.0;
                minZ = -50.0; maxZ = 50.0;
            }

            let  	cx = ( minX + maxX) * 0.5;
            let  	cy = ( minY + maxY) * 0.5;
            let  	cz = ( minZ + maxZ) * 0.5;
            let  	dx = maxX - minX;
            let  	dy = maxY - minY;
            let  	dz = maxZ - minZ;
            let  	maxDim = dx.max( dy).max( dz);
            let  	scaleNorm = if maxDim > 1e-4 { 240.0 / maxDim } else { 1.0 };

            let  	gpuMesh = device.map( |dev| {
                let  	vertices: Vec< ViewportVertex> = pts.iter().map( |p| {
                    ViewportVertex {
                        _Pos:    *p,
                        _Normal: [0.0, 1.0, 0.0],
                    }
                }).collect();

                GpuMesh::FromVerticesAndIndices(
                    dev,
                    &vertices,
                    None,
                    None,
                    [minX, minY, minZ],
                    [maxX, maxY, maxZ],
                )
            });

            Some( LoadedMesh {
                _Points:    pts,
                _Normals:   Buff::New(),
                _Triangles: Buff::New(),
                _BboxMin:   [minX, minY, minZ],
                _BboxMax:   [maxX, maxY, maxZ],
                _Center:    [cx, cy, cz],
                _ScaleNorm: scaleNorm,
                _GpuMesh:   gpuMesh,
            })
        } else if pathStr.ends_with( ".obj") {
            let  	dto = XplrParseWaveObjFile( &pathStr).ok()?;
            let  	pts = dto._Points;
            let  	normals = dto._Normals;
            let  	triangles = dto._Triangles;

            let  	mut minX = f32::MAX; let  	mut maxX = f32::MIN;
            let  	mut minY = f32::MAX; let  	mut maxY = f32::MIN;
            let  	mut minZ = f32::MAX; let  	mut maxZ = f32::MIN;

            for p in pts.iter() {
                minX = minX.min( p[0]); maxX = maxX.max( p[0]);
                minY = minY.min( p[1]); maxY = maxY.max( p[1]);
                minZ = minZ.min( p[2]); maxZ = maxZ.max( p[2]);
            }

            if minX == f32::MAX {
                minX = -50.0; maxX = 50.0;
                minY = -50.0; maxY = 50.0;
                minZ = -50.0; maxZ = 50.0;
            }

            let  	cx = ( minX + maxX) * 0.5;
            let  	cy = ( minY + maxY) * 0.5;
            let  	cz = ( minZ + maxZ) * 0.5;
            let  	dx = maxX - minX;
            let  	dy = maxY - minY;
            let  	dz = maxZ - minZ;
            let  	maxDim = dx.max( dy).max( dz);
            let  	scaleNorm = if maxDim > 1e-4 { 240.0 / maxDim } else { 1.0 };

            let  	gpuMesh = device.map( |dev| {
                let  	vertices: Vec< ViewportVertex> = pts.iter().enumerate().map( |( i, p)| {
                    let  	norm = if i < normals.len() { normals[i] } else { [0.0, 1.0, 0.0] };
                    ViewportVertex {
                        _Pos:    *p,
                        _Normal: norm,
                    }
                }).collect();

                let  	indices: Vec< u32> = triangles.iter().flat_map( |t| t.iter().cloned()).collect();

                let  	mut wireIndices: Vec< u32> = Vec::with_capacity( triangles.len() * 6);
                for t in triangles.iter() {
                    wireIndices.push( t[0]); wireIndices.push( t[1]);
                    wireIndices.push( t[1]); wireIndices.push( t[2]);
                    wireIndices.push( t[2]); wireIndices.push( t[0]);
                }

                GpuMesh::FromVerticesAndIndices(
                    dev,
                    &vertices,
                    Some( &indices),
                    Some( &wireIndices),
                    [minX, minY, minZ],
                    [maxX, maxY, maxZ],
                )
            });

            Some( LoadedMesh {
                _Points:    pts,
                _Normals:   normals,
                _Triangles: triangles,
                _BboxMin:   [minX, minY, minZ],
                _BboxMax:   [maxX, maxY, maxZ],
                _Center:    [cx, cy, cz],
                _ScaleNorm: scaleNorm,
                _GpuMesh:   gpuMesh,
            })
        } else {
            None
        }
    }
}
