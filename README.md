# Progetto Ruggine - Chat App

Questo progetto è l'implementazione della traccia **"Ruggine: App di Chat Testuale"**.  
L'architettura scelta prevede un **backend in Rust** (con [Axum](https://docs.rs/axum/latest/axum/) / [Tokio](https://tokio.rs/)) e un **frontend in React** (come Web App).

---

## 🧩 Stato Attuale del Progetto

Attualmente, il progetto è uno **scheletro funzionante**.  
Abbiamo:
- un sistema di **autenticazione HTTP**
- una **chat in tempo reale via WebSocket**, che funziona come una singola “stanza” globale.

---

## ⚙️ Backend (Server Rust)

Il server è in grado di:

### 🔐 Gestire Utenti
- Si connette a un database **SQLite** (`ruggine.db`).
- **Registrazione** → `POST /register`: registra nuovi utenti con password sicure (hashing via **argon2**).
- **Login** → `POST /login`: autentica gli utenti esistenti.

### 💬 Gestione WebSocket
- **Endpoint:** `GET /ws`
- Accetta connessioni WebSocket.
- Effettua un **broadcast globale**: qualsiasi messaggio ricevuto su qualsiasi WebSocket viene inoltrato a **tutti i client connessi** in quel momento (“effetto pappagallo”).

---

## 🖥️ Frontend (Client React)

Il client è una **Single Page Application (SPA)** che:

### 🧭 Gestisce le Rotte
Utilizza **react-router-dom** per due pagine principali:
- `/` → Pagina di **Login/Registrazione**
- `/chat` → Pagina della **Chat**

### 🔑 Autenticazione
- Permette agli utenti di registrarsi e accedere tramite gli endpoint HTTP del server.
- Salva l’**username** nel `localStorage` dopo il login per ricordarsi dell’utente.

### 🚧 Route Protection
- Se un utente tenta di accedere a `/chat` senza essere loggato, viene **reindirizzato** automaticamente alla pagina di login.

### ⚡ Chat in Tempo Reale
- Si connette al server via **WebSocket**.
- Invia messaggi formattati come `username: messaggio`.
- Riceve i messaggi inoltrati dal server e li mostra in tempo reale.

---

## 🚧 Cosa Manca (Prossimi Passi)

Il sistema attuale presenta una **lacuna fondamentale**:  
👉 **l’identificazione sugli WebSocket.**

### ❗ Il Problema
Il server sa autenticare un utente via HTTP, ma quando lo stesso utente apre una connessione WebSocket,  
**non sa chi sia** — vede solo una nuova connessione anonima.

### 🎯 L’Obiettivo
Insegnare al server a **collegare una connessione WebSocket a un ID utente**.

---

## 🗺️ Piano d’Azione

### 1. Frontend
- Dopo il login (in `LoginPage`), salvare nel `localStorage` **non solo l’username**, ma anche un **token** (per ora può essere l’username stesso, in futuro un JWT).
- Quando il client (`ChatPage.jsx`) apre la connessione WebSocket, il **primo messaggio** inviato al server deve essere un messaggio di **autenticazione**, ad esempio:

```json
{ "type": "auth", "token": "mario" }
```

### 2. Backend

Quando il server riceve il primo messaggio su una nuova connessione WebSocket:

- Legge il JSON ricevuto.
- Verifica il token.
- Salva l’associazione, ad esempio in una HashMap<UserId, WebSocketSender>.

Una volta fatto questo, potremo abbandonare il broadcast globale.
Quando il server riceve un messaggio da mario, 
saprà chi è e potrà decidere a chi inviarlo (es. solo ai membri di un gruppo specifico).