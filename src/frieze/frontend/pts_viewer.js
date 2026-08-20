// =================================================================
// Fenst — Rust-GPU Swarm Multi-GPU Point Cloud Display
// Pure Swarm Display Pipeline (Zero WebGL — 100% Swarm Compute Backend)
// =================================================================

(function () {
    const urlParams = new URLSearchParams(window.location.search);
    const filePath = urlParams.get('file') || 'pointcloud.pts';

    const canvas = document.getElementById('pts-canvas');
    if (!canvas) return;
    const ctx = canvas.getContext('2d', { alpha: false });

    const rotSpeedInput = document.getElementById('pts-rot-speed');
    const lineColorInput = document.getElementById('pts-line-color');
    const zoomInput = document.getElementById('pts-zoom');
    const zoomValDisplay = document.getElementById('pts-zoom-val');
    const btnResetCam = document.getElementById('pts-btn-reset-cam');

    // UI Stats elements
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
    let dragMode = 'rotate';
    let lastMouseX = 0;
    let lastMouseY = 0;
    let isInteractive = false;
    let interactiveTimeout = null;

    // Rendering & Frame timing state
    let isRequestPending = false;
    let needsRedraw = true;
    let frameCount = 0;
    let fps = 60;
    let fpsTimer = performance.now();

    function setInteractive() {
        isInteractive = true;
        needsRedraw = true;
        if (interactiveTimeout) clearTimeout(interactiveTimeout);
        interactiveTimeout = setTimeout(() => {
            isInteractive = false;
            needsRedraw = true;
        }, 150);
    }

    function resizeCanvas() {
        const dpr = window.devicePixelRatio || 1;
        const rect = canvas.getBoundingClientRect();
        const displayWidth = Math.round(rect.width * dpr);
        const displayHeight = Math.round(rect.height * dpr);

        if (canvas.width !== displayWidth || canvas.height !== displayHeight) {
            canvas.width = displayWidth;
            canvas.height = displayHeight;
            needsRedraw = true;
        }
    }

    window.addEventListener('resize', resizeCanvas);
    resizeCanvas();

    // Mouse & Touch Controls
    canvas.addEventListener('mousedown', (e) => {
        isDragging = true;
        dragMode = (e.button === 2 || e.shiftKey) ? 'pan' : 'rotate';
        lastMouseX = e.clientX;
        lastMouseY = e.clientY;
        setInteractive();
    });

    window.addEventListener('mousemove', (e) => {
        if (!isDragging) return;
        const dx = e.clientX - lastMouseX;
        const dy = e.clientY - lastMouseY;
        lastMouseX = e.clientX;
        lastMouseY = e.clientY;

        if (dragMode === 'rotate') {
            rotY -= dx * 0.008;
            rotX -= dy * 0.008;
        } else {
            panX += dx * (window.devicePixelRatio || 1);
            panY += dy * (window.devicePixelRatio || 1);
        }
        setInteractive();
    });

    window.addEventListener('mouseup', () => {
        isDragging = false;
    });

    canvas.addEventListener('contextmenu', (e) => e.preventDefault());

    canvas.addEventListener('wheel', (e) => {
        e.preventDefault();
        const zoomDelta = e.deltaY < 0 ? 1.15 : 0.87;
        zoomLevel = Math.max(0.05, Math.min(100.0, zoomLevel * zoomDelta));
        if (zoomInput) zoomInput.value = zoomLevel;
        if (zoomValDisplay) zoomValDisplay.textContent = zoomLevel.toFixed(2) + 'x';
        setInteractive();
    }, { passive: false });

    if (zoomInput) {
        zoomInput.addEventListener('input', (e) => {
            zoomLevel = parseFloat(e.target.value);
            if (zoomValDisplay) zoomValDisplay.textContent = zoomLevel.toFixed(2) + 'x';
            setInteractive();
        });
    }

    if (btnResetCam) {
        btnResetCam.addEventListener('click', async () => {
            try {
                if (window.__TAURI__ && window.__TAURI__.core) {
                    await window.__TAURI__.core.invoke('xplr_reset_camera', { path: filePath });
                }
            } catch (_) {}
            panX = 0.0;
            panY = 0.0;
            zoomLevel = 1.0;
            rotX = 0.4;
            rotY = 0.6;
            if (zoomInput) zoomInput.value = 1.0;
            if (zoomValDisplay) zoomValDisplay.textContent = '1.00x';
            setInteractive();
        });
    }

    // Color parsing helper
    function parseHexColor(hex) {
        const clean = hex.replace('#', '');
        if (clean.length === 6) {
            return {
                r: parseInt(clean.substring(0, 2), 16),
                g: parseInt(clean.substring(2, 4), 16),
                b: parseInt(clean.substring(4, 6), 16),
            };
        }
        return { r: 0, g: 243, b: 255 };
    }

    // Cached draw frame
    let cachedFrame = null;

    async function fetchSwarmFrame() {
        if (isRequestPending) return;
        isRequestPending = true;

        const width = canvas.width;
        const height = canvas.height;
        const dpr = window.devicePixelRatio || 1;
        const speed = parseFloat((rotSpeedInput && rotSpeedInput.value) || 30);
        const color = (lineColorInput && lineColorInput.value) || '#00f3ff';

        try {
            if (window.__TAURI__ && window.__TAURI__.core) {
                const rawFrame = await window.__TAURI__.core.invoke('xplr_project_pts', {
                    path: filePath,
                    width: width,
                    height: height,
                    dpr: dpr,
                    speed: speed,
                    color: color,
                    panX: panX,
                    panY: panY,
                    zoom: zoomLevel,
                    rotX: rotX,
                    rotY: rotY,
                    isInteractive: isInteractive || isDragging,
                });
                const frame = (typeof BinaryDecoder !== 'undefined') ? BinaryDecoder.decodePtsFrame(rawFrame) : rawFrame;
                if (frame) cachedFrame = frame;

                if (frame) {
                    if (titleEl && frame.file_name) titleEl.textContent = frame.file_name;
                    if (statPointCount && frame.count !== undefined) statPointCount.textContent = frame.count.toLocaleString();
                    if (statBbox && frame.bbox_label) statBbox.textContent = frame.bbox_label;
                    if (statShaderStatus && frame.shader_status) statShaderStatus.textContent = frame.shader_status;
                }
            }
        } catch (err) {
            console.error('Swarm frame error:', err);
        } finally {
            isRequestPending = false;
        }
    }

    function render2D() {
        const width = canvas.width;
        const height = canvas.height;
        const dpr = window.devicePixelRatio || 1;

        // Dark background with subtle radial gradient
        const bgGrad = ctx.createRadialGradient(
            width * 0.5, height * 0.5, Math.min(width, height) * 0.1,
            width * 0.5, height * 0.5, Math.max(width, height) * 0.8
        );
        bgGrad.addColorStop(0, '#0d1527');
        bgGrad.addColorStop(1, '#050811');
        ctx.fillStyle = bgGrad;
        ctx.fillRect(0, 0, width, height);

        if (!cachedFrame) return;

        const baseColor = parseHexColor((lineColorInput && lineColorInput.value) || '#00f3ff');
        const points = cachedFrame.points || [];
        const boxLines = cachedFrame.box_lines || [];

        // 1. Draw Bounding Box Wireframe (Swarm GPU Projected)
        if (boxLines.length > 0) {
            ctx.save();
            ctx.strokeStyle = 'rgba(0, 243, 255, 0.45)';
            ctx.lineWidth = 1.5 * dpr;
            ctx.setLineDash([4 * dpr, 4 * dpr]);
            ctx.beginPath();
            for (let i = 0; i < boxLines.length; i++) {
                const line = boxLines[i];
                ctx.moveTo(line.x1, line.y1);
                ctx.lineTo(line.x2, line.y2);
            }
            ctx.stroke();
            ctx.restore();
        }

        // 2. Draw Point Cloud (Swarm Multi-GPU Projected)
        const pointCount = points.length;
        if (pointCount > 0) {
            // Group into alpha buckets (0.1 to 1.0) for high-efficiency batch draws
            const BUCKETS = 10;
            const haloPaths = new Array(BUCKETS);
            const corePaths = new Array(BUCKETS);
            for (let b = 0; b < BUCKETS; b++) {
                haloPaths[b] = new Path2D();
                corePaths[b] = new Path2D();
            }

            for (let i = 0; i < pointCount; i++) {
                const pt = points[i];
                const px = pt.x;
                const py = pt.y;

                // Frustum culling check in 2D viewport
                if (px < -50 || px > width + 50 || py < -50 || py > height + 50) continue;

                const alpha = Math.max(0.05, Math.min(1.0, pt.alpha || 0.8));
                const bIdx = Math.min(BUCKETS - 1, Math.floor(alpha * BUCKETS));

                const r = pt.radius || (4.0 * dpr);
                const cr = pt.core_radius || (1.5 * dpr);

                haloPaths[bIdx].moveTo(px + r, py);
                haloPaths[bIdx].arc(px, py, r, 0, 6.2831853);

                corePaths[bIdx].moveTo(px + cr, py);
                corePaths[bIdx].arc(px, py, cr, 0, 6.2831853);
            }

            // Draw outer halos
            for (let b = 0; b < BUCKETS; b++) {
                const alphaVal = ((b + 1) / BUCKETS) * 0.45;
                ctx.fillStyle = `rgba(${baseColor.r}, ${baseColor.g}, ${baseColor.b}, ${alphaVal.toFixed(3)})`;
                ctx.fill(haloPaths[b]);
            }

            // Draw bright glowing cores
            for (let b = 0; b < BUCKETS; b++) {
                const alphaVal = Math.min(1.0, ((b + 1) / BUCKETS) * 0.95);
                ctx.fillStyle = `rgba(255, 255, 255, ${alphaVal.toFixed(3)})`;
                ctx.fill(corePaths[b]);
            }
        }

        // 3. Draw HUD Overlay
        ctx.save();
        ctx.font = `${11 * dpr}px monospace`;
        ctx.fillStyle = 'rgba(0, 243, 255, 0.85)';
        if (cachedFrame.overlay_text1) {
            ctx.fillText(cachedFrame.overlay_text1, 16 * dpr, height - 32 * dpr);
        }
        ctx.fillStyle = 'rgba(148, 163, 184, 0.85)';
        if (cachedFrame.overlay_text2) {
            ctx.fillText(`${cachedFrame.overlay_text2} | Display: Swarm 2D (${fps} FPS)`, 16 * dpr, height - 16 * dpr);
        }
        ctx.restore();
    }

    function mainLoop(now) {
        // Calculate FPS
        frameCount++;
        if (now - fpsTimer >= 500) {
            fps = Math.round((frameCount * 1000) / (now - fpsTimer));
            frameCount = 0;
            fpsTimer = now;
        }

        const speed = parseFloat((rotSpeedInput && rotSpeedInput.value) || 30);
        if (speed > 0 || isInteractive || isDragging || needsRedraw) {
            fetchSwarmFrame();
            needsRedraw = false;
        }

        render2D();
        requestAnimationFrame(mainLoop);
    }

    requestAnimationFrame(mainLoop);
})();
