// ==================== State & Config ====================
const CONFIG = {
    simDuration: 60.0,
    traceLen: 100,
    colors: ['#1f77b4', '#d62728', '#2ca02c', '#17becf', '#9467bd', '#e377c2', '#bcbd22']
};

let state = {
    animData: null,
    isPlaying: false,
    frameIdx: 0,
    animId: null,
    playStartTime: 0
};

// ==================== DOM Elements ====================
const els = {
    canvas: document.getElementById('anim-canvas'),
    ctx: document.getElementById('anim-canvas').getContext('2d'),
    graph: document.getElementById('graph-canvas'),
    gctx: document.getElementById('graph-canvas').getContext('2d'),
    inputs: {
        n: document.getElementById('n'),
        container: document.getElementById('params-fields')
    },
    btns: {
        run: document.getElementById('btn-run'),
        play: document.getElementById('btn-playpause'),
        reset: document.getElementById('btn-reset')
    },
    loader: document.getElementById('loading-txt')
};

// ==================== Helpers ====================

// Convert physics coordinates to canvas coordinates
const getScreenPos = (x, y, scale, w, h) => ({
    x: x * scale + (w / 2),
    y: -y * scale + (h / 2)
});

// Draws a graph-paper style grid (Major & Minor lines)
const drawGrid = (ctx, w, h, scale, detailed = false) => {
    ctx.save();
    const ox = w / 2, oy = h / 2;
    
    // 1. Draw Background
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, w, h);

    // 2. Draw Minor Grid (Light) - Every 0.1 units roughly
    if (detailed && scale) {
        ctx.strokeStyle = '#f0f0f0';
        ctx.lineWidth = 1;
        ctx.beginPath();
        const step = scale / 5; // density of grid
        // Vertical & Horizontal loops combined logic could go here, 
        // but simple loops are faster for grids
        for (let x = ox % step; x < w; x += step) { ctx.moveTo(x, 0); ctx.lineTo(x, h); }
        for (let y = oy % step; y < h; y += step) { ctx.moveTo(0, y); ctx.lineTo(w, y); }
        ctx.stroke();
    }

    // 3. Draw Major Grid (Darker)
    ctx.strokeStyle = detailed ? '#ccc' : '#eee';
    ctx.lineWidth = 1;
    ctx.beginPath();
    // Center Axes
    ctx.moveTo(0, oy); ctx.lineTo(w, oy);
    ctx.moveTo(ox, 0); ctx.lineTo(ox, h);
    ctx.stroke();
    
    // Border
    if (detailed) {
        ctx.strokeStyle = '#000';
        ctx.strokeRect(0, 0, w, h);
    }
    ctx.restore();
};

// Gather inputs for N pendulums
const getSimInputs = (n) => {
    let m = [], l = [], th = [];
    for (let i = 1; i <= n; i++) {
        m.push(document.getElementById(`m${i}`).value);
        l.push(document.getElementById(`L${i}`).value);
        th.push(document.getElementById(`th${i}`).value);
    }
    return { 
        n, 
        masses: m.join(','), 
        lengths: l.join(','), 
        initial_angles: th.join(','), 
        t_max: CONFIG.simDuration, 
        n_points: 8000 
    };
};

// ==================== Core Functions ====================

const resizeCanvases = () => {
    const size = els.canvas.parentElement.clientWidth;
    els.canvas.width = els.canvas.height = size;
    els.graph.width = els.graph.height = size;
    // Redraw static content if paused
    if (!state.isPlaying && state.animData) {
        drawFrame();
        drawGraph();
    }
};

const generateFields = () => {
    const n = parseInt(els.inputs.n.value) || 2;
    els.inputs.container.innerHTML = Array.from({ length: n }, (_, i) => {
        const idx = i + 1;
        // Default logic: 1st=90deg, 2nd=45deg, others=0
        const thVal = idx === 1 ? 90 : idx === 2 ? 45 : 0;
        return `
            <div class="param-group">
                <strong>Pendulum ${idx}</strong><br>
                Mass (kg): <input type="number" step="0.01" id="m${idx}" value="1.0" required><br>
                Length (m): <input type="number" step="0.01" id="L${idx}" value="1.0" required><br>
                Initial θ (deg): <input type="number" step="1" id="th${idx}" value="${thVal}" required>
            </div>`;
    }).join('');
};

// ==================== Drawing ====================

