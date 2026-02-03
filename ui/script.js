// Estado da aplicação (mock para Fase 1)
let profiles = [
    { id: 1, name: 'WhatsApp', active: false },
    { id: 2, name: 'Gmail', active: false },
    { id: 3, name: 'Slack', active: false }
];

// Elementos DOM
const addBtn = document.getElementById('addBtn');
const settingsBtn = document.getElementById('settingsBtn');
const addServiceModal = document.getElementById('addServiceModal');
const closeModalBtn = document.getElementById('closeModalBtn');
const cancelBtn = document.getElementById('cancelBtn');
const addServiceForm = document.getElementById('addServiceForm');
const contextMenu = document.getElementById('contextMenu');
const mainPanel = document.getElementById('mainPanel');

// Perfil atualmente ativo
let activeProfileId = null;

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
addServiceForm.addEventListener('submit', (e) => {
    e.preventDefault();
    
    const formData = new FormData(addServiceForm);
    const newProfile = {
        id: profiles.length + 1,
        name: formData.get('serviceName'),
        url: formData.get('serviceUrl'),
        userAgent: formData.get('userAgent') || null,
        iconPath: formData.get('iconPath') || null,
        active: false
    };
    
    profiles.push(newProfile);
    console.log('Novo perfil adicionado:', newProfile);
    
    // TODO: Na Fase 2, enviar para o backend Tauri
    // await invoke('add_profile', { profile: newProfile });
    
    closeModal();
    
    // Mostrar notificação (futuro)
    alert(`Serviço "${newProfile.name}" adicionado com sucesso!`);
});

// Gerenciar cliques nos perfis
document.addEventListener('click', (e) => {
    const profileBtn = e.target.closest('.profile-btn');
    
    if (profileBtn) {
        const profileId = parseInt(profileBtn.dataset.profileId);
        switchProfile(profileId);
    }
});

// Alternar perfil ativo
function switchProfile(profileId) {
    // Remover estado ativo de todos os botões
    document.querySelectorAll('.profile-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    
    // Adicionar estado ativo ao botão clicado
    const activeBtn = document.querySelector(`[data-profile-id="${profileId}"]`);
    if (activeBtn) {
        activeBtn.classList.add('active');
    }
    
    activeProfileId = profileId;
    
    // Atualizar painel principal
    const profile = profiles.find(p => p.id === profileId);
    if (profile) {
        mainPanel.innerHTML = `
            <div class="welcome-screen">
                <h1>${profile.name}</h1>
                <p>Webview será carregada aqui na Fase 2</p>
            </div>
        `;
    }
    
    console.log('Perfil ativado:', profileId);
    
    // TODO: Na Fase 2, comandar o Tauri para mostrar a webview correspondente
    // await invoke('show_webview', { profileId });
}

// Menu de contexto (clique direito)
document.addEventListener('contextmenu', (e) => {
    const profileBtn = e.target.closest('.profile-btn');
    
    if (profileBtn) {
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
        const profileId = parseInt(currentContextProfileBtn.dataset.profileId);
        const profile = profiles.find(p => p.id === profileId);
        
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

// Log inicial
console.log('Feather Alloy UI carregada');
console.log('Perfis:', profiles);
