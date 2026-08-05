// =================================================================
// Fenst — Frontend Application Logic
// =================================================================

// ---- Tauri IPC Bridge ----
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// ---- State ----
const state = {
    rootPath: null,
    currentFile: null,
    explorerVisible: true,
    toolbarVisible: true,
    wordWrap: false,
    theme: 'dark',
    reuseWindow: true,
    openTabs: [],
    activeTabId: null,
    expandedDirs: new Set(),
    selectedTreeItem: null,
};

// ---- DOM References ----
const dom = {
    toolbar:            document.getElementById('toolbar'),
    explorer:           document.getElementById('explorer'),
    explorerTree:       document.getElementById('explorer-tree'),
    content:            document.getElementById('content'),
    contentWelcome:     document.getElementById('content-welcome'),
    contentViewer:      document.getElementById('content-viewer'),
    fileContent:        document.getElementById('file-content'),
    tabBar:             document.getElementById('content-tab-bar'),
    statusFilePath:     document.getElementById('status-file-path'),
    statusLines:        document.getElementById('status-lines'),
    statusSize:         document.getElementById('status-size'),
    resizeHandle:       document.getElementById('resize-handle'),
    btnOpenFolder:      document.getElementById('btn-open-folder'),
    btnOpenFolderEmpty: document.getElementById('btn-open-folder-empty'),
    btnRefresh:         document.getElementById('btn-refresh'),
    btnToggleExplorer:  document.getElementById('btn-toggle-explorer'),
    btnToggleWordWrap:  document.getElementById('btn-toggle-word-wrap'),
    btnToggleTheme:     document.getElementById('btn-toggle-theme'),
    btnToggleReuseWindow: document.getElementById('btn-toggle-reuse-window'),
};

