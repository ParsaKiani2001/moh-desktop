// ============================================
// ارتباط با WM از طریق IPC
// ============================================

function sendToWM(action, data = {}) {
    const msg = JSON.stringify({ action, ...data });
    window.ipc.postMessage(msg);
}

// ============================================
// دکمه‌ها
// ============================================

document.querySelectorAll('.btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const action = btn.dataset.action;
        console.log('Button clicked:', action);
        sendToWM(action);
    });
});

// ============================================
// Drag برای جابه‌جایی
// ============================================

const titlebar = document.getElementById('titlebar');
let isDragging = false;
let startX = 0, startY = 0;

titlebar.addEventListener('mousedown', (e) => {
    if (e.target.classList.contains('btn')) return;
    
    isDragging = true;
    startX = e.screenX;
    startY = e.screenY;
    titlebar.style.cursor = 'grabbing';
});

document.addEventListener('mousemove', (e) => {
    if (!isDragging) return;
    
    const dx = e.screenX - startX;
    const dy = e.screenY - startY;
    
    if (Math.abs(dx) > 0 || Math.abs(dy) > 0) {
        sendToWM('move', { dx, dy });
        startX = e.screenX;
        startY = e.screenY;
    }
});

document.addEventListener('mouseup', () => {
    if (isDragging) {
        isDragging = false;
        titlebar.style.cursor = 'grab';
        sendToWM('drag_end');
    }
});

// ============================================
// Resize (با لبه‌های پنجره)
// ============================================

let isResizing = false;
let resizeStartX, resizeStartY;

// Resize از لبه راست
document.addEventListener('mousemove', (e) => {
    if (!isResizing) return;
    
    const dx = e.screenX - resizeStartX;
    const dy = e.screenY - resizeStartY;
    
    sendToWM('resize', { dx, dy });
    resizeStartX = e.screenX;
    resizeStartY = e.screenY;
});

window.addEventListener('ipc-message', (e) => {
    try {
        const msg = JSON.parse(e.data);
        handleWMMessage(msg);
    } catch (err) {
        console.error('Invalid message:', err);
    }
});

function handleWMMessage(msg) {
    switch (msg.type) {
        case 'set_title':
            document.getElementById('title').textContent = msg.title;
            break;
            
        case 'set_focus':
            titlebar.classList.toggle('inactive', !msg.focused);
            break;
            
        case 'set_theme':
            document.getElementById('theme').href = `themes/${msg.theme}.css`;
            break;
            
        case 'fullscreen':
            document.body.classList.toggle('fullscreen', msg.enabled);
            break;
            
        case 'minimized':
            document.body.style.display = msg.hidden ? 'none' : 'block';
            break;
    }
}
titlebar.addEventListener('dblclick', (e) => {
    if (e.target.classList.contains('btn')) return;
    sendToWM('maximize');
});

console.log('UI Service ready');