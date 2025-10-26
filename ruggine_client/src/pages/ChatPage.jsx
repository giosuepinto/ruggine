// src/pages/ChatPage.jsx
import { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';

function ChatPage() {
    const socket = useRef(null);
    const [messages, setMessages] = useState([]);
    const [input, setInput] = useState('');
    const [isConnected, setIsConnected] = useState(false);

    const navigate = useNavigate();

    // NOTA: Prendiamo il nome utente salvato!
    const username = localStorage.getItem('username');

    useEffect(() => {
        // Se non c'è username, non dovremmo essere qui. Torna al login.
        if (!username) {
            navigate('/');
            return;
        }

        socket.current = new WebSocket('ws://127.0.0.1:3000/ws');

        socket.current.onopen = () => {
            console.log('Connesso al server WebSocket!');
            setIsConnected(true);
            // *** STEP FUTURO: QUI DOVREMMO AUTENTICARE IL SOCKET ***
            // Per ora, ci connettiamo e basta.
        };

        socket.current.onclose = () => {
            console.log('Disconnesso dal server WebSocket.');
            setIsConnected(false);
        };

        socket.current.onmessage = (event) => {
            setMessages((prevMessages) => [...prevMessages, event.data]);
        };

        return () => {
            if (socket.current) socket.current.close();
        };
    }, [username, navigate]);

    const sendMessage = () => {
        if (input.trim() && socket.current && socket.current.readyState === WebSocket.OPEN) {
            // *** MODIFICA: Inviamo un JSON! ***
            // In futuro, il server userà 'username' per identificare il mittente.
            // Per ora, lo usiamo per il prefisso del messaggio.
            const payload = {
                from: username,
                content: input,
            };

            // NOTA: il nostro server attuale non sa leggere i JSON.
            // Leggerà [object Object]. Lo sistemeremo DOPO.
            // Per ora, facciamo come prima MA usiamo il nome utente.

            // Vecchio modo, ma con 'username' (funzionerà con l'ultimo main.rs)
            socket.current.send(`${username}: ${input}`);

            setInput('');
        }
    };

    return (
        <div className="App">
            <h1>Chat Ruggine - Loggato come: {username}</h1>
            <p>Stato Connessione: {isConnected ? 'Connesso' : 'Disconnesso'}</p>
            <div className="chat-window" style={{ border: '1px solid #ccc', height: '300px', overflowY: 'scroll', padding: '10px' }}>
                {messages.map((msg, index) => (
                    <div key={index}>{msg}</div>
                ))}
            </div>
            <div className="input-area">
                <input
                    type="text"
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
                />
                <button onClick={sendMessage}>Invia</button>
            </div>
        </div>
    );
}

export default ChatPage;