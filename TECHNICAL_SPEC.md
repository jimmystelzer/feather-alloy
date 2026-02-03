# **Especificação Técnica: Agregador de Serviços Web de Alta Performance**

## **1\. Visão Geral**

Aplicação desktop ultra-leve inspirada no Ferdium, desenvolvida em **Rust**, utilizando **Iced** para a interface nativa e **Tauri 2.0 (wry)** para renderização web isolada. O foco é o consumo mínimo de recursos e isolamento total de sessões (multi-perfil).

## **2\. Pilha Tecnológica**

* **Backend & Core:** Rust.  
* **Frontend (UI):** Iced (Interface nativa via GPU).  
* **Engine Web:** Tauri 2.0 / Wry (Webview nativa do OS).  
* **Isolamento:** WebContext do Tauri para containers de dados separados.  
* **Ícones:** iced\_aw para ícones de fonte ou carregamento dinâmico de SVGs/Favicons.

## **3\. Arquitetura da Interface (UI Layout)**

### **3.1. Barra de Ferramentas Esquerda (Sidebar)**

* **Largura fixa:** 60-70px.  
* **Composição:**  
  * **Lista de Perfis:** Coluna vertical de botões circulares ou arredondados.  
    * Cada botão representa uma aplicação web.  
    * **Ícone:** Prioridade para ícone customizado (PNG/SVG local). Caso nulo, buscar favicon.ico da URL configurada.  
    * **Interação Esquerda:** Alterna a visibilidade da WebView correspondente no painel principal.  
    * **Interação Direita (Context Menu):** Abre menu flutuante (Iced Overlay) com a opção "Editar Perfil".  
  * **Botão Adicionar ("+"):** Abre modal para cadastro de novo serviço (Nome, URL, User-Agent, Ícone).  
  * **Botão Configurações (Engrenagem):** Posicionado na base da barra lateral.

### **3.2. Painel de Conteúdo (Main View)**

* Área adjacente à barra lateral que ocupa o restante da janela.  
* Atua como container para as Webviews. Apenas uma Webview é visível por vez (as demais permanecem em estado de suspensão ou ocultas para economizar recursos).

## **4\. Funcionalidades e Comportamento**

### **4.1. Isolamento de Perfis (Multi-Instância)**

* Cada aplicação criada gera um id\_perfil único.  
* O diretório de dados (data\_directory) no Rust deve ser mapeado como:  
  app\_data\_dir/profiles/{id\_perfil}/.  
* Isso permite rodar múltiplas instâncias do WhatsApp, Gmail ou Teams sem conflito de cookies.

### **4.2. Configurações da Aplicação**

A tela de configurações (ícone de engrenagem) deve gerenciar:

* **Minimizar ao Abrir:** Inicia a aplicação ocultada na bandeja.  
* **Minimizar ao Fechar:** O botão "X" da janela não encerra o processo, apenas minimiza.  
* **Ocultar ao Fechar:** O botão "X" esconde a janela completamente (acessível apenas via Tray).  
* **Ícone de Bandeja (System Tray):**  
  * Ícone persistente na área de notificação.  
  * Clique simples: Restaura/Foca a aplicação.  
  * Menu de contexto: Sair da aplicação.

### **4.3. Gerenciamento de Memória (Hibernação)**

* Implementar lógica em Rust para detectar Webviews inativas por mais de X minutos.  
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

## **6\. Fluxo de Implementação Recomendado**

1. **Fase 1 (Iced Shell):** Criar a janela principal com a barra lateral esquerda e botões estáticos.  
2. **Fase 2 (Tauri Multi-Webview):** Integrar a criação de Webview controlada pelo Iced, garantindo que o WebContext seja único por perfil.  
3. **Fase 3 (Persistência):** Implementar salvamento de perfis em JSON ou SQLite.  
4. **Fase 4 (Tray & Lifecycle):** Configurar o tauri-plugin-shell e tauri-plugin-tray para os comportamentos de minimizar/ocultar.  
5. **Fase 5 (UI Polishing):** Implementar o menu de clique direito e o buscador de favicons.

## **7\. Notas de Performance**

* O uso do **Iced** elimina o overhead de ter uma "WebView para renderizar a UI da aplicação", deixando o Chromium/WebKit focado apenas nos serviços desejados.  
* O executável final deve ser significativamente menor que o do Electron (\< 20MB vs \> 100MB).