function drawFrame() {
    if (!state.animData) return;
    const { width: w, height: h } = els.canvas;
    const scale = (w / 2) / state.animData.limit;

    // Clear & Grid
    els.ctx.clearRect(0, 0, w, h);
    drawGrid(els.ctx, w, h, scale, false); // Simple grid for animation

    const pos = state.animData.positions[state.frameIdx];

    // 1. Trace (Red tail)
    els.ctx.strokeStyle = 'rgba(255, 0, 0, 0.6)';
    els.ctx.lineWidth = 2;
    els.ctx.beginPath();
    const startTrace = Math.max(0, state.frameIdx - CONFIG.traceLen);
    const endIdx = (state.animData.n - 1) * 2; // Index for last pendulum x

    for (let j = startTrace; j <= state.frameIdx; j++) {
        const p = state.animData.positions[j];
        const { x, y } = getScreenPos(p[endIdx], p[endIdx + 1], scale, w, h);
        j === startTrace ? els.ctx.moveTo(x, y) : els.ctx.lineTo(x, y);
    }
    els.ctx.stroke();

    // 2. Rods & Masses
    els.ctx.lineWidth = 3;
    els.ctx.strokeStyle = 'black';
    els.ctx.beginPath();
    els.ctx.moveTo(w / 2, h / 2); // Pivot

    // Draw rods
    for (let k = 0; k < state.animData.n; k++) {
        const { x, y } = getScreenPos(pos[2 * k], pos[2 * k + 1], scale, w, h);
        els.ctx.lineTo(x, y);
    }
    els.ctx.stroke();

    // Draw pivot & masses
    const drawCircle = (ctx, cx, cy, r, color) => {
        ctx.fillStyle = color;
        ctx.beginPath(); ctx.arc(cx, cy, r, 0, Math.PI * 2); ctx.fill();
    };

    drawCircle(els.ctx, w/2, h/2, 4, 'black'); // Pivot
    for (let k = 0; k < state.animData.n; k++) {
        const { x, y } = getScreenPos(pos[2 * k], pos[2 * k + 1], scale, w, h);
        drawCircle(els.ctx, x, y, 6, '#333');
    }
}

function drawGraph() {
    if (!state.animData) return;
    const { width: w, height: h } = els.graph;
    const scale = (Math.min(w, h) / 2) / state.animData.limit;

    // Draw detailed "Graph Paper" background
    drawGrid(els.gctx, w, h, scale, true); 

    // Draw Trajectories
    els.gctx.lineWidth = 2.5;
    els.gctx.lineJoin = els.gctx.lineCap = 'round';

    for (let k = 0; k < state.animData.n; k++) {
        els.gctx.strokeStyle = CONFIG.colors[k % CONFIG.colors.length];
        els.gctx.beginPath();
        let started = false;
        
        // Loop up to current frameIdx
        for (let j = 0; j <= state.frameIdx; j++) {
            const p = state.animData.positions[j];
            const { x, y } = getScreenPos(p[2 * k], p[2 * k + 1], scale, w, h);
            if (!started) { els.gctx.moveTo(x, y); started = true; }
            else { els.gctx.lineTo(x, y); }
        }
        els.gctx.stroke();
    }
}

// ==================== Animation Loop ====================

const animate = () => {
    if (state.isPlaying && state.animData) {
        const elapsed = (Date.now() - state.playStartTime) / 1000;
        const progress = (elapsed / CONFIG.simDuration) % 1;
        state.frameIdx = Math.floor(progress * state.animData.positions.length);
        
        drawFrame();
        drawGraph();
    }
    state.animId = requestAnimationFrame(animate);
};

// Enforce strict limits on the Input field
const nInput = document.getElementById('n');

nInput.addEventListener('input', () => {
    let val = parseInt(nInput.value);
    
    // Force minimum 1
    if (val < 1) {
        nInput.value = 1;
    } 
    // Force maximum 150
    else if (val > 150) {
        nInput.value = 150;
        //Alert the user or just silently clamp it
        alert("Maximum pendulums limited to 150 for performance."); 
    }
    
    // Regenerate the fields immediately
    generateFields();
});


// ==================== Event Listeners ====================

els.inputs.n.addEventListener('change', generateFields);
window.addEventListener('resize', resizeCanvases);

els.btns.run.addEventListener('click', () => {
    const n = parseInt(els.inputs.n.value);
    if (n > 20 && !confirm(`Simulating ${n} pendulums may be slow. Continue?`)) return;

    // UI Updates
    els.loader.style.display = 'block';
    state.isPlaying = false;
    els.btns.play.textContent = 'Play';

    // Fetch
    fetch('/simulate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(getSimInputs(n))
    })
    .then(res => res.ok ? res.json() : res.text().then(t => { throw new Error(t) }))
    .then(data => {
        if (!data.success) throw new Error(data.message || 'Unknown error');
        
        // Success
        state.animData = data.animation_data;
        state.frameIdx = 0;
        state.isPlaying = true;
        els.btns.play.textContent = 'Pause';
        state.playStartTime = Date.now();
        drawFrame();
        drawGraph();
    })
    .catch(err => alert('Error: ' + err.message))
    .finally(() => els.loader.style.display = 'none');
});

els.btns.play.addEventListener('click', () => {
    if (!state.animData) return els.btns.run.click();
    
    // Resume logic: Calculate offset to maintain smooth animation
    if (!state.isPlaying) {
        const currentProg = state.frameIdx / (state.animData.positions.length - 1);
        state.playStartTime = Date.now() - (currentProg * CONFIG.simDuration * 1000);
    }
    
    state.isPlaying = !state.isPlaying;
    els.btns.play.textContent = state.isPlaying ? 'Pause' : 'Play';
});

els.btns.reset.addEventListener('click', () => {
    state.frameIdx = 0;
    if (state.isPlaying) state.playStartTime = Date.now();
    drawFrame();
    drawGraph();
});

// Init
resizeCanvases();
generateFields();
animate();