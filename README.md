# Ruggine: App di Chat Testuale

## 1. Introduzione

Il progetto **Ruggine** è un'applicazione di messaggistica istantanea Client/Server sviluppata per il corso di Programmazione di Sistema.
L'obiettivo è fornire una piattaforma performante, scalabile e sicura per lo scambio di messaggi testuali in tempo reale, supportando chat private (DM), gruppi e una chat globale persistente.

L'architettura si basa su un backend in **Rust** (focalizzato su sicurezza e concorrenza) e un frontend in **React** (per una user experience moderna).

---

## 2. Architettura del Sistema

Il sistema segue un'architettura a tre livelli (Three-Tier Architecture) con comunicazione ibrida HTTP/WebSocket.
```text
[ FRONTEND: React + Vite ]
^      ^
|      |
(HTTP/REST) (WebSocket JSON)
|      |
v      v
[ BACKEND: Rust + Axum ]
|
+--- [ Controllers ] (Gestione I/O e Routing)
|         |
|         v
+--- [ Service Layer ] (Logica di Business: Auth, Inviti, Chat)
|         |
|         v
+--- [ Repository Layer ] (Query SQL e Accesso Dati)
|
v
[ DATABASE: SQLite ]
```

### Tecnologie Utilizzate

**Backend:**
* **Linguaggio:** Rust
* **Framework Web:** Axum (basato su Tokio per l'asincronia)
* **Database:** SQLite (tramite driver SQLx con query asincrone)
* **Sicurezza:** Argon2 (hashing password), JSON Web Token (JWT)
* **Monitoraggio:** Sysinfo (logging CPU)

**Frontend:**
* **Framework:** React
* **Build Tool:** Vite
* **Stato:** React Context API (`WebSocketContext`)
* **Navigazione:** React Router DOM

---

## 3. Struttura del Progetto (Backend)

Il backend è stato rifattorizzato passando da un singolo file monolitico a una **Layered Architecture** modulare per garantire manutenibilità e separazione delle responsabilità.

### File System
* `src/main.rs`: Entry point. Configura il logging, la connessione al DB, avvia il task di monitoraggio CPU e il server HTTP.
* `src/router.rs`: Definisce le rotte HTTP (`/login`, `/register`) e WebSocket (`/ws`).
* `src/state.rs`: Gestisce lo stato condiviso in memoria (connessioni attive) thread-safe tramite `Arc<RwLock<...>>`.
* `src/error.rs`: Gestione centralizzata degli errori con mapping su codici HTTP.

### Moduli Funzionali
1.  **Controllers (`src/controllers/`)**: Gestiscono l'I/O. Ricevono le richieste (HTTP o messaggi WS), validano l'input basilare e chiamano i servizi. Non contengono logica di business.
2.  **Services (`src/services/`)**: Contengono la logica di business pura (es. regole per gli inviti, hashing password, gestione cooldown).
3.  **Repositories (`src/repositories/`)**: Eseguono le query SQL pure. Nessuna logica di business qui, solo accesso ai dati.
4.  **DTOs (`src/DTOs/`)**: Definiscono le strutture dati JSON per la comunicazione Client-Server (es. `ClientMessage`, `ServerMessage`).

---

## 4. Scelte Implementative e Funzionalità Chiave

### 4.1 Gestione Connessioni e Multi-Device
Il server supporta connessioni multiple per lo stesso utente (es. PC e Telefono contemporaneamente).
* **Struttura:** `HashMap<UserId, HashMap<SocketUUID, Sender>>`.
* Ogni tab/dispositivo ha un UUID univoco. Quando un utente si disconnette, viene rimosso solo quel socket specifico, mantenendo attive le altre sessioni.

### 4.2 Protocollo di Comunicazione
La comunicazione avviene tramite messaggi JSON tipizzati.
* **Client -> Server:** `Auth`, `CreateGroup`, `InviteToGroup`, `HandleInvitation`, `SendGroupMessage`.
* **Server -> Client:** `UpdateGroupList` (include cronologia), `UpdateInvitationsList`, `GroupChat`, `Error`.

### 4.3 Sistema di Inviti e Cooldown (Logica Avanzata)
Per evitare spam e gestire la privacy, la creazione di chat non è immediata (eccetto per i propri gruppi).
1.  **Invio:** Un utente invia una richiesta (`InviteToGroup` o `CreateDM`).
2.  **Pending:** L'invito viene salvato nel DB con stato `pending`.
3.  **Notifica:** Il destinatario vede l'invito in tempo reale nella pagina dedicata.
4.  **Decisione:**
    * **Accetta:** L'utente viene aggiunto al gruppo e la chat appare nella lista.
    * **Rifiuta:** L'invito passa a `rejected`.
5.  **Cooldown:** Se un invito viene rifiutato, il sistema impedisce nuovi inviti tra le stesse parti per **2 minuti**. Il controllo viene effettuato lato server confrontando il timestamp `updated_at` del rifiuto.

### 4.4 Sicurezza
* **Autenticazione:** Utilizzo di **JWT (JSON Web Token)** firmati dal server. Il frontend non salva mai la password, ma solo il token.
* **WebSocket Auth:** La prima azione del socket è l'invio del token JWT per l'identificazione.
* **Validazione:** Controlli server-side su lunghezza password, username vuoti e tentativi di impersonificazione.
* **Password Hashing:** Utilizzo dell'algoritmo **Argon2** con salt casuale per ogni utente.

### 4.5 Persistenza e Cronologia
* Tutti i dati (Utenti, Gruppi, Membri, Inviti, Messaggi) sono persistiti su **SQLite**.
* Al login, il server recupera automaticamente gli ultimi 50 messaggi per ogni chat e li invia al client, garantendo la continuità della conversazione anche dopo un refresh della pagina.

### 4.6 Monitoraggio Risorse
Come da requisito, il server avvia un task asincrono (`tokio::spawn`) che utilizza la libreria `sysinfo` per campionare l'utilizzo della CPU ogni **2 minuti** e scriverlo nel file `server_cpu_stats.log`.

---

## 5. Manuale Utente (Frontend)

### Installazione e Avvio
1.  **Backend:**
    ```bash
    cd ruggine_server
    cargo run --release
    ```
    *(Al primo avvio il server creerà automaticamente il DB e applicherà le migrazioni).*

2.  **Frontend:**
    ```bash
    cd ruggine_client
    npm install
    npm run dev
    ```

### Guida all'Uso
1.  **Login/Registrazione:** Interfaccia unificata. Inserire credenziali valide (min 3 caratteri user, min 4 pass).
2.  **Home (Le mie Chat):**
    * A sinistra: Lista delle chat attive e Chat Globale.
    * Pulsante **`+`**: Apre il pannello per creare un nuovo Gruppo o richiedere un DM.
    * A destra: Finestra di chat.
3.  **Inviti:**
    * Accedere alla sezione "Inviti" dalla Navbar.
    * Visualizzare richieste inviate (con stato Pending/Rejected) e ricevute.
    * Accettare o Rifiutare le richieste in arrivo.

---

## 6. Dati Tecnici TODO!

* **Dimensione Eseguibile (Release):** [INSERIRE QUI LA DIMENSIONE] (su architettura macOS/Linux a 64bit).
* **Utilizzo CPU average:** TODO
* **Database:** Singolo file `ruggine.db` portabile.

---

## 7. Conclusioni

Il progetto soddisfa tutti i requisiti funzionali e non funzionali della traccia. L'adozione di Rust ha garantito prestazioni elevate e memory safety, mentre l'architettura a micro-componenti di React ha permesso di creare un'interfaccia utente reattiva e complessa (gestione stati inviti/chat in tempo reale).