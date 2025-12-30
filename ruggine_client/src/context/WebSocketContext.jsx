// src/context/WebSocketContext.jsx
import React, { createContext, useContext, useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';

const WebSocketContext = createContext(null);

export const WebSocketProvider = ({ children }) => {
    const socket = useRef(null);
    const navigate = useNavigate();

    // Stati Globali
    const [isConnected, setIsConnected] = useState(false);
    const [groups, setGroups] = useState([]);
    const [invitations, setInvitations] = useState({ received: [], sent: [] });
    const [messages, setMessages] = useState({});
    const [user, setUser] = useState(null);
    const [wsError, setWsError] = useState(null);

    // Login al montaggio (per gestire il refresh della pagina)
    useEffect(() => {
        // MODIFICATO: Usa sessionStorage
        const token = sessionStorage.getItem('token');
        const savedUsername = sessionStorage.getItem('username');

        if (token && savedUsername) {
            setUser(savedUsername);
            connect(token);
        }
    }, []);

    const connect = (token) => {
        if (socket.current) return;

        // Ricorda di cambiare l'IP per la demo LAN!
        socket.current = new WebSocket('ws://127.0.0.1:3000/ws');

        socket.current.onopen = () => {
            console.log('WS Connesso');
            setIsConnected(true);
            socket.current.send(JSON.stringify({ type: "auth", token }));
        };

        socket.current.onclose = () => {
            console.log('WS Disconnesso');
            setIsConnected(false);
            socket.current = null;
        };

        socket.current.onmessage = (event) => {
            const msg = JSON.parse(event.data);
            handleMessage(msg);
        };
    };

    const handleMessage = (msg) => {
        switch (msg.type) {
            case 'update_group_list':
                setGroups(msg.groups);
                const initialMsgs = {};
                msg.groups.forEach(g => {
                    initialMsgs[g.group_id] = g.messages.map(m => ({
                        from: m.from_username,
                        content: m.content
                    }));
                });
                setMessages(prev => ({ ...initialMsgs, ...prev }));
                break;

            case 'update_invitations_list':
                setInvitations({
                    received: msg.received || [],
                    sent: msg.sent || []
                });
                break;

            case 'group_chat':
                setMessages(prev => ({
                    ...prev,
                    [msg.group_id]: [
                        ...(prev[msg.group_id] || []),
                        { from: msg.from_username, content: msg.content }
                    ]
                }));
                break;

            case 'error':
                if (msg.content.includes("Token") || msg.content.includes("autenticarti")) {
                    logout();
                } else {
                    setWsError(msg.content);
                }
                break;
            default:
                break;
        }
    };

    // --- FUNZIONE LOGIN AGGIORNATA ---
    const login = (username, token) => {
        // MODIFICATO: Usa sessionStorage
        sessionStorage.setItem('username', username);
        sessionStorage.setItem('token', token);
        setUser(username);
        connect(token);
    };

    // --- FUNZIONE LOGOUT AGGIORNATA ---
    const logout = () => {
        if (socket.current) socket.current.close();
        // MODIFICATO: Pulisce solo la sessione corrente
        sessionStorage.clear();
        socket.current = null;
        setUser(null);
        setGroups([]);
        setInvitations({ received: [], sent: [] });
        setMessages({});
        setIsConnected(false);
        navigate('/');
    };

    // --- AZIONI (Invariate) ---
    const sendMessage = (groupId, content) => {
        if (socket.current) {
            socket.current.send(JSON.stringify({ type: "send_group_message", group_id: groupId, content }));
        }
    };
    const createChat = (type, name) => {
        const msgType = type === 'dm' ? "create_dm" : "create_group";
        const payload = type === 'dm' ? { username: name } : { name };
        socket.current?.send(JSON.stringify({ type: msgType, ...payload }));
    };
    const sendInvite = (groupId, username) => {
        socket.current?.send(JSON.stringify({ type: "invite_to_group", group_id: groupId, username }));
    };
    const respondToInvite = (invitationId, accept) => {
        socket.current?.send(JSON.stringify({ type: "handle_invitation", invitation_id: invitationId, accept }));
    };

    const value = {
        user,
        isConnected,
        groups,
        invitations,
        messages,
        login,
        logout,
        connect,
        sendMessage,
        createChat,
        sendInvite,
        respondToInvite,
        wsError,
        setWsError
    };

    return (
        <WebSocketContext.Provider value={value}>
            {children}
        </WebSocketContext.Provider>
    );
};

export const useWebSocket = () => useContext(WebSocketContext);