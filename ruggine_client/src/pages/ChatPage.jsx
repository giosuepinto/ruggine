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

            // === NUOVA LOGICA DI AUTENTICAZIONE ===
            const token = localStorage.getItem('token');
            if (token) {
                socket.current.send(JSON.stringify({
                    type: "auth",
                    token: token
                }));
            } else {
                console.error("Token non trovato, impossibile autenticarsi.");
                navigate('/'); // Torna al login se non c'è il token
            }
        };

        socket.current.onclose = () => {
            console.log('Disconnesso dal server WebSocket.');
            setIsConnected(false);
        };

        socket.current.onmessage = (event) => {
            const message = JSON.parse(event.data);

            if (message.type === "chat") {
                // Formattiamo il messaggio per la visualizzazione
                const formattedMessage = `${message.from}: ${message.content}`;
                setMessages((prevMessages) => [...prevMessages, formattedMessage]);
            } else if (message.type === "info") {
                // Messaggio di info (es. "Utente X si è connesso")
                setMessages((prevMessages) => [...prevMessages, `[INFO] ${message.content}`]);
            }
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

            // === NUOVO MODO (JSON) ===
            socket.current.send(JSON.stringify({
                type: "chat",
                content: input
            }));

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