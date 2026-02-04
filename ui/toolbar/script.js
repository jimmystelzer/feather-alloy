// Estado local
let profiles = [];
let activeProfileUuid = null;

// Inicialização
document.addEventListener('DOMContentLoaded', () => {
    initializeEventListeners();
    loadProfiles();
});

function initializeEventListeners() {
    // Botão adicionar
    document.getElementById('addBtn').addEventListener('click', openAddModal);
    
    // Botão configurações
    document.getElementById('settingsBtn').addEventListener('click', () => {
        sendMessage({ type: 'ShowSettings' });
    });
    
    // Modal
    document.getElementById('closeModalBtn').addEventListener('click', closeAddModal);
    document.getElementById('cancelBtn').addEventListener('click', closeAddModal);
    document.getElementById('addServiceForm').addEventListener('submit', handleAddProfile);
    
    // Fechar modal ao clicar fora
    document.getElementById('addServiceModal').addEventListener('click', (e) => {
        if (e.target.id === 'addServiceModal') {
            closeAddModal();
        }
    });
}

function openAddModal() {
    document.getElementById('addServiceModal').classList.add('active');
}

function closeAddModal() {
    document.getElementById('addServiceModal').classList.remove('active');
    document.getElementById('addServiceForm').reset();
}

function handleAddProfile(e) {
    e.preventDefault();
    
    const name = document.getElementById('serviceName').value;
    const url = document.getElementById('serviceUrl').value;
    const userAgent = document.getElementById('userAgent').value || null;
    
    sendMessage({
        type: 'AddProfile',
        payload: {
            name,
            url,
            icon_path: null,
            user_agent: userAgent
        }
    });
    
    closeAddModal();
}

function loadProfiles() {
    sendMessage({ type: 'GetProfiles' });
}

function renderProfiles() {
    const container = document.getElementById('profilesContainer');
    container.innerHTML = '';
    
    profiles.forEach(profile => {
        const btn = createProfileButton(profile);
        container.appendChild(btn);
    });
}

function createProfileButton(profile) {
    const btn = document.createElement('button');
    btn.className = 'profile-btn';
    btn.title = profile.name;
    btn.dataset.uuid = profile.uuid;
    
    if (profile.uuid === activeProfileUuid) {
        btn.classList.add('active');
    }
    
    // Se tem ícone, usar imagem, senão usar inicial do nome
    if (profile.icon_path) {
        const img = document.createElement('img');
        img.src = profile.icon_path;
        img.alt = profile.name;
        btn.appendChild(img);
    } else {
        const initial = document.createElement('span');
        initial.className = 'initial';
        initial.textContent = profile.name.charAt(0);
        btn.appendChild(initial);
    }
    
    btn.addEventListener('click', () => {
        selectProfile(profile.uuid);
    });
    
    // Context menu (botão direito)
    btn.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        showContextMenu(profile, e.clientX, e.clientY);
    });
    
    return btn;
}

function selectProfile(uuid) {
    activeProfileUuid = uuid;
    renderProfiles();
    
    sendMessage({
        type: 'ShowProfile',
        payload: { uuid }
    });
}

function showContextMenu(profile, x, y) {
    // TODO: Implementar menu de contexto para editar/remover perfil
    console.log('Context menu for profile:', profile);
}

// Comunicação IPC
function sendMessage(message) {
    if (window.ipc) {
        window.ipc.postMessage(JSON.stringify(message));
    } else {
        console.error('IPC not available');
    }
}

// Receber mensagens do backend
window.addEventListener('message', (event) => {
    try {
        const message = JSON.parse(event.data);
        handleBackendMessage(message);
    } catch (e) {
        console.error('Failed to parse message:', e);
    }
});

function handleBackendMessage(message) {
    console.log('Received message:', message);
    
    switch (message.type) {
        case 'ProfileAdded':
            profiles.push(message.payload.profile);
            renderProfiles();
            break;
            
        case 'ProfileRemoved':
            profiles = profiles.filter(p => p.uuid !== message.payload.uuid);
            if (activeProfileUuid === message.payload.uuid) {
                activeProfileUuid = null;
            }
            renderProfiles();
            break;
            
        case 'ProfilesList':
            profiles = message.payload.profiles;
            renderProfiles();
            break;
            
        case 'Error':
            console.error('Backend error:', message.payload.message);
            alert('Erro: ' + message.payload.message);
            break;
    }
}
