// src/pages/ChatPage.jsx

import { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';

// Piccolo helper per formattare il nome del gruppo
const formatGroupName = (group, currentUsername) => {
    if (group.is_dm) {
        // È un DM. Trova l'altro utente.
        const otherUser = group.members.find(m => m.username !== currentUsername);
        return otherUser ? `DM: ${otherUser.username}` : 'DM Sconosciuto';
    } else {
        // È un gruppo. Usa il suo nome.
        return group.group_name || 'Gruppo Senza Nome';
    }
};

function ChatPage() {
    const socket = useRef(null);
    const navigate = useNavigate();
    const username = localStorage.getItem('username'); // Per la UI

    // --- NUOVI STATI ---
    const [isConnected, setIsConnected] = useState(false);

    // Lista di tutti i gruppi/DM dell'utente (dal server)
    const [groups, setGroups] = useState([]);

    // L'ID del gruppo attualmente selezionato
    const [activeGroupId, setActiveGroupId] = useState(null);

    // Messaggi (ora è un oggetto: { group_id: [messaggi...] })
    const [messages, setMessages] = useState({});

    // Input per il messaggio di chat
    const [input, setInput] = useState('');

    // Input per creare nuovi gruppi/DM
    const [newChatInput, setNewChatInput] = useState('');
    const [newChatType, setNewChatType] = useState('dm'); // 'dm' o 'group'

    const [inviteInput, setInviteInput] = useState('');

    // --- FINE NUOVI STATI ---

    useEffect(() => {
        if (!username) {
            navigate('/');
            return;
        }

        const token = localStorage.getItem('token');
        socket.current = new WebSocket('ws://127.0.0.1:3000/ws');

        socket.current.onopen = () => {
            console.log('Connesso al server WebSocket!');
            setIsConnected(true);

            // Invia il messaggio di autenticazione
            socket.current.send(JSON.stringify({
                type: "auth",
                token: token
            }));
        };

        socket.current.onclose = () => {
            console.log('Disconnesso.');
            setIsConnected(false);
        };

        // === GESTORE MESSAGGI (MODIFICATO) ===
        socket.current.onmessage = (event) => {
            const msg = JSON.parse(event.data);
            const initialMessages = {};

            switch (msg.type) {
                // Caso 1: Il server ci invia la nostra lista di gruppi
                case 'update_group_list':
                    console.log("Ricevuta lista gruppi:", msg.groups);
                    setGroups(msg.groups);


                    for (const group of msg.groups) {
                        // 'group.messages' è l'array di cronologia dal server
                        // Lo convertiamo nel formato che la UI usa
                        initialMessages[group.group_id] = group.messages.map(chatMsg => ({
                            from: chatMsg.from_username,
                            content: chatMsg.content
                        }));
                    }
                    setMessages(initialMessages);
                    break;

                // Caso 2: Riceviamo un messaggio di chat
                case 'group_chat':
                    console.log("Ricevuto messaggio di chat:", msg);
                    setMessages(prevMessages => ({
                        ...prevMessages,
                        // Aggiungi il messaggio al gruppo corretto
                        [msg.group_id]: [
                            ...(prevMessages[msg.group_id] || []),
                            { from: msg.from_username, content: msg.content }
                        ]
                    }));
                    break;

                // Caso 3: Errore dal server
                case 'error':
                    console.error("Errore dal server:", msg.content);
                    alert(`Errore: ${msg.content}`); // Manteniamo l'alert per informare l'utente

                    // --- NUOVA LOGICA DI REDIRECT ---
                    // Controlliamo se è un errore di autenticazione
                    if (msg.content.includes("Token non valido") || msg.content.includes("autenticarti")) {

                        console.log("Errore di autenticazione, pulizia token e redirect.");

                        // Pulisce i dati errati dal localStorage
                        localStorage.removeItem('token');
                        localStorage.removeItem('username');

                        // Torna alla pagina di login
                        navigate('/');
                    }
                    break;
            }
        };

        // Cleanup
        return () => {
            if (socket.current) socket.current.close();
        };
    }, [username, navigate]);

    // === FUNZIONE PER INVIARE UN MESSAGGIO (MODIFICATA) ===
    const handleSendMessage = () => {
        if (!input.trim() || !activeGroupId) {
            // Non inviare se l'input è vuoto o nessun gruppo è selezionato
            return;
        }

        socket.current.send(JSON.stringify({
            type: "send_group_message",
            group_id: activeGroupId, // Invia l'ID del gruppo attivo
            content: input
        }));
        setInput('');
    };

    // === FUNZIONE PER CREARE UN NUOVO GRUPPO O DM ===
    const handleCreateChat = () => {
        if (!newChatInput.trim()) return;

        if (newChatType === 'dm') {
            // Invia richiesta per creare un DM
            socket.current.send(JSON.stringify({
                type: "create_dm",
                username: newChatInput
            }));
        } else {
            // Invia richiesta per creare un Gruppo
            socket.current.send(JSON.stringify({
                type: "create_group",
                name: newChatInput
            }));
        }
        setNewChatInput('');
    };

    const handleInvite = () => {
        if (!inviteInput.trim() || !activeGroupId) {
            alert("Devi selezionare un gruppo e inserire un username da invitare.");
            return;
        }

        socket.current.send(JSON.stringify({
            type: "invite_to_group",
            group_id: activeGroupId,
            username: inviteInput
        }));
        setInviteInput('');
    };


    const handleLogout = () => {
        console.log("Eseguo il logout...");

        // 1. Chiudi il socket
        if (socket.current) {
            socket.current.close();
        }

        // 2. Pulisci il localStorage
        localStorage.removeItem('token');
        localStorage.removeItem('username');

        // 3. Torna alla pagina di login
        navigate('/');
    };

    // Helper per ottenere i messaggi del gruppo attivo
    const activeMessages = messages[activeGroupId] || [];

    return (
        <div style={{ display: 'flex', height: '100vh', fontFamily: 'Arial, sans-serif' }}>

            {/* Colonna Sinistra: Lista Gruppi e Creazione */}
            <div style={{ width: '300px', borderRight: '1px solid #ccc', display: 'flex', flexDirection: 'column' }}>

                {/* Area Creazione Chat */}
                <div style={{ padding: '10px', borderBottom: '1px solid #ccc' }}>
                    <h3>Nuova Chat</h3>
                    <input
                        type="text"
                        placeholder={newChatType === 'dm' ? "Username utente..." : "Nome gruppo..."}
                        value={newChatInput}
                        onChange={(e) => setNewChatInput(e.target.value)}
                        style={{ width: '90%' }}
                    />
                    <div style={{ marginTop: '5px' }}>
                        <button onClick={() => setNewChatType('dm')}>DM</button>
                        <button onClick={() => setNewChatType('group')}>Gruppo</button>
                        <button onClick={handleCreateChat} style={{ float: 'right' }}>Crea</button>
                    </div>
                </div>

                {/* Area Lista Gruppi */}
                <div style={{ flex: 1, overflowY: 'auto' }}>
                    {groups.map(group => (
                        <div
                            key={group.group_id}
                            onClick={() => setActiveGroupId(group.group_id)}
                            style={{
                                padding: '15px',
                                borderBottom: '1px solid #eee',
                                cursor: 'pointer',
                                backgroundColor: group.group_id === activeGroupId ? '#e0f7fa' : 'transparent'
                            }}
                        >
                            {formatGroupName(group, username)}
                        </div>
                    ))}
                </div>

                <div style={{ padding: '10px', borderTop: '1px solid #ccc' }}>
                    <p>Loggato come: <strong>{username}</strong></p>
                    <p>Stato: {isConnected ? 'Connesso' : 'Disconnesso'}</p>
                    {/* --- PULSANTE AGGIUNTO --- */}
                    <button onClick={handleLogout} style={{ width: '100%', padding: '8px', marginTop: '10px' }}>
                        Logout
                    </button>
                </div>
            </div>

            {/* Colonna Destra: Finestra Chat */}
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>

                {/* Area Messaggi */}
                <div style={{ flex: 1, padding: '20px', overflowY: 'auto', backgroundColor: '#f9f9f9' }}>

                    {/* --- BLOCCO INVITO AGGIUNTO --- */}
                    {activeGroupId && !groups.find(g => g.group_id === activeGroupId)?.is_dm && (
                        <div style={{ paddingBottom: '15px', borderBottom: '1px solid #eee', marginBottom: '15px' }}>
                            <input
                                type="text"
                                placeholder="Username da invitare..."
                                value={inviteInput}
                                onChange={(e) => setInviteInput(e.target.value)}
                                style={{ marginRight: '10px' }}
                            />
                            <button onClick={handleInvite}>Invita</button>
                        </div>
                    )}
                    {/* --- FINE BLOCCO INVITO --- */}

                    {activeGroupId ? (
                        activeMessages.map((msg, index) => (
                            <div key={index} style={{ marginBottom: '10px' }}>
                                <strong style={{ color: msg.from === username ? '#007bff' : '#28a745' }}>
                                    {msg.from}:
                                </strong>
                                <span style={{ marginLeft: '10px' }}>{msg.content}</span>
                            </div>
                        ))
                    ) : (
                        <div style={{ textAlign: 'center', color: '#888', marginTop: '50px' }}>
                            Seleziona una chat per iniziare a messaggiare.
                        </div>
                    )}
                </div>

                {/* Area Input Messaggio */}
                <div style={{ padding: '20px', borderTop: '1px solid #ccc', display: 'flex' }}>
                    <input
                        type="text"
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                        onKeyPress={(e) => e.key === 'Enter' && handleSendMessage()}
                        style={{ flex: 1, padding: '10px', marginRight: '10px' }} // flex: 1 si prende lo spazio
                        placeholder={activeGroupId ? "Scrivi un messaggio..." : "Seleziona una chat"}
                        disabled={!activeGroupId}
                    />
                    <button onClick={handleSendMessage} style={{ padding: '10px 20px' }} disabled={!activeGroupId}>
                        Invia
                    </button>
                </div>
            </div>
        </div>
    );
}

export default ChatPage;