// ---- File Icons (SVG) ----
const icons = {
    chevronRight: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>`,
    folder: `<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/></svg>`,
    folderOpen: `<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M20 6h-8l-2-2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm0 12H4V8h16v10z"/></svg>`,
    file: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>`,
};

// ---- File Extension to Icon Class Map ----
function getFileIconClass(extension) {
    const map = {
        'rs': 'icon-rust',
        'js': 'icon-js',
        'ts': 'icon-ts',
        'tsx': 'icon-ts',
        'jsx': 'icon-js',
        'html': 'icon-html',
        'htm': 'icon-html',
        'css': 'icon-css',
        'scss': 'icon-css',
        'json': 'icon-json',
        'md': 'icon-md',
        'markdown': 'icon-md',
        'toml': 'icon-toml',
        'yaml': 'icon-toml',
        'yml': 'icon-toml',
        'py': 'icon-py',
        'txt': 'icon-txt',
        'log': 'icon-txt',
        'png': 'icon-image',
        'jpg': 'icon-image',
        'jpeg': 'icon-image',
        'gif': 'icon-image',
        'svg': 'icon-image',
        'webp': 'icon-image',
    };
    return map[extension?.toLowerCase()] || 'icon-file';
}

// ---- Utility: Format File Size ----
function formatSize(bytes) {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
}

// ---- Explorer Tree ----

async function loadDirectory(path) {
    try {
        const entries = await invoke('XplrListEntries', { path });
        return entries;
    } catch (err) {
        console.error('Failed to load directory:', err);
        return [];
    }
}

function createTreeItem(entry, depth) {
    const item = document.createElement('div');
    item.className = 'tree-item';
    item.dataset.path = entry.path;
    item.dataset.isDir = entry.is_dir;
    item.style.paddingLeft = (12 + depth * 16) + 'px';

    // Chevron (for directories) or spacer (for files)
    const chevron = document.createElement('span');
    chevron.className = 'tree-item-chevron';
    if (entry.is_dir) {
        chevron.innerHTML = icons.chevronRight;
        if (state.expandedDirs.has(entry.path)) {
            chevron.classList.add('expanded');
        }
    } else {
        chevron.className = 'tree-item-chevron file-spacer';
    }
    item.appendChild(chevron);

    // Icon
    const icon = document.createElement('span');
    icon.className = 'tree-item-icon';
    if (entry.is_dir) {
        icon.className += state.expandedDirs.has(entry.path)
            ? ' icon-folder-open'
            : ' icon-folder';
        icon.innerHTML = state.expandedDirs.has(entry.path)
            ? icons.folderOpen
            : icons.folder;
    } else {
        icon.className += ' ' + getFileIconClass(entry.extension);
        icon.innerHTML = icons.file;
    }
    item.appendChild(icon);

    // Name
    const name = document.createElement('span');
    name.className = 'tree-item-name';
    name.textContent = entry.name;
    item.appendChild(name);

    // Click handler
    item.addEventListener('click', (e) => {
        e.stopPropagation();
        if (entry.is_dir) {
            toggleDirectory(entry, item, depth);
        } else {
            selectFile(entry, item);
        }
    });

    return item;
}

async function toggleDirectory(entry, treeItem, depth) {
    const isExpanded = state.expandedDirs.has(entry.path);

    // Update chevron and icon
    const chevron = treeItem.querySelector('.tree-item-chevron');
    const icon = treeItem.querySelector('.tree-item-icon');

    if (isExpanded) {
        // Collapse
        state.expandedDirs.delete(entry.path);
        chevron.classList.remove('expanded');
        icon.className = 'tree-item-icon icon-folder';
        icon.innerHTML = icons.folder;

        const children = treeItem.nextElementSibling;
        if (children && children.classList.contains('tree-children')) {
            children.style.maxHeight = children.scrollHeight + 'px';
            children.offsetHeight; // force reflow
            children.style.maxHeight = '0';
            children.style.opacity = '0';
            // Remove after animation
            setTimeout(() => children.remove(), 300);
        }
    } else {
        // Expand
        state.expandedDirs.add(entry.path);
        chevron.classList.add('expanded');
        icon.className = 'tree-item-icon icon-folder-open';
        icon.innerHTML = icons.folderOpen;

        const entries = await loadDirectory(entry.path);
        const container = document.createElement('div');
        container.className = 'tree-children';

        for (const child of entries) {
            container.appendChild(createTreeItem(child, depth + 1));
        }

        // Animate in
        container.style.maxHeight = '0';
        container.style.opacity = '0';
        treeItem.insertAdjacentElement('afterend', container);

        requestAnimationFrame(() => {
            container.style.maxHeight = container.scrollHeight + 'px';
            container.style.opacity = '1';
        });

        // Clear max-height after transition to allow nested children to expand without clipping
        container.addEventListener('transitionend', function handler(e) {
            if (e.propertyName === 'max-height') {
                container.style.maxHeight = 'none';
                container.removeEventListener('transitionend', handler);
            }
        });
    }

    // Highlight selection
    selectTreeItem(treeItem);
}

function selectTreeItem(item) {
    if (state.selectedTreeItem) {
        state.selectedTreeItem.classList.remove('selected');
    }
    if (item) {
        item.classList.add('selected');
        state.selectedTreeItem = item;
    }
}

// ---- File Viewer ----
function renderFileContent(tabOrContent) {
    dom.fileContent.innerHTML = '';

    const path = typeof tabOrContent === 'object' ? (tabOrContent.path || '') : '';
    const content = typeof tabOrContent === 'object' ? (tabOrContent.content || '') : String(tabOrContent);
    const isPts = path.toLowerCase().includes('.pts') || (typeof tabOrContent === 'object' && tabOrContent.isPts);

    if (isPts) {
        const ptsData = typeof tabOrContent === 'object' ? tabOrContent.ptsData : null;
        const wrapper = document.createElement('div');
        wrapper.className = 'pts-embedded-viewer';
        wrapper.style.cssText = 'width:100%; height:100%; position:relative; background:#0b0f19; overflow:hidden; display:flex; flex-direction:column;';

        const ptCount = ptsData ? ptsData.count : 0;
        const bboxLabel = ptsData ? `[${ptsData.bbox_min.map(v => v.toFixed(0)).join(', ')}] → [${ptsData.bbox_max.map(v => v.toFixed(0)).join(', ')}]` : '—';

        wrapper.innerHTML = `
            <div style="height:40px; background:rgba(15,23,42,0.95); border-bottom:1px solid rgba(255,255,255,0.1); display:flex; align-items:center; justify-content:space-between; padding:0 16px;">
                <div style="display:flex; align-items:center; gap:8px;">
                    <span style="background:linear-gradient(135deg, #00f3ff, #0077ff); color:#000; font-weight:700; font-size:10px; padding:2px 6px; border-radius:10px; text-transform:uppercase;">Rust-GPU</span>
                    <span style="font-size:13px; font-weight:600; color:#f8fafc;">${path.split(/[/\\]/).pop() || 'pointcloud.pts'}</span>
                </div>
                <span style="font-family:monospace; font-size:12px; color:#00f3ff;">gcomp::pts_pointcloud_cs | ${ptCount} Points | BBox: ${bboxLabel}</span>
            </div>
            <div style="flex:1; position:relative; width:100%; height:calc(100% - 40px);">
                <canvas id="main-pts-canvas" style="position:absolute; top:0; left:0; width:100%; height:100%; display:block;"></canvas>
            </div>
        `;
        dom.fileContent.appendChild(wrapper);

        setTimeout(() => {
            const canvas = document.getElementById('main-pts-canvas');
            if (!canvas || !ptsData) return;
            const ctx = canvas.getContext('2d');
            let angleX = 0.4;
            let angleY = 0.6;

            const points = ptsData.points;
            const bMin = ptsData.bbox_min;
            const bMax = ptsData.bbox_max;

            // Bounding box vertices
            const bboxVerts = [
                [bMin[0], bMin[1], bMin[2]], [bMax[0], bMin[1], bMin[2]],
                [bMax[0], bMax[1], bMin[2]], [bMin[0], bMax[1], bMin[2]],
                [bMin[0], bMin[1], bMax[2]], [bMax[0], bMin[1], bMax[2]],
                [bMax[0], bMax[1], bMax[2]], [bMin[0], bMax[1], bMax[2]]
            ];
            const bboxEdges = [
                [0,1],[1,2],[2,3],[3,0],
                [4,5],[5,6],[6,7],[7,4],
                [0,4],[1,5],[2,6],[3,7]
            ];

            function project(x, y, z, width, height) {
                const cosY = Math.cos(angleY), sinY = Math.sin(angleY);
                const x1 = x * cosY + z * sinY, z1 = -x * sinY + z * cosY;
                const cosX = Math.cos(angleX), sinX = Math.sin(angleX);
                const y2 = y * cosX - z1 * sinX, z2 = y * sinX + z1 * cosX;
                const scale = 350 / (250 + z2);
                return { x: width / 2 + x1 * scale, y: height / 2 - y2 * scale, z: z2 };
            }

            function draw() {
                if (!canvas.parentElement) return;
                const dpr = window.devicePixelRatio || 1;
                const w = canvas.clientWidth || canvas.parentElement.clientWidth || 800;
                const h = canvas.clientHeight || canvas.parentElement.clientHeight || 500;
                canvas.width = Math.max(w * dpr, 200);
                canvas.height = Math.max(h * dpr, 200);

                ctx.clearRect(0, 0, canvas.width, canvas.height);
                ctx.fillStyle = '#0b0f19';
                ctx.fillRect(0, 0, canvas.width, canvas.height);

                // Draw faint bounding box wireframe
                const projBox = bboxVerts.map(v => project(v[0], v[1], v[2], canvas.width, canvas.height));
                ctx.strokeStyle = 'rgba(0, 243, 255, 0.15)';
                ctx.shadowColor = 'transparent';
                ctx.shadowBlur = 0;
                ctx.lineWidth = 1 * dpr;
                bboxEdges.forEach(([i, j]) => {
                    ctx.beginPath();
                    ctx.moveTo(projBox[i].x, projBox[i].y);
                    ctx.lineTo(projBox[j].x, projBox[j].y);
                    ctx.stroke();
                });

                // Draw point cloud
                ctx.shadowColor = '#00f3ff';
                ctx.shadowBlur = 8 * dpr;
                points.forEach(pt => {
                    const p = project(pt[0], pt[1], pt[2], canvas.width, canvas.height);
                    const depthFactor = Math.max(0.3, Math.min(1.0, (300 - p.z) / 400));
                    const radius = (3 + depthFactor * 3) * dpr;
                    const alpha = 0.5 + depthFactor * 0.5;
                    ctx.fillStyle = `rgba(0, 243, 255, ${alpha})`;
                    ctx.beginPath();
                    ctx.arc(p.x, p.y, radius, 0, Math.PI * 2);
                    ctx.fill();
                });

                ctx.shadowBlur = 0;
                ctx.fillStyle = 'rgba(226,232,240,0.8)';
                ctx.font = `${12 * dpr}px monospace`;
                ctx.fillText(`${ptCount} Points | BBox: ${bboxLabel}`, 16 * dpr, canvas.height - 16 * dpr);

                angleY += 0.02;
                angleX += 0.01;
                requestAnimationFrame(draw);
            }
            requestAnimationFrame(draw);
        }, 50);
        return;
    }

    const lines = content.split('\n');
    const fragment = document.createDocumentFragment();
    lines.forEach((line, i) => {
        const lineRow = document.createElement('div');
        lineRow.className = 'code-line';

        const lineNum = document.createElement('span');
        lineNum.className = 'line-number';
        lineNum.textContent = i + 1;

        const lineText = document.createElement('pre');
        lineText.className = 'line-text';
        lineText.textContent = line || ' ';

        lineRow.appendChild(lineNum);
        lineRow.appendChild(lineText);
        fragment.appendChild(lineRow);
    });
    dom.fileContent.appendChild(fragment);
}

function renderTabBar() {
    const tabBar = document.getElementById('content-tab-bar');
    if (!tabBar) return;
    tabBar.innerHTML = '';

    const fragment = document.createDocumentFragment();
    state.openTabs.forEach(tab => {
        const tabEl = document.createElement('div');
        tabEl.className = 'tab' + (tab.id === state.activeTabId ? ' active' : '');

        const nameSpan = document.createElement('span');
        nameSpan.className = 'tab-name';
        nameSpan.textContent = tab.name;

        const closeBtn = document.createElement('button');
        closeBtn.className = 'tab-close';
        closeBtn.title = 'Close';
        closeBtn.innerHTML = '&times;';

        closeBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            closeTab(tab.id);
        });

        tabEl.addEventListener('click', () => {
            activateTab(tab.id);
        });

        tabEl.appendChild(nameSpan);
        tabEl.appendChild(closeBtn);
        fragment.appendChild(tabEl);
    });

    tabBar.appendChild(fragment);
}

function displayActiveTabContent() {
    const tab = state.openTabs.find(t => t.id === state.activeTabId);
    if (!tab) {
        closeFile();
        return;
    }

    state.currentFile = tab;

    // Show viewer, hide welcome
    dom.contentWelcome.style.display = 'none';
    dom.contentViewer.style.display = 'flex';

    // Render content
    renderFileContent(tab);

    // Update status bar
    updateStatusBar(tab);
}

function activateTab(tabId) {
    const tab = state.openTabs.find(t => t.id === tabId);
    if (!tab) return;

    state.activeTabId = tabId;
    renderTabBar();
    displayActiveTabContent();
}

function closeTab(tabId) {
    const index = state.openTabs.findIndex(t => t.id === tabId);
    if (index === -1) return;

    state.openTabs.splice(index, 1);

    if (state.activeTabId === tabId) {
        if (state.openTabs.length > 0) {
            const nextIndex = Math.min(index, state.openTabs.length - 1);
            state.activeTabId = state.openTabs[nextIndex].id;
        } else {
            state.activeTabId = null;
        }
    }

    if (state.openTabs.length === 0) {
        closeFile();
    } else {
        renderTabBar();
        displayActiveTabContent();
    }
}

async function selectFile(entry, treeItem) {
    if (treeItem) {
        selectTreeItem(treeItem);
    }

    const ext = (entry.extension || entry.name?.split('.').pop() || entry.path.split('.').pop() || '').toLowerCase();
    const isPts = ext.startsWith('pts') || entry.path.toLowerCase().includes('.pts');

    // Check if the file is already open in a tab
    const existingTab = state.openTabs.find(tab => tab.path === entry.path);
    if (existingTab) {
        activateTab(existingTab.id);
        return;
    }

    try {
        let result = { content: '', line_count: 1, size: 0, path: entry.path };
        let ptsData = null;

        if (isPts) {
            try {
                ptsData = await invoke('XplrFetchPtsPoints');
            } catch (e) {
                console.error('GPU point cloud generation failed:', e);
            }
        } else {
            result = await invoke('XplrFetchContent', { path: entry.path });
        }

        const tabId = 'tab_' + Math.random().toString(36).substr(2, 9);
        const newTab = {
            id: tabId,
            path: entry.path || result.path,
            name: entry.name || entry.path.split(/[/\\]/).pop(),
            content: result.content,
            line_count: result.line_count,
            size: result.size,
            isPts: isPts,
            ptsData: ptsData
        };

        if (state.reuseWindow && state.openTabs.length > 0) {
            // Reuse active tab slot
            const activeIndex = state.openTabs.findIndex(t => t.id === state.activeTabId);
            if (activeIndex !== -1) {
                state.openTabs[activeIndex] = newTab;
            } else {
                state.openTabs = [newTab];
            }
        } else {
            // Open in a new tab
            state.openTabs.push(newTab);
        }

        state.activeTabId = newTab.id;

        renderTabBar();
        displayActiveTabContent();

    } catch (err) {
        console.error('Failed to read file:', err);
        dom.fileContent.innerHTML = `<div class="code-line"><span class="line-number">1</span><pre class="line-text">Error: ${err}</pre></div>`;
    }
}

function updateStatusBar(fileInfo) {
    if (fileInfo) {
        dom.statusFilePath.textContent = fileInfo.path;
        dom.statusLines.textContent = `Ln ${fileInfo.line_count}`;
        dom.statusSize.textContent = formatSize(fileInfo.size);
    } else {
        dom.statusFilePath.textContent = '';
        dom.statusLines.textContent = '';
        dom.statusSize.textContent = '';
    }
}

function updateReuseWindowUI() {
    if (dom.btnToggleReuseWindow) {
        dom.btnToggleReuseWindow.classList.toggle('active', state.reuseWindow);
        dom.btnToggleReuseWindow.title = state.reuseWindow
            ? 'Toggle Tab Reuse (Current: Reuse Active Tab) [Alt+R]'
            : 'Toggle Tab Reuse (Current: Open in New Tab) [Alt+R]';
    }
}

function toggleReuseWindow() {
    state.reuseWindow = !state.reuseWindow;
    updateReuseWindowUI();
}

function closeFile() {
    state.openTabs = [];
    state.activeTabId = null;
    state.currentFile = null;
    dom.contentViewer.style.display = 'none';
    dom.contentWelcome.style.display = 'flex';
    dom.fileContent.innerHTML = '';
    const tabBar = document.getElementById('content-tab-bar');
    if (tabBar) tabBar.innerHTML = '';
    updateStatusBar(null);
}

function toggleWordWrap() {
    state.wordWrap = !state.wordWrap;
    dom.fileContent.classList.toggle('word-wrap', state.wordWrap);
    dom.btnToggleWordWrap.classList.toggle('active', state.wordWrap);
}

const sunIcon = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <circle cx="12" cy="12" r="5"/>
    <line x1="12" y1="1" x2="12" y2="3"/>
    <line x1="12" y1="21" x2="12" y2="23"/>
    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
    <line x1="1" y1="12" x2="3" y2="12"/>
    <line x1="21" y1="12" x2="23" y2="12"/>
    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
</svg>`;

const moonIcon = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
</svg>`;

function toggleTheme() {
    state.theme = state.theme === 'dark' ? 'light' : 'dark';
    const isLight = state.theme === 'light';
    document.body.classList.toggle('light-theme', isLight);
    dom.btnToggleTheme.innerHTML = isLight ? moonIcon : sunIcon;
}

// ---- Open Folder ----

function customPrompt(message, defaultValue) {
    return new Promise((resolve) => {
        const overlay = document.createElement('div');
        overlay.style.position = 'fixed';
        overlay.style.top = '0';
        overlay.style.left = '0';
        overlay.style.width = '100%';
        overlay.style.height = '100%';
        overlay.style.background = 'rgba(0,0,0,0.5)';
        overlay.style.display = 'flex';
        overlay.style.alignItems = 'center';
        overlay.style.justifyContent = 'center';
        overlay.style.zIndex = '9999';

        const modal = document.createElement('div');
        modal.style.background = 'var(--bg-elevated)';
        modal.style.padding = '20px';
        modal.style.borderRadius = '8px';
        modal.style.boxShadow = '0 4px 12px rgba(0,0,0,0.5)';
        modal.style.width = '400px';
        modal.style.display = 'flex';
        modal.style.flexDirection = 'column';
        modal.style.gap = '12px';

        const label = document.createElement('label');
        label.textContent = message;
        label.style.color = 'var(--text-primary)';

        const input = document.createElement('input');
        input.type = 'text';
        input.value = defaultValue;
        input.style.padding = '8px';
        input.style.background = 'var(--bg-base)';
        input.style.border = '1px solid var(--border)';
        input.style.color = 'var(--text-primary)';
        input.style.borderRadius = '4px';
        input.style.outline = 'none';

        const btnRow = document.createElement('div');
        btnRow.style.display = 'flex';
        btnRow.style.justifyContent = 'flex-end';
        btnRow.style.gap = '8px';

        const btnCancel = document.createElement('button');
        btnCancel.textContent = 'Cancel';
        btnCancel.style.padding = '6px 12px';
        btnCancel.style.background = 'transparent';
        btnCancel.style.border = '1px solid var(--border)';
        btnCancel.style.color = 'var(--text-primary)';
        btnCancel.style.borderRadius = '4px';
        btnCancel.style.cursor = 'pointer';

        const btnOk = document.createElement('button');
        btnOk.textContent = 'OK';
        btnOk.style.padding = '6px 12px';
        btnOk.style.background = 'var(--accent)';
        btnOk.style.border = 'none';
        btnOk.style.color = '#11111b';
        btnOk.style.borderRadius = '4px';
        btnOk.style.cursor = 'pointer';

        btnRow.appendChild(btnCancel);
        btnRow.appendChild(btnOk);

        modal.appendChild(label);
        modal.appendChild(input);
        modal.appendChild(btnRow);
        overlay.appendChild(modal);
        document.body.appendChild(overlay);

        input.focus();
        input.select();

        const close = (val) => {
            document.body.removeChild(overlay);
            resolve(val);
        };

        btnOk.onclick = () => close(input.value);
        btnCancel.onclick = () => close(null);
        input.onkeydown = (e) => {
            if (e.key === 'Enter') close(input.value);
            if (e.key === 'Escape') close(null);
        };
    });
}

async function openFolder() {
    try {
        const path = await invoke('XplrSelectBranch');
        if (!path) return;

        state.rootPath = path;
        state.expandedDirs.clear();
        state.selectedTreeItem = null;

        // Clear tree
        dom.explorerTree.innerHTML = '';

        // Load root directory
        const entries = await loadDirectory(path);

        if (entries.length === 0) {
            dom.explorerTree.innerHTML = `
                <div class="explorer-empty">
                    <p>Empty directory</p>
                </div>
            `;
            return;
        }

        for (const entry of entries) {
            dom.explorerTree.appendChild(createTreeItem(entry, 0));
        }
    } catch (err) {
        console.error('Failed to select directory:', err);
    }
}

// ---- Toggle Explorer ----

function toggleExplorer() {
    state.explorerVisible = !state.explorerVisible;
    dom.explorer.classList.toggle('hidden', !state.explorerVisible);
    dom.resizeHandle.style.display = state.explorerVisible ? '' : 'none';
    dom.btnToggleExplorer.classList.toggle('active', state.explorerVisible);
}

// ---- Toggle Toolbar ----

function toggleToolbar() {
    state.toolbarVisible = !state.toolbarVisible;
    dom.toolbar.style.display = state.toolbarVisible ? '' : 'none';
    // Adjust grid
    const app = document.getElementById('app');
    if (state.toolbarVisible) {
        app.style.gridTemplateRows = 'var(--toolbar-height) 1fr var(--statusbar-height)';
    } else {
        app.style.gridTemplateRows = '0 1fr var(--statusbar-height)';
    }
}

// ---- Resize Handle (Explorer width) ----

function initResize() {
    let isResizing = false;

    dom.resizeHandle.addEventListener('mousedown', (e) => {
        isResizing = true;
        dom.resizeHandle.classList.add('active');
        document.body.style.cursor = 'col-resize';
        e.preventDefault();
    });

    document.addEventListener('mousemove', (e) => {
        if (!isResizing) return;
        const newWidth = e.clientX;
        if (newWidth >= 180 && newWidth <= 500) {
            dom.explorer.style.width = newWidth + 'px';
        }
    });

    document.addEventListener('mouseup', () => {
        if (isResizing) {
            isResizing = false;
            dom.resizeHandle.classList.remove('active');
            document.body.style.cursor = '';
        }
    });
}

// ---- Menu Event Listener ----

async function initMenuEvents() {
    try {
        await listen('menu-event', (event) => {
            const menuId = event.payload;
            switch (menuId) {
                case 'open_folder':
                    openFolder();
                    break;
                case 'close_folder':
                    state.rootPath = null;
                    state.expandedDirs.clear();
                    dom.explorerTree.innerHTML = `
                        <div class="explorer-empty">
                            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" opacity="0.3">
                                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                            </svg>
                            <p>No folder opened</p>
                            <button onclick="openFolder()" class="btn-open-empty">Open Folder</button>
                        </div>
                    `;
                    closeFile();
                    break;
                case 'toggle_explorer':
                    toggleExplorer();
                    break;
                case 'toggle_toolbar':
                    toggleToolbar();
                    break;
                case 'toggle_word_wrap':
                    toggleWordWrap();
                    break;
                case 'toggle_theme':
                    toggleTheme();
                    break;
                case 'toggle_reuse_window':
                    toggleReuseWindow();
                    break;
                case 'about':
                    alert('Fenst v0.1.0\nLightweight File Explorer & Viewer\nBuilt with Tauri');
                    break;
            }
        });
    } catch (err) {
        console.error('Failed to listen for menu events:', err);
    }
}

// ---- Keyboard Shortcuts ----

function initKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
        const ctrl = e.ctrlKey || e.metaKey;

        if (ctrl && e.key === 'o') {
            e.preventDefault();
            openFolder();
        } else if (ctrl && e.key === 'b') {
            e.preventDefault();
            toggleExplorer();
        } else if (ctrl && e.key === 't') {
            e.preventDefault();
            toggleTheme();
        } else if (e.altKey && e.key.toLowerCase() === 'z') {
            e.preventDefault();
            toggleWordWrap();
        } else if (e.altKey && e.key.toLowerCase() === 'r') {
            e.preventDefault();
            toggleReuseWindow();
        }
    });
}

// ---- Initialize ----

function init() {
    // Button events
    dom.btnOpenFolder.addEventListener('click', openFolder);
    dom.btnOpenFolderEmpty.addEventListener('click', openFolder);
    dom.btnRefresh.addEventListener('click', () => {
        if (state.rootPath) {
            // Re-open current root
            const path = state.rootPath;
            state.expandedDirs.clear();
            state.selectedTreeItem = null;
            dom.explorerTree.innerHTML = '';
            state.rootPath = path;
            loadDirectory(path).then(entries => {
                for (const entry of entries) {
                    dom.explorerTree.appendChild(createTreeItem(entry, 0));
                }
            });
        }
    });
    dom.btnToggleExplorer.addEventListener('click', toggleExplorer);
    dom.btnToggleWordWrap.addEventListener('click', toggleWordWrap);
    dom.btnToggleTheme.addEventListener('click', toggleTheme);
    if (dom.btnToggleReuseWindow) {
        dom.btnToggleReuseWindow.addEventListener('click', toggleReuseWindow);
    }

    // Initialize subsystems
    updateReuseWindowUI();
    initResize();
    initKeyboardShortcuts();
    initMenuEvents();

    // Check if launched as a dedicated file viewer window via URL query parameter
    const urlParams = new URLSearchParams(window.location.search);
    const fileParam = urlParams.get('file');
    if (fileParam) {
        const fileName = fileParam.split(/[/\\]/).pop() || fileParam;
        const savedReuse = state.reuseWindow;
        state.reuseWindow = true;
        selectFile({ path: fileParam, name: fileName }, null);
        state.reuseWindow = savedReuse;
    }
}

// Start when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
