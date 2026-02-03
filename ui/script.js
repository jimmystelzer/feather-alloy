// Função para aguardar a API do Tauri estar disponível
let invoke;

async function waitForTauri(maxAttempts = 50) {
    for (let i = 0; i < maxAttempts; i++) {
        console.log(`Tentativa ${i + 1} de detectar API do Tauri...`);
        console.log('window.__TAURI__:', window.__TAURI__);
        
        if (window.__TAURI__ && window.__TAURI__.invoke) {
            console.log('✓ API do Tauri detectada via window.__TAURI__.invoke');
            return window.__TAURI__.invoke;
        }
        
        if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
            console.log('✓ API do Tauri detectada via window.__TAURI__.core.invoke');
            return window.__TAURI__.core.invoke;
        }
        
        if (window.__TAURI_INVOKE__) {
            console.log('✓ API do Tauri detectada via window.__TAURI_INVOKE__');
            return window.__TAURI_INVOKE__;
        }
        
        // Aguarda 100ms antes da próxima tentativa
        await new Promise(resolve => setTimeout(resolve, 100));
    }
    
    console.error('❌ API do Tauri não encontrada após', maxAttempts, 'tentativas');
    console.log('window.__TAURI__:', window.__TAURI__);
    console.log('Todas as propriedades de window:', Object.keys(window).filter(k => k.includes('TAURI')));
    return null;
}

// Estado da aplicação
let profiles = [];

// Elementos DOM
const addBtn = document.getElementById('addBtn');
const settingsBtn = document.getElementById('settingsBtn');
const addServiceModal = document.getElementById('addServiceModal');
const closeModalBtn = document.getElementById('closeModalBtn');
const cancelBtn = document.getElementById('cancelBtn');
const addServiceForm = document.getElementById('addServiceForm');
const contextMenu = document.getElementById('contextMenu');
const mainPanel = document.getElementById('mainPanel');
const profilesContainer = document.getElementById('profilesContainer');

// Perfil atualmente ativo
let activeProfileId = null;

// Carregar perfis ao iniciar
async function loadProfiles() {
    try {
        profiles = await invoke('get_profiles');
        renderProfiles();
        console.log('Perfis carregados:', profiles);
    } catch (error) {
        console.error('Erro ao carregar perfis:', error);
    }
}

// Renderizar botões de perfis dinamicamente
function renderProfiles() {
    // Limpa botões existentes (exceto os de controle)
    const existingButtons = profilesContainer.querySelectorAll('.profile-btn');
    existingButtons.forEach(btn => btn.remove());
    
    // Cria botões para cada perfil
    profiles.forEach(profile => {
        const button = document.createElement('button');
        button.className = 'profile-btn';
        button.dataset.profileId = profile.uuid;
        button.title = profile.name;
        
        // Se tiver ícone customizado, usa; senão, usa ícone padrão
        if (profile.icon_path) {
            const img = document.createElement('img');
            img.src = profile.icon_path;
            img.alt = profile.name;
            button.appendChild(img);
        } else {
            // Ícone padrão (círculo com inicial)
            const initial = profile.name.charAt(0).toUpperCase();
            button.textContent = initial;
            button.style.fontSize = '24px';
            button.style.fontWeight = 'bold';
        }
        
        profilesContainer.appendChild(button);
    });
}

// Abrir modal de adicionar serviço
addBtn.addEventListener('click', () => {
    addServiceModal.classList.add('active');
});

// Fechar modal
function closeModal() {
    addServiceModal.classList.remove('active');
    addServiceForm.reset();
}

closeModalBtn.addEventListener('click', closeModal);
cancelBtn.addEventListener('click', closeModal);

// Fechar modal ao clicar fora
addServiceModal.addEventListener('click', (e) => {
    if (e.target === addServiceModal) {
        closeModal();
    }
});

// Fechar modal ao pressionar ESC
document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
        closeModal();
        closeContextMenu();
    }
});

// Submeter formulário de adicionar serviço
addServiceForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const formData = new FormData(addServiceForm);
    const name = formData.get('serviceName');
    const url = formData.get('serviceUrl');
    const userAgent = formData.get('userAgent') || null;
    const iconPath = formData.get('iconPath') || null;
    
    try {
        const newProfile = await invoke('add_profile', {
            name,
            url,
            iconPath,
            userAgent,
        });
        
        console.log('Novo perfil adicionado:', newProfile);
        
        // Recarrega lista de perfis
        await loadProfiles();
        
        closeModal();
        
        // Notificação de sucesso
        showNotification(`Serviço "${name}" adicionado com sucesso!`);
    } catch (error) {
        console.error('Erro ao adicionar perfil:', error);
        alert(`Erro ao adicionar serviço: ${error}`);
    }
});

