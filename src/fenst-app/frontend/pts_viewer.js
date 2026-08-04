// =================================================================
// Fenst — Rust-GPU Wireframe Graphics Viewer (.pts files)
// Renders 100x100x100 3D wireframe block from gcomp shader pipeline
// =================================================================

(function () {
    const urlParams = new URLSearchParams(window.location.search);
    const filePath = urlParams.get('file') || 'block.pts';
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

    // 100x100x100 Block Vertices (Extents -50 to +50 along X, Y, Z)
    const vertices = [
        [-50, -50, -50],
        [ 50, -50, -50],
        [ 50,  50, -50],
        [-50,  50, -50],
        [-50, -50,  50],
        [ 50, -50,  50],
        [ 50,  50,  50],
        [-50,  50,  50],
    ];

    // 12 Line Edges connecting the 8 block vertices (24 indices)
    const edges = [
        [0, 1], [1, 2], [2, 3], [3, 0], // Bottom face
        [4, 5], [5, 6], [6, 7], [7, 4], // Top face
        [0, 4], [1, 5], [2, 6], [3, 7]  // Vertical pillars
    ];

    let angleX = 0.4;
    let angleY = 0.6;

    function resizeCanvas() {
        const dpr = window.devicePixelRatio || 1;
        const w = canvas.clientWidth || canvas.parentElement?.clientWidth || window.innerWidth || 960;
        const h = canvas.clientHeight || canvas.parentElement?.clientHeight || (window.innerHeight - 48) || 672;
        canvas.width = Math.max(w * dpr, 300);
        canvas.height = Math.max(h * dpr, 300);
    }

    window.addEventListener('resize', resizeCanvas);
    resizeCanvas();

    function project(x, y, z, width, height) {
        // Rotate around Y axis
        const cosY = Math.cos(angleY);
        const sinY = Math.sin(angleY);
        const x1 = x * cosY + z * sinY;
        const z1 = -x * sinY + z * cosY;

        // Rotate around X axis
        const cosX = Math.cos(angleX);
        const sinX = Math.sin(angleX);
        const y2 = y * cosX - z1 * sinX;
        const z2 = y * sinX + z1 * cosX;

        // Perspective projection
        const fov = 350;
        const distance = 250;
        const scale = fov / (distance + z2);

        const projX = width / 2 + x1 * scale;
        const projY = height / 2 - y2 * scale;

        return { x: projX, y: projY, scale, z: z2 };
    }

    function render() {
        resizeCanvas();
        const width = canvas.width;
        const height = canvas.height;
        const dpr = window.devicePixelRatio || 1;

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

        // Project 8 corner vertices
        const projected = vertices.map(v => project(v[0], v[1], v[2], width, height));

        // Draw wireframe edges
        const color = (lineColorInput && lineColorInput.value) ? lineColorInput.value : '#00f3ff';
        ctx.strokeStyle = color;
        ctx.shadowColor = color;
        ctx.shadowBlur = 12 * dpr;
        ctx.lineWidth = 2.5 * dpr;

        edges.forEach(([i, j]) => {
            const p1 = projected[i];
            const p2 = projected[j];
            ctx.beginPath();
            ctx.moveTo(p1.x, p1.y);
            ctx.lineTo(p2.x, p2.y);
            ctx.stroke();
        });

        // Draw corner vertex points
        ctx.shadowBlur = 16 * dpr;
        ctx.fillStyle = '#ffffff';
        projected.forEach(p => {
            ctx.beginPath();
            ctx.arc(p.x, p.y, 4.5 * dpr, 0, Math.PI * 2);
            ctx.fill();
        });

        // Reset shadow
        ctx.shadowBlur = 0;

        // Draw dimension text overlays at bottom left
        ctx.fillStyle = 'rgba(226, 232, 240, 0.85)';
        ctx.font = `${13 * dpr}px monospace`;
        ctx.fillText('Dimension: 100 x 100 x 100', 20 * dpr, height - 35 * dpr);
        ctx.fillText('Shader Backend: Rust-GPU (gcomp)', 20 * dpr, height - 15 * dpr);

        // Update rotation angle
        const speed = parseFloat((rotSpeedInput && rotSpeedInput.value) || 30) / 1000;
        angleY += speed;
        angleX += speed * 0.5;

        requestAnimationFrame(render);
    }

    requestAnimationFrame(render);
})();
