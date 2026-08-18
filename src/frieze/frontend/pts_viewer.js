// =================================================================
// Fenst — Rust-GPU Point Cloud Viewer (.pts files)
// SceneGraph Camera: Pan, Zoom, Rotate, and GPU Point Projection
// =================================================================

(function () {
    const urlParams = new URLSearchParams(window.location.search);
    const filePath = urlParams.get('file') || 'pointcloud.pts';

    const canvas = document.getElementById('pts-canvas');
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    const rotSpeedInput = document.getElementById('pts-rot-speed');
    const lineColorInput = document.getElementById('pts-line-color');
    const zoomInput = document.getElementById('pts-zoom');
    const zoomValDisplay = document.getElementById('pts-zoom-val');
    const btnResetCam = document.getElementById('pts-btn-reset-cam');

    // Stats elements for updating after GPU data arrives
    const titleEl = document.getElementById('pts-filename-txt');
    const statPointCount = document.getElementById('pts-stat-count');
    const statBbox = document.getElementById('pts-stat-bbox');
    const statShaderStatus = document.getElementById('pts-shader-status');

    // Camera state
    let panX = 0.0;
    let panY = 0.0;
    let zoomLevel = 1.0;
    let rotX = 0.4;
    let rotY = 0.6;

    // Interaction state
    let isDragging = false;
    let dragMode = 'rotate'; // 'rotate' (left button) vs 'pan' (right button or shift key)
    let lastMouseX = 0;
    let lastMouseY = 0;
    let isInteractive = false;
    let interactiveTimer = null;

    function markInteractive() {
        isInteractive = true;
        if (interactiveTimer) clearTimeout(interactiveTimer);
        interactiveTimer = setTimeout(() => {
            if (!isDragging) {
                isInteractive = false;
            }
        }, 500);
    }

    function updateZoomDisplay() {
        if (zoomValDisplay) {
            zoomValDisplay.textContent = zoomLevel.toFixed(2) + 'x';
        }
        if (zoomInput && document.activeElement !== zoomInput) {
            zoomInput.value = Math.round(zoomLevel * 100);
        }
    }

    // Zoom slider handler
    if (zoomInput) {
        zoomInput.addEventListener('input', () => {
            zoomLevel = Math.max(0.05, Math.min(50.0, parseFloat(zoomInput.value) / 100.0));
            updateZoomDisplay();
            markInteractive();
        });
    }

    // Reset camera button
    if (btnResetCam) {
        btnResetCam.addEventListener('click', async () => {
            panX = 0.0;
            panY = 0.0;
            zoomLevel = 1.0;
            rotX = 0.4;
            rotY = 0.6;
            updateZoomDisplay();
            markInteractive();
            try {
                const { invoke } = window.__TAURI__.core;
                await invoke('XplrResetCamera', { path: filePath });
            } catch (err) {
                console.error('Failed to reset camera:', err);
            }
        });
    }

    // Canvas mouse events for Orbit (Rotate) and Pan
    canvas.addEventListener('contextmenu', (e) => e.preventDefault());

    canvas.addEventListener('mousedown', (e) => {
        isDragging = true;
        dragMode = (e.button === 2 || e.shiftKey || e.button === 1) ? 'pan' : 'rotate';
        lastMouseX = e.clientX;
        lastMouseY = e.clientY;
        canvas.style.cursor = dragMode === 'pan' ? 'move' : 'grabbing';
        markInteractive();
    });

    window.addEventListener('mousemove', (e) => {
        if (!isDragging) return;
        const dx = e.clientX - lastMouseX;
        const dy = e.clientY - lastMouseY;
        lastMouseX = e.clientX;
        lastMouseY = e.clientY;

        if (dragMode === 'rotate') {
            rotY += dx * 0.008;
            rotX += dy * 0.008;
        } else {
            panX += dx;
            panY += dy;
        }
        markInteractive();
    });

    window.addEventListener('mouseup', () => {
        if (isDragging) {
            isDragging = false;
            canvas.style.cursor = 'grab';
            markInteractive();
        }
    });

    canvas.style.cursor = 'grab';

    // Mouse wheel for Zoom
    canvas.addEventListener('wheel', (e) => {
        e.preventDefault();
        const factor = e.deltaY < 0 ? 1.12 : 0.89;
        zoomLevel = Math.max(0.05, Math.min(50.0, zoomLevel * factor));
        updateZoomDisplay();
        markInteractive();
    }, { passive: false });

    // Double click to reset view
    canvas.addEventListener('dblclick', async () => {
        panX = 0.0;
        panY = 0.0;
        zoomLevel = 1.0;
        rotX = 0.4;
        rotY = 0.6;
        updateZoomDisplay();
        markInteractive();
        try {
            const { invoke } = window.__TAURI__.core;
            await invoke('XplrResetCamera', { path: filePath });
        } catch (err) {
            console.error('Failed to reset camera:', err);
        }
    });

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

        const color = (lineColorInput && lineColorInput.value) ? lineColorInput.value : '#00f3ff';
        const speed = parseFloat((rotSpeedInput && rotSpeedInput.value) || 30);

        // Call backend SceneGraph to compute projected frame
        let frameData;
        try {
            const { invoke } = window.__TAURI__.core;
            const rawFrame = await invoke('XplrProjectPts', {
                path: filePath,
                width: width,
                height: height,
                dpr: dpr,
                speed: speed,
                color: color,
                pan_x: panX,
                pan_y: panY,
                zoom: zoomLevel,
                rot_x: rotX,
                rot_y: rotY,
                is_interactive: isDragging || isInteractive
            });
            frameData = {
                file_name: rawFrame.file_name ?? rawFrame._file_name ?? 'pointcloud.pts',
                count: rawFrame.count ?? rawFrame._count ?? 0,
                bbox_label: rawFrame.bbox_label ?? rawFrame._bbox_label ?? '—',
                shader_status: rawFrame.shader_status ?? rawFrame._shader_status ?? '',
                box_lines: (rawFrame.box_lines ?? rawFrame._box_lines ?? []).map(l => ({
                    x1: l.x1 ?? l._x1 ?? 0,
                    y1: l.y1 ?? l._y1 ?? 0,
                    x2: l.x2 ?? l._x2 ?? 0,
                    y2: l.y2 ?? l._y2 ?? 0,
                })),
                points: (rawFrame.points ?? rawFrame._points ?? []).map(p => ({
                    x: p.x ?? p._x ?? 0,
                    y: p.y ?? p._y ?? 0,
                    radius: p.radius ?? p._radius ?? 3,
                    core_radius: p.core_radius ?? p._core_radius ?? 1,
                    color: p.color ?? p._color ?? '#00f3ff',
                })),
                overlay_text1: rawFrame.overlay_text1 ?? rawFrame._overlay_text1 ?? '',
                overlay_text2: rawFrame.overlay_text2 ?? rawFrame._overlay_text2 ?? '',
            };
        } catch (err) {
            console.error('Failed to compute frame in Rust:', err);
            requestAnimationFrame(render);
            return;
        }

        // Update UI panels with backend pre-formatted strings
        if (titleEl) titleEl.textContent = frameData.file_name;
        if (statPointCount) statPointCount.textContent = frameData.count;
        if (statBbox) statBbox.textContent = frameData.bbox_label;
        if (statShaderStatus) statShaderStatus.textContent = frameData.shader_status;

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
        ctx.strokeStyle = 'rgba(0, 243, 255, 0.2)';
        ctx.shadowColor = 'transparent';
        ctx.shadowBlur = 0;
        ctx.lineWidth = 1.5 * dpr;

        frameData.box_lines.forEach(line => {
            ctx.beginPath();
            ctx.moveTo(line.x1, line.y1);
            ctx.lineTo(line.x2, line.y2);
            ctx.stroke();
        });

        // Draw point cloud (outer halo)
        const ptsLen = frameData.points.length;
        for (let i = 0; i < ptsLen; i++) {
            const p = frameData.points[i];
            ctx.fillStyle = p.color;
            ctx.beginPath();
            ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
            ctx.fill();
        }

        // Draw glowing white cores (depth-sorted visual)
        ctx.fillStyle = 'rgba(255, 255, 255, 0.75)';
        ctx.beginPath();
        for (let i = 0; i < ptsLen; i++) {
            const p = frameData.points[i];
            ctx.moveTo(p.x + p.core_radius, p.y);
            ctx.arc(p.x, p.y, p.core_radius, 0, Math.PI * 2);
        }
        ctx.fill();

        // Draw HUD overlays
        ctx.fillStyle = 'rgba(226, 232, 240, 0.85)';
        ctx.font = `${13 * dpr}px monospace`;
        ctx.fillText(frameData.overlay_text1, 20 * dpr, height - 35 * dpr);
        ctx.fillText(frameData.overlay_text2, 20 * dpr, height - 15 * dpr);

        requestAnimationFrame(render);
    }

    // Start rendering loop immediately
    requestAnimationFrame(render);
})();
