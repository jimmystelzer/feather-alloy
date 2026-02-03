# **Especificação Técnica: Agregador de Serviços Web de Alta Performance**

## **1\. Visão Geral**

Aplicação desktop ultra-leve inspirada no Ferdium, desenvolvida em **Rust** com **Tauri 2.0**, utilizando **HTML/CSS/JavaScript** para a interface do usuário e **Wry** para renderização de webviews isoladas dos serviços. O foco é o consumo mínimo de recursos e isolamento total de sessões (multi-perfil).

**Plataformas Suportadas:** Windows e Linux.

## **2\. Pilha Tecnológica**

* **Backend & Core:** Rust (Tauri).  
* **Frontend (UI):** HTML/CSS/JavaScript (interface moderna e responsiva).  
* **Engine Web:** Tauri 2.0 / Wry (Webview nativa do OS).  
* **Isolamento:** WebContext do Tauri para containers de dados separados.  
* **Persistência:** Arquivos JSON para armazenamento de perfis e configurações.  
* **Ícones:** Ícones customizados (PNG/SVG) ou favicons dinâmicos via JavaScript.  
* **Plataformas:** Windows e Linux (usando webview nativa de cada sistema operacional).

## **3\. Arquitetura da Interface (UI Layout)**

### **3.1. Barra de Ferramentas Esquerda (Sidebar)**

* **Largura fixa:** 60-70px.  
* **Composição:**  
  * **Lista de Perfis:** Coluna vertical de botões circulares ou arredondados.  
    * Cada botão representa uma aplicação web.  
    * **Ícone:** Prioridade para ícone customizado (PNG/SVG local). Caso nulo, buscar favicon.ico da URL configurada.  
    * **Interação Esquerda (Clique):** Alterna a visibilidade da WebView correspondente no painel principal através de comandos Tauri.  
    * **Interação Direita (Context Menu):** Abre menu de contexto HTML/CSS com a opção "Editar Perfil".  
  * **Botão Adicionar ("+"):** Abre modal HTML para cadastro de novo serviço (Nome, URL, User-Agent, Ícone).  
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

1. **Fase 1 (Interface Web):** Criar a interface HTML/CSS/JS com a barra lateral esquerda e painel de conteúdo responsivo.  
2. **Fase 2 (Integração Tauri):** Implementar comandos Tauri em Rust para gerenciar webviews isoladas, garantindo que o WebContext seja único por perfil.  
3. **Fase 3 (Persistência JSON):** Implementar salvamento e leitura de perfis em arquivos JSON através de comandos Tauri, com suporte multiplataforma (Windows e Linux).  
4. **Fase 4 (Tray & Lifecycle):** Configurar tauri-plugin-tray e eventos de janela para comportamentos de minimizar/ocultar.  
5. **Fase 5 (UI Polishing):** Implementar menu de contexto, modais de configuração e buscador de favicons via JavaScript.

## **7\. Notas de Performance**

* O uso do **Tauri** com renderização web nativa elimina o overhead do Electron, permitindo que o executável final seja significativamente menor.  
* A interface em HTML/CSS/JS será renderizada na webview principal, mantendo webviews separadas e isoladas para cada serviço configurado.  
* O executável final deve ser significativamente menor que o do Electron (< 20MB vs > 100MB), com menor consumo de memória e CPU.