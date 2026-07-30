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
    lineNumbers:        document.getElementById('line-numbers'),
    fileText:           document.getElementById('file-text'),
    tabFilename:        document.getElementById('tab-filename'),
    tabClose:           document.getElementById('tab-close'),
    statusFilePath:     document.getElementById('status-file-path'),
    statusLines:        document.getElementById('status-lines'),
    statusSize:         document.getElementById('status-size'),
    resizeHandle:       document.getElementById('resize-handle'),
    btnOpenFolder:      document.getElementById('btn-open-folder'),
    btnOpenFolderEmpty: document.getElementById('btn-open-folder-empty'),
    btnRefresh:         document.getElementById('btn-refresh'),
    btnToggleExplorer:  document.getElementById('btn-toggle-explorer'),
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
        const entries = await invoke('read_directory', { path });
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
            children.classList.add('collapsed');
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
    }

    // Highlight selection
    selectTreeItem(treeItem);
}

function selectTreeItem(item) {
    if (state.selectedTreeItem) {
        state.selectedTreeItem.classList.remove('selected');
    }
    item.classList.add('selected');
    state.selectedTreeItem = item;
}

// ---- File Viewer ----

async function selectFile(entry, treeItem) {
    selectTreeItem(treeItem);

    try {
        const result = await invoke('read_file_contents', { path: entry.path });

        state.currentFile = result;

        // Show viewer, hide welcome
        dom.contentWelcome.style.display = 'none';
        dom.contentViewer.style.display = 'flex';

        // Update tab
        dom.tabFilename.textContent = entry.name;

        // Render line numbers
        const lines = result.content.split('\n');
        dom.lineNumbers.textContent = lines.map((_, i) => i + 1).join('\n');

        // Render file content
        dom.fileText.textContent = result.content;

        // Sync scroll between line numbers and content
        dom.fileContent.addEventListener('scroll', syncScroll);

        // Update status bar
        updateStatusBar(result);

    } catch (err) {
        console.error('Failed to read file:', err);
        dom.fileText.textContent = `Error: ${err}`;
        dom.lineNumbers.textContent = '1';
    }
}

function syncScroll() {
    dom.lineNumbers.style.transform = `translateY(-${dom.fileContent.scrollTop}px)`;
}

function updateStatusBar(fileContents) {
    if (fileContents) {
        dom.statusFilePath.textContent = fileContents.path;
        dom.statusLines.textContent = `Ln ${fileContents.line_count}`;
        dom.statusSize.textContent = formatSize(fileContents.size);
    } else {
        dom.statusFilePath.textContent = '';
        dom.statusLines.textContent = '';
        dom.statusSize.textContent = '';
    }
}

function closeFile() {
    state.currentFile = null;
    dom.contentViewer.style.display = 'none';
    dom.contentWelcome.style.display = 'flex';
    dom.fileText.textContent = '';
    dom.lineNumbers.textContent = '';
    updateStatusBar(null);
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
    const path = await customPrompt('Enter folder path to open:', state.rootPath || '/');
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
    dom.tabClose.addEventListener('click', closeFile);

    // Initialize subsystems
    initResize();
    initKeyboardShortcuts();
    initMenuEvents();
}

// Start when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
