// =================================================================
// Fenst — Rust-GPU Point Cloud Viewer (.pts files)
// Renders 100 GPU-generated 3D points from gcomp shader pipeline
// =================================================================

(function () {
    const urlParams = new URLSearchParams(window.location.search);
    const filePath = urlParams.get('file') || 'pointcloud.pts';
    const fileName = filePath.split(/[/\\]/).pop();

    const titleEl = document.getElementById('pts-filename-txt');
    if (titleEl) {
        titleEl.textContent = fileName;
    }

    const canvas = document.getElementById('pts-canvas');
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    const rotSpeedInput = document.getElementById('pts-rot-speed');
    const lineColorInput = document.getElementById('pts-line-color');

    // Stats elements for updating after GPU data arrives
    const statPointCount = document.getElementById('pts-stat-count');
    const statBbox = document.getElementById('pts-stat-bbox');
    const statShaderStatus = document.getElementById('pts-shader-status');

    let angleX = 0.4;
    let angleY = 0.6;

    // Point cloud data — populated by GPU compute shader
    let points = [];
    let bboxMin = [-20, -20, -20];
    let bboxMax = [20, 20, 20];

    // Fetch point cloud from GPU compute shader via Tauri IPC
    async function fetchPointCloud() {
        try {
            const { invoke } = window.__TAURI__.core;
            const data = await invoke('XplrFetchPtsPoints');
            points = data.points;
            bboxMin = data.bbox_min;
            bboxMax = data.bbox_max;

            // Update sidebar stats
            if (statPointCount) statPointCount.textContent = data.count;
            if (statBbox) {
                statBbox.textContent = `[${bboxMin.map(v => v.toFixed(0)).join(', ')}] → [${bboxMax.map(v => v.toFixed(0)).join(', ')}]`;
            }
            if (statShaderStatus) statShaderStatus.textContent = 'Shader Active: gcomp::pts_pointcloud_cs';
        } catch (err) {
            console.error('Failed to fetch point cloud:', err);
            if (statShaderStatus) statShaderStatus.textContent = 'Shader Error: ' + err;
        }
    }

    function resizeCanvas() {
        const dpr = window.devicePixelRatio || 1;
        const w = canvas.clientWidth || canvas.parentElement?.clientWidth || window.innerWidth || 960;
        const h = canvas.clientHeight || canvas.parentElement?.clientHeight || (window.innerHeight - 48) || 672;
        canvas.width = Math.max(w * dpr, 300);
        canvas.height = Math.max(h * dpr, 300);
    }

    window.addEventListener('resize', resizeCanvas);
    resizeCanvas();

    async function render() {
        resizeCanvas();
        const width = canvas.width;
        const height = canvas.height;
        const dpr = window.devicePixelRatio || 1;

        // Call backend to compute projected frame
        let frameData;
        try {
            const { invoke } = window.__TAURI__.core;
            frameData = await invoke('XplrProjectPts', {
                points: points,
                bboxMin: bboxMin,
                bboxMax: bboxMax,
                angleX: angleX,
                angleY: angleY,
                width: width,
                height: height,
                dpr: dpr
            });
        } catch (err) {
            console.error('Failed to compute frame in Rust:', err);
            // Schedule next frame anyway to keep animating when connection recovers
            const speed = parseFloat((rotSpeedInput && rotSpeedInput.value) || 30) / 1000;
            angleY += speed;
            angleX += speed * 0.5;
            requestAnimationFrame(render);
            return;
        }

        ctx.clearRect(0, 0, width, height);

        // Dark gradient background
        const bgGrad = ctx.createRadialGradient(width / 2, height / 2, 50, width / 2, height / 2, Math.max(width, height) / 1.2);
        bgGrad.addColorStop(0, '#111827');
        bgGrad.addColorStop(1, '#070a12');
        ctx.fillStyle = bgGrad;
        ctx.fillRect(0, 0, width, height);

        // Background grid effect
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.04)';
        ctx.lineWidth = 1;
        const gridSize = 40 * dpr;
        for (let x = 0; x < width; x += gridSize) {
            ctx.beginPath();
            ctx.moveTo(x, 0);
            ctx.lineTo(x, height);
            ctx.stroke();
        }
        for (let y = 0; y < height; y += gridSize) {
            ctx.beginPath();
            ctx.moveTo(0, y);
            ctx.lineTo(width, y);
            ctx.stroke();
        }

        // Draw bounding box wireframe (faint)
        ctx.strokeStyle = 'rgba(0, 243, 255, 0.18)';
        ctx.shadowColor = 'transparent';
        ctx.shadowBlur = 0;
        ctx.lineWidth = 1.5 * dpr;

        frameData.box_lines.forEach(line => {
            ctx.beginPath();
            ctx.moveTo(line.x1, line.y1);
            ctx.lineTo(line.x2, line.y2);
            ctx.stroke();
        });

        // Draw point cloud
        const color = (lineColorInput && lineColorInput.value) ? lineColorInput.value : '#00f3ff';
        // Parse hex color for rgba usage
        const r = parseInt(color.slice(1, 3), 16);
        const g = parseInt(color.slice(3, 5), 16);
        const b = parseInt(color.slice(5, 7), 16);

        ctx.shadowColor = color;
        ctx.shadowBlur = 10 * dpr;

        frameData.points.forEach(p => {
            ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${p.alpha})`;
            ctx.beginPath();
            ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
            ctx.fill();
        });

        // Draw small white core for each point (depth-sorted visual)
        ctx.shadowBlur = 0;
        ctx.fillStyle = 'rgba(255, 255, 255, 0.7)';
        frameData.points.forEach(p => {
            ctx.beginPath();
            ctx.arc(p.x, p.y, p.core_radius, 0, Math.PI * 2);
            ctx.fill();
        });

        // Reset shadow
        ctx.shadowBlur = 0;

        // Draw dimension text overlays at bottom left
        const bboxLabel = `[${bboxMin.map(v => v.toFixed(0)).join(', ')}] → [${bboxMax.map(v => v.toFixed(0)).join(', ')}]`;
        ctx.fillStyle = 'rgba(226, 232, 240, 0.85)';
        ctx.font = `${13 * dpr}px monospace`;
        ctx.fillText(`Points: ${points.length} | BBox: ${bboxLabel}`, 20 * dpr, height - 35 * dpr);
        ctx.fillText('Shader Backend: Rust-GPU (gcomp::pts_pointcloud_cs)', 20 * dpr, height - 15 * dpr);

        // Update rotation angle
        const speed = parseFloat((rotSpeedInput && rotSpeedInput.value) || 30) / 1000;
        angleY += speed;
        angleX += speed * 0.5;

        requestAnimationFrame(render);
    }

    // Initialize: fetch GPU data then start rendering
    fetchPointCloud().then(() => {
        requestAnimationFrame(render);
    });
})();
