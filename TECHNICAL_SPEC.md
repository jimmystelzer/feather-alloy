# **Especificação Técnica: Agregador de Serviços Web de Alta Performance**

## **1\. Visão Geral**

Aplicação desktop ultra-leve inspirada no Ferdium, desenvolvida em **Rust** com **WRY + Tao**, utilizando **HTML/CSS/JavaScript** para a interface do usuário e **WRY** para renderização de webviews isoladas dos serviços. O foco é o consumo mínimo de recursos e isolamento total de sessões (multi-perfil).

**Decisão Arquitetural:** Optamos por usar WRY + Tao diretamente ao invés de Tauri para ter controle fino sobre o gerenciamento de múltiplas webviews dentro de uma única janela, permitindo um layout dual-pane (toolbar + content) com webviews embutidas.

**Plataformas Suportadas:** Windows e Linux.

## **2. Pilha Tecnológica**

* **Backend & Core:** Rust puro com WRY + Tao.
* **Window Management:** Tao (fork do Winit otimizado para webviews).
* **WebView Engine:** WRY 0.54 (wrapper cross-platform para webviews nativas).
* **Frontend (UI):** HTML/CSS/JavaScript inline (sem bundler, carregado via `include_str!`).
* **Isolamento:** `WebContext::new(Some(data_dir))` do WRY para containers de dados separados por perfil.
* **Persistência:** Arquivos JSON para armazenamento de perfis e configurações.
* **IPC:** Sistema customizado usando `window.ipc.postMessage()` e `EventLoopProxy<AppEvent>`.
* **Ícones:** Ícones customizados (PNG/SVG) ou favicons dinâmicos via JavaScript.
* **Plataformas:** 
  - **Linux:** WebKitGTK 4.1
  - **Windows:** WebView2 (planejado)


## **3\. Arquitetura da Interface (UI Layout)**

### **3.1. Barra de Ferramentas Esquerda (Sidebar)**

* **Largura fixa:** 60-70px.  
* **Composição:**  
  * **Lista de Perfis:** Coluna vertical de botões circulares ou arredondados.  
    * Cada botão representa uma aplicação web.  
    * **Ícone:** 
      * Usuário pode fazer upload de ícone customizado (salvo na pasta do perfil).
      * Pode remover o ícone customizado.
      * Se não houver ícone customizado, o sistema tenta baixar o favicon da URL configurada.
      * O ícone é persistido em `app_data_dir/profiles/{uuid}/icon.{ext}` ou `favicon.ico`.
    * **Interação Esquerda (Clique):** Alterna a visibilidade da WebView correspondente no painel principal através de comandos Tauri.  
    * **Interação Direita (Context Menu)::** Abre menu de contexto HTML/CSS com as opções: "Atualizar conteúdo", "Atualizar ícone", "Editar Perfil" e "Remover Perfil".  
  * **Botão Adicionar ("+"):** Abre modal HTML para cadastro de novo serviço (Nome, URL, User-Agent, Upload de Ícone).  
  * **Botão Configurações (Engrenagem):** Posicionado na base da barra lateral.

### **3.2. Painel de Conteúdo (Main View)**

* Área adjacente à barra lateral que ocupa o restante da janela.
* **Implementação:** Múltiplas webviews WRY embutidas como child webviews da janela principal.
* **Gerenciamento de Visibilidade:** 
  - Todas as webviews de perfis permanecem ativas em background (para receber notificações).
  - Apenas uma webview é visível por vez usando `webview.set_visible(true/false)`.
  - Troca instantânea entre perfis sem recarregamento.
* **Isolamento:** Cada perfil tem seu próprio `WebContext` com diretório de dados separado em `~/.local/share/feather-alloy/profiles/{uuid}/`.


## **4\. Funcionalidades e Comportamento**

### **4.1. Isolamento de Perfis (Multi-Instância)**

* Cada aplicação criada gera um id\_perfil único.  
* O diretório de dados (data\_directory) no Rust deve ser mapeado como:  
  app\_data\_dir/profiles/{id\_perfil}/.  
