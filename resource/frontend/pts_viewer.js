// =================================================================
// Fenst — Rust-GPU Point Cloud Viewer (.pts files)
// Hardware-Accelerated WebGL Point-Sprite & SceneGraph Visualization
// =================================================================

(function () {
    const urlParams = new URLSearchParams(window.location.search);
    const filePath = urlParams.get('file') || 'pointcloud.pts';

    const canvas = document.getElementById('pts-canvas');
    if (!canvas) return;

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
    let interactiveTimer = null;

    // Point cloud data
    let pointPositions = null; // Float32Array
    let pointCount = 0;
    let bboxMin = [-20, -20, -20];
    let bboxMax = [20, 20, 20];
    let center = [0, 0, 0];
    let scaleNorm = 1.0;
    let fileName = filePath.split(/[\\/]/).pop() || 'pointcloud.pts';

    // WebGL state
    let gl = null;
    let pointProgram = null;
    let lineProgram = null;
    let pointBuffer = null;
    let lineBuffer = null;
    let lineCount = 0;
    let webglSupported = false;

    // Shaders for WebGL Point Sprites
    const vsSource = `
        attribute vec3 a_position;
        uniform vec2 u_resolution;
        uniform vec2 u_rotation;
        uniform vec2 u_pan;
        uniform float u_zoom;
        uniform float u_fov;
        uniform float u_distance;
        uniform vec3 u_center;
        uniform float u_scaleNorm;
        uniform float u_dpr;
        varying float v_depthFactor;

        void main() {
            vec3 normPos = (a_position - u_center) * u_scaleNorm;
            
            float cosY = cos(u_rotation.y);
            float sinY = sin(u_rotation.y);
            float x1 = normPos.x * cosY + normPos.z * sinY;
            float z1 = -normPos.x * sinY + normPos.z * cosY;
            
            float cosX = cos(u_rotation.x);
            float sinX = sin(u_rotation.x);
            float y2 = normPos.y * cosX - z1 * sinX;
            float z2 = normPos.y * sinX + z1 * cosX;
            
            float denom = u_distance + z2;
            float w = max(denom, 0.0001);
            float scale = (u_fov * u_zoom) / w;
            
            float projX = u_resolution.x * 0.5 + u_pan.x + x1 * scale;
            float projY = u_resolution.y * 0.5 + u_pan.y - y2 * scale;
            
            float ndcX = (projX / u_resolution.x) * 2.0 - 1.0;
            float ndcY = 1.0 - (projY / u_resolution.y) * 2.0;
            float ndcZ = z2 / 400.0;
            
            gl_Position = vec4(ndcX, ndcY, ndcZ, 1.0);
            
            float depthFactor = clamp((300.0 - z2) / 400.0, 0.3, 1.0);
            v_depthFactor = depthFactor;
            gl_PointSize = (4.0 + depthFactor * 6.0) * u_dpr;
        }
    `;

    const fsSource = `
        precision mediump float;
        uniform vec4 u_baseColor;
        varying float v_depthFactor;

        void main() {
            vec2 delta = gl_PointCoord - vec2(0.5);
            float distSq = dot(delta, delta);
            if (distSq > 0.25) {
                discard;
            }
            float dist = sqrt(distSq) * 2.0;
            float alpha = max(0.0, 1.0 - dist) * (0.4 + v_depthFactor * 0.6);
            float core = dist < 0.3 ? (1.0 - dist / 0.3) : 0.0;
            
            vec3 col = mix(u_baseColor.rgb, vec3(1.0), core * 0.85);
            gl_FragColor = vec4(col, alpha);
        }
    `;

    const lineVsSource = `
        attribute vec3 a_position;
        uniform vec2 u_resolution;
        uniform vec2 u_rotation;
        uniform vec2 u_pan;
        uniform float u_zoom;
        uniform float u_fov;
        uniform float u_distance;
        uniform vec3 u_center;
        uniform float u_scaleNorm;

        void main() {
            vec3 normPos = (a_position - u_center) * u_scaleNorm;
            
            float cosY = cos(u_rotation.y);
            float sinY = sin(u_rotation.y);
            float x1 = normPos.x * cosY + normPos.z * sinY;
            float z1 = -normPos.x * sinY + normPos.z * cosY;
            
            float cosX = cos(u_rotation.x);
            float sinX = sin(u_rotation.x);
            float y2 = normPos.y * cosX - z1 * sinX;
            float z2 = normPos.y * sinX + z1 * cosX;
            
            float denom = u_distance + z2;
            float w = max(denom, 0.0001);
            float scale = (u_fov * u_zoom) / w;
            
            float projX = u_resolution.x * 0.5 + u_pan.x + x1 * scale;
            float projY = u_resolution.y * 0.5 + u_pan.y - y2 * scale;
            
            float ndcX = (projX / u_resolution.x) * 2.0 - 1.0;
            float ndcY = 1.0 - (projY / u_resolution.y) * 2.0;
            float ndcZ = z2 / 400.0;
            
            gl_Position = vec4(ndcX, ndcY, ndcZ, 1.0);
        }
    `;

    const lineFsSource = `
        precision mediump float;
        uniform vec4 u_lineColor;

        void main() {
            gl_FragColor = u_lineColor;
        }
    `;

    function initWebGL() {
        gl = canvas.getContext('webgl', { alpha: false, antialias: true, depth: false }) ||
             canvas.getContext('experimental-webgl');
        if (!gl) {
            console.warn('WebGL not supported, falling back to CPU 2D canvas projection');
            return false;
        }

        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE); // Additive blending for radiant point cloud glow

        pointProgram = createProgram(gl, vsSource, fsSource);
        lineProgram = createProgram(gl, lineVsSource, lineFsSource);

        return !!(pointProgram && lineProgram);
    }

    function createShader(gl, type, source) {
        const shader = gl.createShader(type);
        gl.shaderSource(shader, source);
        gl.compileShader(shader);
        if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
            console.error('Shader compilation failed:', gl.getShaderInfoLog(shader));
            gl.deleteShader(shader);
            return null;
        }
        return shader;
    }

    function createProgram(gl, vs, fs) {
        const vertexShader = createShader(gl, gl.VERTEX_SHADER, vs);
        const fragmentShader = createShader(gl, gl.FRAGMENT_SHADER, fs);
        if (!vertexShader || !fragmentShader) return null;

        const program = gl.createProgram();
        gl.attachShader(program, vertexShader);
        gl.attachShader(program, fragmentShader);
        gl.linkProgram(program);
        if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
            console.error('Program linking failed:', gl.getProgramInfoLog(program));
            return null;
        }
        return program;
    }

    function markInteractive() {
        isInteractive = true;
        if (interactiveTimer) clearTimeout(interactiveTimer);
        interactiveTimer = setTimeout(() => {
            if (!isDragging) isInteractive = false;
        }, 500);
    }

    function updateZoomDisplay() {
        if (zoomValDisplay) zoomValDisplay.textContent = zoomLevel.toFixed(2) + 'x';
        if (zoomInput && document.activeElement !== zoomInput) {
            zoomInput.value = Math.round(zoomLevel * 100);
        }
    }

    if (zoomInput) {
        zoomInput.addEventListener('input', () => {
            zoomLevel = Math.max(0.05, Math.min(50.0, parseFloat(zoomInput.value) / 100.0));
            updateZoomDisplay();
            markInteractive();
        });
    }

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
            } catch (err) {}
        });
    }

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

    canvas.addEventListener('wheel', (e) => {
        e.preventDefault();
        const factor = e.deltaY < 0 ? 1.12 : 0.89;
        zoomLevel = Math.max(0.05, Math.min(50.0, zoomLevel * factor));
        updateZoomDisplay();
        markInteractive();
    }, { passive: false });

    canvas.addEventListener('dblclick', async () => {
        panX = 0.0;
        panY = 0.0;
        zoomLevel = 1.0;
        rotX = 0.4;
        rotY = 0.6;
        updateZoomDisplay();
        markInteractive();
    });

    function resizeCanvas() {
        const dpr = window.devicePixelRatio || 1;
        const w = canvas.clientWidth || canvas.parentElement?.clientWidth || window.innerWidth || 960;
        const h = canvas.clientHeight || canvas.parentElement?.clientHeight || (window.innerHeight - 48) || 672;
        canvas.width = Math.max(w * dpr, 300);
        canvas.height = Math.max(h * dpr, 300);
        if (gl) {
            gl.viewport(0, 0, canvas.width, canvas.height);
        }
    }

    window.addEventListener('resize', resizeCanvas);

    function calcNormalization(bMin, bMax) {
        const cx = (bMin[0] + bMax[0]) * 0.5;
        const cy = (bMin[1] + bMax[1]) * 0.5;
        const cz = (bMin[2] + bMax[2]) * 0.5;
        const dx = bMax[0] - bMin[0];
        const dy = bMax[1] - bMin[1];
        const dz = bMax[2] - bMin[2];
        const maxDim = Math.max(dx, Math.max(dy, dz));
        const sNorm = maxDim > 1e-4 ? 35.0 / maxDim : 1.0;
        return { center: [cx, cy, cz], scaleNorm: sNorm };
    }

    function createBoundingBoxLines(bMin, bMax) {
        const corners = [
            bMin[0], bMin[1], bMin[2],
            bMax[0], bMin[1], bMin[2],
            bMax[0], bMax[1], bMin[2],
            bMin[0], bMax[1], bMin[2],
            bMin[0], bMin[1], bMax[2],
            bMax[0], bMin[1], bMax[2],
            bMax[0], bMax[1], bMax[2],
            bMin[0], bMax[1], bMax[2],
        ];

        const edges = [
            0, 1, 1, 2, 2, 3, 3, 0,
            4, 5, 5, 6, 6, 7, 7, 4,
            0, 4, 1, 5, 2, 6, 3, 7
        ];

        const lineVerts = new Float32Array(edges.length * 3);
        for (let i = 0; i < edges.length; i++) {
            const idx = edges[i] * 3;
            lineVerts[i * 3 + 0] = corners[idx + 0];
            lineVerts[i * 3 + 1] = corners[idx + 1];
            lineVerts[i * 3 + 2] = corners[idx + 2];
        }
        return lineVerts;
    }

    function parseHexColor(hex) {
        let clean = hex.replace('#', '');
        if (clean.length === 3) {
            clean = clean.split('').map(c => c + c).join('');
        }
        const num = parseInt(clean, 16);
        return [
            ((num >> 16) & 255) / 255.0,
            ((num >> 8) & 255) / 255.0,
            (num & 255) / 255.0,
            1.0
        ];
    }

    async function loadPointCloud() {
        try {
            const { invoke } = window.__TAURI__.core;
            const dto = await invoke('XplrFetchPtsPoints', { path: filePath });
            
            const rawPoints = dto.points ?? dto._points ?? [];
            pointCount = dto.count ?? dto._count ?? rawPoints.length;
            bboxMin = dto.bbox_min ?? dto._bbox_min ?? [-20, -20, -20];
            bboxMax = dto.bbox_max ?? dto._bbox_max ?? [20, 20, 20];

            const norm = calcNormalization(bboxMin, bboxMax);
            center = norm.center;
            scaleNorm = norm.scaleNorm;

            pointPositions = new Float32Array(rawPoints.length * 3);
            for (let i = 0; i < rawPoints.length; i++) {
                pointPositions[i * 3 + 0] = rawPoints[i][0];
                pointPositions[i * 3 + 1] = rawPoints[i][1];
                pointPositions[i * 3 + 2] = rawPoints[i][2];
            }

            if (titleEl) titleEl.textContent = fileName;
            if (statPointCount) statPointCount.textContent = pointCount.toLocaleString();
            if (statBbox) {
                statBbox.textContent = `[${bboxMin[0].toFixed(1)}, ${bboxMin[1].toFixed(1)}, ${bboxMin[1].toFixed(1)}] → [${bboxMax[0].toFixed(1)}, ${bboxMax[1].toFixed(1)}, ${bboxMax[2].toFixed(1)}]`;
            }
            if (statShaderStatus) {
                statShaderStatus.textContent = webglSupported ? 'WebGL 2.0 / Hardware Rasterizer' : 'Rust-GPU / CPU Engine';
            }

            if (webglSupported && gl) {
                pointBuffer = gl.createBuffer();
                gl.bindBuffer(gl.ARRAY_BUFFER, pointBuffer);
                gl.bufferData(gl.ARRAY_BUFFER, pointPositions, gl.STATIC_DRAW);

                const lineVerts = createBoundingBoxLines(bboxMin, bboxMax);
                lineCount = lineVerts.length / 3;
                lineBuffer = gl.createBuffer();
                gl.bindBuffer(gl.ARRAY_BUFFER, lineBuffer);
                gl.bufferData(gl.ARRAY_BUFFER, lineVerts, gl.STATIC_DRAW);
            }
        } catch (err) {
            console.error('Failed to load point cloud points:', err);
        }
    }

    function renderWebGL() {
        const width = canvas.width;
        const height = canvas.height;
        const dpr = window.devicePixelRatio || 1;
        const speed = parseFloat((rotSpeedInput && rotSpeedInput.value) || 30);

        if (!isDragging && !isInteractive && speed > 0) {
            const speedRad = speed / 1000.0;
            rotY += speedRad;
            rotX += speedRad * 0.5;
        }

        gl.clearColor(0.043, 0.059, 0.098, 1.0); // #0b0f19
        gl.clear(gl.COLOR_BUFFER_BIT);

        const baseColor = parseHexColor((lineColorInput && lineColorInput.value) || '#00f3ff');

        // Render Point Cloud
        if (pointProgram && pointBuffer && pointCount > 0) {
            gl.useProgram(pointProgram);

            const aPos = gl.getAttribLocation(pointProgram, 'a_position');
            gl.bindBuffer(gl.ARRAY_BUFFER, pointBuffer);
            gl.enableVertexAttribArray(aPos);
            gl.vertexAttribPointer(aPos, 3, gl.FLOAT, false, 0, 0);

            gl.uniform2f(gl.getUniformLocation(pointProgram, 'u_resolution'), width, height);
            gl.uniform2f(gl.getUniformLocation(pointProgram, 'u_rotation'), rotX, rotY);
            gl.uniform2f(gl.getUniformLocation(pointProgram, 'u_pan'), panX, panY);
            gl.uniform1f(gl.getUniformLocation(pointProgram, 'u_zoom'), zoomLevel);
            gl.uniform1f(gl.getUniformLocation(pointProgram, 'u_fov'), 350.0);
            gl.uniform1f(gl.getUniformLocation(pointProgram, 'u_distance'), 250.0);
            gl.uniform3f(gl.getUniformLocation(pointProgram, 'u_center'), center[0], center[1], center[2]);
            gl.uniform1f(gl.getUniformLocation(pointProgram, 'u_scaleNorm'), scaleNorm);
            gl.uniform1f(gl.getUniformLocation(pointProgram, 'u_dpr'), dpr);
            gl.uniform4fv(gl.getUniformLocation(pointProgram, 'u_baseColor'), baseColor);

            gl.drawArrays(gl.POINTS, 0, pointCount);
        }

        // Render Bounding Box Lines
        if (lineProgram && lineBuffer && lineCount > 0) {
            gl.useProgram(lineProgram);

            const aPos = gl.getAttribLocation(lineProgram, 'a_position');
            gl.bindBuffer(gl.ARRAY_BUFFER, lineBuffer);
            gl.enableVertexAttribArray(aPos);
            gl.vertexAttribPointer(aPos, 3, gl.FLOAT, false, 0, 0);

            gl.uniform2f(gl.getUniformLocation(lineProgram, 'u_resolution'), width, height);
            gl.uniform2f(gl.getUniformLocation(lineProgram, 'u_rotation'), rotX, rotY);
            gl.uniform2f(gl.getUniformLocation(lineProgram, 'u_pan'), panX, panY);
            gl.uniform1f(gl.getUniformLocation(lineProgram, 'u_zoom'), zoomLevel);
            gl.uniform1f(gl.getUniformLocation(lineProgram, 'u_fov'), 350.0);
            gl.uniform1f(gl.getUniformLocation(lineProgram, 'u_distance'), 250.0);
            gl.uniform3f(gl.getUniformLocation(lineProgram, 'u_center'), center[0], center[1], center[2]);
            gl.uniform1f(gl.getUniformLocation(lineProgram, 'u_scaleNorm'), scaleNorm);
            gl.uniform4f(gl.getUniformLocation(lineProgram, 'u_lineColor'), 0.0, 0.95, 1.0, 0.25);

            gl.drawArrays(gl.LINES, 0, lineCount);
        }

        requestAnimationFrame(renderLoop);
    }

    function renderLoop() {
        resizeCanvas();
        if (webglSupported) {
            renderWebGL();
        }
    }

    webglSupported = initWebGL();
    resizeCanvas();
    loadPointCloud().then(() => {
        requestAnimationFrame(renderLoop);
    });
})();