// Função para mostrar notificação temporária
function showNotification(message) {
    const notification = document.createElement('div');
    notification.className = 'notification';
    notification.textContent = message;
    notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: #4CAF50;
        color: white;
        padding: 16px 24px;
        border-radius: 8px;
        box-shadow: 0 4px 12px rgba(0,0,0,0.2);
        z-index: 10000;
        animation: slideIn 0.3s ease-out;
    `;
    
    document.body.appendChild(notification);
    
    setTimeout(() => {
        notification.style.animation = 'slideOut 0.3s ease-out';
        setTimeout(() => notification.remove(), 300);
    }, 3000);
}

// Gerenciar cliques nos perfis
document.addEventListener('click', async (e) => {
    const profileBtn = e.target.closest('.profile-btn');
    
    if (profileBtn && profileBtn.dataset.profileId) {
        const uuid = profileBtn.dataset.profileId;
        await switchProfile(uuid);
    }
});

// Alternar perfil ativo
async function switchProfile(uuid) {
    try {
        // Remover estado ativo de todos os botões
        document.querySelectorAll('.profile-btn').forEach(btn => {
            btn.classList.remove('active');
        });
        
        // Adicionar estado ativo ao botão clicado
        const activeBtn = document.querySelector(`[data-profile-id="${uuid}"]`);
        if (activeBtn) {
            activeBtn.classList.add('active');
        }
        
        activeProfileId = uuid;
        
        // Atualizar painel principal
        const profile = profiles.find(p => p.uuid === uuid);
        if (profile) {
            mainPanel.innerHTML = `
                <div class="welcome-screen">
                    <h1>${profile.name}</h1>
                    <p>Abrindo webview...</p>
                </div>
            `;
        }
        
        console.log('Abrindo webview para perfil:', uuid);
        
        // Invocar comando Tauri para mostrar webview
        await invoke('show_webview', { uuid });
        
        console.log('Webview aberta com sucesso');
    } catch (error) {
        console.error('Erro ao alternar perfil:', error);
        alert(`Erro ao abrir webview: ${error}`);
    }
}

// Menu de contexto (clique direito)
document.addEventListener('contextmenu', (e) => {
    const profileBtn = e.target.closest('.profile-btn');
    
    if (profileBtn && profileBtn.dataset.profileId) {
        e.preventDefault();
        showContextMenu(e.clientX, e.clientY, profileBtn);
    } else {
        closeContextMenu();
    }
});

let currentContextProfileBtn = null;

function showContextMenu(x, y, profileBtn) {
    currentContextProfileBtn = profileBtn;
    contextMenu.style.left = `${x}px`;
    contextMenu.style.top = `${y}px`;
    contextMenu.classList.add('active');
}

function closeContextMenu() {
    contextMenu.classList.remove('active');
    currentContextProfileBtn = null;
}

// Fechar menu de contexto ao clicar fora
document.addEventListener('click', (e) => {
    if (!e.target.closest('.context-menu') && !e.target.closest('.profile-btn')) {
        closeContextMenu();
    }
});

// Ação de editar perfil
document.getElementById('editProfileBtn').addEventListener('click', () => {
    if (currentContextProfileBtn) {
        const uuid = currentContextProfileBtn.dataset.profileId;
        const profile = profiles.find(p => p.uuid === uuid);
        
        if (profile) {
            console.log('Editando perfil:', profile);
            // TODO: Abrir modal de edição com dados pré-preenchidos
            alert(`Editar perfil: ${profile.name}\n(Será implementado na Fase 5)`);
        }
    }
    
    closeContextMenu();
});

// Botão de configurações
settingsBtn.addEventListener('click', () => {
    console.log('Configurações clicadas');
    // TODO: Abrir modal de configurações (Fase 4)
    alert('Configurações serão implementadas na Fase 4');
});

// Adicionar animações CSS
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from {
            transform: translateX(100%);
            opacity: 0;
        }
        to {
            transform: translateX(0);
            opacity: 1;
        }
    }
    
    @keyframes slideOut {
        from {
            transform: translateX(0);
            opacity: 1;
        }
        to {
            transform: translateX(100%);
            opacity: 0;
        }
    }
`;
document.head.appendChild(style);

// Inicialização
console.log('=== Feather Alloy UI carregada ===');

// Aguarda a API do Tauri estar disponível antes de carregar perfis
window.addEventListener('DOMContentLoaded', async () => {
    console.log('DOMContentLoaded disparado, aguardando API do Tauri...');
    invoke = await waitForTauri();
    
    if (invoke) {
        console.log('✓ Inicialização bem-sucedida! Carregando perfis...');
        loadProfiles();
    } else {
        console.error('❌ Não foi possível inicializar a API do Tauri');
        console.error('A aplicação não funcionará corretamente.');
    }
});