* Isso permite rodar múltiplas instâncias do WhatsApp, Gmail ou Teams sem conflito de cookies.
* O ícone de cada aplicação deve ser persistido em disco e referenciado no arquivo de configuração do perfil.

### **4.2. Configurações da Aplicação**

A tela de configurações (ícone de engrenagem) deve gerenciar:

* **Minimizar ao Abrir:** Inicia a aplicação ocultada na bandeja ou minimizada (depende da configuração de minimizar ao fechar e ocultar ao fechar que são excludentes).  
* **Minimizar ao Fechar:** O botão "X" (fechar da janela) não encerra o processo, apenas minimiza.  
* **Ocultar ao Fechar:** O botão "X" (fechar da janela) esconde a janela completamente (acessível apenas via Tray).
* **Fechar Feather Alloy:** Botão para fechar a aplicação/janela encerrando o processo (mesmo se as configurações de minimizar ao fechar e ocultar ao fechar estiverem habilitadas).  
* **Ícone de Bandeja (System Tray):**  
  * Ícone persistente na área de notificação (usar ícone do Feather Alloy contido na aplicação ou na pasta icons).  
  * Clique simples: inverte o estado de ocultar/minimizar da aplicação.  


### **4.3. Gerenciamento de Memória (Hibernação)**

* Utilizar sinais do SO para reduzir o conjunto de trabalho (working set) das Webviews em background.

## **5\. Estrutura de Dados (Configuração)**

// Estrutura para os Perfis  
struct WebProfile {  
    uuid: Uuid,  
    name: String,  
    url: String,  
    icon\_path: Option\<PathBuf\>,  
    user\_agent: String,  
    auto\_hibernate: bool,  
}

// Configurações Globais  
struct AppSettings {  
    minimize\_on\_open: bool,  
    minimize\_on\_close: bool,  
    hide\_on\_close: bool,  
    enable\_tray: bool,  
}

## **6. Fluxo de Implementação Recomendado**

1. **Fase 1 (Interface Web):** ✅ Criar a interface HTML/CSS/JS com a barra lateral esquerda e painel de conteúdo responsivo.
2. **Fase 2 (Integração WRY + Tao):** ✅ Implementar gerenciamento de webviews usando WRY diretamente, com layout dual-pane (toolbar + content) e WebContext isolado por perfil.
3. **Fase 3 (Persistência JSON):** ✅ Implementar salvamento e leitura de perfis em arquivos JSON através do sistema de estado compartilhado (`Arc<Mutex<Vec<WebProfile>>>`).
4. **Fase 4 (Tray & Lifecycle):** ✅ Configurar system tray e eventos de janela para comportamentos de minimizar/ocultar.
5. **Fase 5 (UI Polishing):** 🔄 Implementar menu de contexto, modais de configuração e buscador de favicons via JavaScript.

**Status Atual:** Fases 1-4 concluídas. Sistema de tray icon funcionando como toggle e gerenciamento de ciclo de vida da janela implementado.


## **7. Notas de Performance**

* O uso de **WRY + Tao** diretamente (sem Tauri) permite controle fino sobre o gerenciamento de webviews e elimina overhead desnecessário.
* **Webviews Persistentes:** Todas as webviews de perfis permanecem ativas em background, permitindo recebimento de notificações mesmo quando ocultas.
* **Isolamento Completo:** Cada perfil tem seu próprio `WebContext` com diretório de dados separado, garantindo isolamento total de cookies, localStorage e cache.
* **Troca Instantânea:** Alternância entre perfis usando apenas `set_visible()` ao invés de recriar webviews, resultando em navegação instantânea.
* A interface em HTML/CSS/JS é carregada inline via `include_str!`, eliminando necessidade de bundler ou servidor HTTP.
* O executável final deve ser significativamente menor que o do Electron (< 20MB vs > 100MB), com menor consumo de memória e CPU.
* **Arquitetura Atual:** 
  - 1 janela Tao
  - 1 webview toolbar (70px, sempre visível)
  - 1 webview welcome (para tela inicial e formulários)
  - N webviews de perfis (uma por perfil configurado, alternando visibilidade)