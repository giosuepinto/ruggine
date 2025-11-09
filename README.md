## Progetto Ruggine - Chat App

Questo progetto è l'implementazione della traccia "Ruggine: App di Chat Testuale".
L'architettura scelta prevede un backend in Rust (con Axum / Tokio) e un frontend in React (come Web App).

---

## 🚀 Stato Attuale del Progetto

L'applicazione ha superato la fase di "scheletro" ed è ora un'applicazione di chat funzionale basata su gruppi, con un backend robusto e un'architettura multi-modulo.

---

## ⚙️ Backend (Server Rust)

Il server è stato profondamente ristrutturato per scalabilità e manutenibilità.

1. Architettura
- Multi-Modulo: Il codice non è più in un singolo main.rs. È stato diviso in un'architettura a moduli:
    - src/routes/ (i "Controllers" che gestiscono le richieste HTTP e WS)
    - src/services/ (la logica di business, es. hashing password)
    - src/models.rs (definizioni JSON per la comunicazione Client/Server)
    - src/state.rs (definizione dello stato condiviso dell'app)
    - src/error.rs (gestione centralizzata degli errori)

2. Database (SQLite)
- Migrazioni Automatiche: Il server esegue automaticamente le migrazioni del database (sqlx::migrate!) all'avvio, creando il DB e le tabelle se non esistono.
- Schemi: Oltre alla tabella `users`, il DB ora include:
    - `groups`: Memorizza tutte le "stanze" (sia gruppi che DM).
    - `group_members`: Tabella ponte che collega utenti e gruppi.
    - `messages`: Salva la cronologia di tutti i messaggi inviati.

3. Autenticazione e Sicurezza
- Registrazione: POST /register
    - Validazione Server-Side: Rifiuta registrazioni con username o password vuoti o troppo corti.
- Login: POST /login
    - Sicuro: Non restituisce più HTTP 200 per login falliti. Invia un HTTP 400 con un messaggio generico ("Invalid username or password.") per prevenire la User Enumeration.

4. Gestione WebSocket (GET /ws)
   Questo è il cuore del backend ed è stato completamente riscritto.

- Protocollo JSON: La comunicazione è passata da semplici stringhe a un protocollo JSON strutturato (ClientMessage e ServerMessage).
- Identificazione: Non c'è più un broadcast "pappagallo". Al contrario:
    1. Il client deve inviare un messaggio {"type": "auth", "token": "..."} come prima cosa.
    2. Il server valida il token e associa il socket a un user_id.
- Supporto Multi-Connessione:
    - Risolto il bug critico della "race condition".
    - Lo stato (AppState) ora usa una mappa nidificata: HashMap<UserID, HashMap<SocketID, Sender>>.
    - Ogni connessione riceve un Uuid unico, permettendo a un singolo utente (es. `gio`) di essere connesso da più browser o dispositivi contemporaneamente.
    - Il cleanup alla disconnessione rimuove solo il SocketID specifico, lasciando intatte le altre connessioni dell'utente.
- Logica Gruppi & DM:
    - Il server adotta un'architettura unificata: un Messaggio Diretto (DM) è semplicemente un gruppo con `is_dm: true` e 2 membri.
    - ClientMessage::CreateDM: Crea un nuovo gruppo DM e aggiunge entrambi i membri.
    - ClientMessage::CreateGroup: Crea un gruppo standard, aggiungendo il creatore come primo membro.
    - ClientMessage::InviteToGroup: Aggiunge un nuovo utente a un gruppo esistente (e notifica tutti i membri).
- Logica Messaggi:
    - ClientMessage::SendGroupMessage:
        1. Il server riceve il messaggio.
        2. Lo salva nel database (nella tabella `messages`).
        3. Identifica tutti i membri di quel `group_id`.
        4. Invia il messaggio *solo* ai socket attivi (online) di quei membri.

---

## 🖥️ Frontend (Client React)

L'interfaccia utente è stata completamente ridisegnata per gestire la logica dei gruppi.

1. UI (Layout Multi-Colonna)
- La ChatPage non è più una singola chat, ma un layout in stile Slack/WhatsApp:
    - Colonna Sinistra:
        - Un'area per creare nuovi Gruppi o DM.
        - Un elenco dinamico di tutti i gruppi e DM dell'utente.
        - Un'area utente con il nome e un pulsante di Logout.
    - Colonna Destra:
        - Un'area per invitare utenti al gruppo attualmente selezionato (nascosta per i DM).
        - La finestra della chat che mostra i messaggi.
        - L'area di input del messaggio.

2. Gestione dello Stato e del Protocollo
- Stato: Il client gestisce uno stato centrale che include `groups: []` (l'elenco di tutti i gruppi) e `activeGroupId: null` (la chat attualmente selezionata).
- Flusso di Connessione:
    1. Al login (in LoginPage), salva *sia* `username` che `token` nel `localStorage`.
    2. In ChatPage, invia il messaggio {"type": "auth", "token": ...}.
    3. Riceve un messaggio {"type": "update_group_list", ...} dal server e popola la colonna di sinistra.
- Invio Messaggi: Invia messaggi JSON strutturati (es. `create_dm`, `send_group_message`, `invite_to_group`).
- Gestione Errori:
    - Risolto il bug del "loop di login": un alert per un token non valido ora pulisce il `localStorage` e reindirizza correttamente a `/`.
    - Gli errori di login/registrazione vengono mostrati sotto il modulo nella LoginPage.

3. Correzioni UI/UX
- Risolto il bug di allineamento del pulsante "Invia" (ora usa Flexbox).
- Implementato un pulsante di Logout funzionante.

---

## 🗺️ Evoluzione del Progetto (La Nostra Storia)

1. Fase 1: Scheletro (Il Vecchio README)
    - Autenticazione HTTP (insicura) e un singolo main.rs.
    - Un WebSocket "pappagallo" che inoltrava stringhe a tutti.

2. Fase 2: Architettura e Refactoring
    - Spostamento da un singolo file a un'architettura multi-modulo (routes, services, models, ecc.).
    - Implementazione della migrazione automatica del DB all'avvio.

3. Fase 3: Identificazione e JSON
    - Abbandono delle stringhe per un protocollo JSON completo (ClientMessage/ServerMessage).
    - Implementazione del flusso di autenticazione WebSocket {"type": "auth"}.

4. Fase 4: Multi-Connessione (Risoluzione Bug)
    - Risolto il bug critico della "race condition" (un utente che si riconnetteva "cancellava" la propria connessione).
    - Evoluzione dello AppState per usare Uuid e la mappa nidificata HashMap<UserID, HashMap<SocketID, Sender>>.

5. Fase 5: Logica Gruppi & DM
    - Aggiunte le tabelle `groups`, `group_members`, `messages` al DB.
    - Implementata la logica "DM è un gruppo" nel backend.
    - Sostituito il broadcast globale con un inoltro mirato ai membri del gruppo.
    - Implementata la logica per creare gruppi, DM e invitare utenti.
    - Rifacimento completo della UI del frontend per supportare i gruppi.

6. Fase 6: Sicurezza e Stabilità
    - Corretto il bug catastrofico del login (restituiva sempre HTTP 200).
    - Aggiunta la validazione server-side per la registrazione.
    - Corretti i bug di reindirizzamento e UI nel frontend.
    - Aggiunto il Logout.

---

## 🚧 Cosa Manca (Prossimi Passi)

1. Cronologia Messaggi (Requisito Chiave)
    - Backend: Modificare ClientMessage::Auth per far sì che, dopo l'autenticazione, il server esegua una query al DB e invii la cronologia dei messaggi (es. ultimi 50) per i gruppi dell'utente.
    - Frontend: Gestire questo nuovo ServerMessage::ChatHistory e usarlo per popolare lo stato `messages` al caricamento, risolvendo il problema della chat vuota al refresh.

2. Miglioramenti UI/UX
    - Invio con "Invio": Aggiungere un onKeyPress all'input dei messaggi per inviare con il tasto Invio.
    - Barra Info Gruppo: Aggiungere una barra in cima alla finestra della chat (colonna destra) che mostri "Parlando con: Ale" (per i DM) o "Gruppo: Test Inviti (gio, ale, fra)" (per i gruppi).

3. Requisiti Finali del Progetto
    - Logging CPU: Aggiungere `sysinfo` al backend e avviare un task che registri l'uso della CPU ogni 2 minuti in un file di log.
    - Dimensione Eseguibile: Compilare il server in modalità `release` (cargo build --release) e annotare la dimensione finale dell'eseguibile in questo README.