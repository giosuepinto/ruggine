// src/pages/InvitesPage.jsx
import React, { useState } from 'react';
import { useWebSocket } from '../context/WebSocketContext';
import '../App.css';

function InvitesPage() {
    // Estraiamo invitations (che ora è un oggetto {received, sent})
    const { invitations, respondToInvite, createChat } = useWebSocket();

    const [newChatInput, setNewChatInput] = useState('');
    const [newChatType, setNewChatType] = useState('dm'); // 'dm' | 'group'

    const handleCreate = () => {
        if (!newChatInput.trim()) return;
        createChat(newChatType, newChatInput);
        setNewChatInput('');
    };

    // Helper per gestire il caso in cui invitations fosse undefined all'inizio
    const receivedList = invitations.received || [];
    const sentList = invitations.sent || [];

    return (
        <div className="invites-container" style={{ padding: '2rem', maxWidth: '1000px', margin: '0 auto', display: 'flex', gap: '40px', height: 'calc(100vh - 80px)' }}>

            {/* COLONNA SINISTRA: CREA E SENT */}
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: '20px' }}>

                {/* Box Creazione */}
                <div style={{ background: 'white', padding: '25px', borderRadius: '12px', boxShadow: '0 4px 15px rgba(0,0,0,0.05)' }}>
                    <h3 style={{ marginTop: 0, color: '#667eea', borderBottom: '2px solid #f0f2f5', paddingBottom: '10px' }}>Nuova Connessione</h3>
                    <p style={{ fontSize: '0.9rem', color: '#666', marginBottom: '15px' }}>
                        Cerca un utente per iniziare una chat privata o crea un nuovo gruppo di lavoro.
                    </p>

                    <div style={{ display: 'flex', gap: '10px', marginBottom: '15px' }}>
                        <button
                            className={`mini-btn ${newChatType === 'dm' ? 'active' : ''}`}
                            onClick={() => setNewChatType('dm')}
                            style={{ flex: 1, padding: '10px', fontSize: '0.9rem' }}
                        >
                            DM Utente
                        </button>
                        <button
                            className={`mini-btn ${newChatType === 'group' ? 'active' : ''}`}
                            onClick={() => setNewChatType('group')}
                            style={{ flex: 1, padding: '10px', fontSize: '0.9rem' }}
                        >
                            Nuovo Gruppo
                        </button>
                    </div>

                    <div style={{ display: 'flex', gap: '10px' }}>
                        <input
                            type="text"
                            className="login-input"
                            placeholder={newChatType === 'dm' ? "Inserisci username..." : "Nome del gruppo..."}
                            value={newChatInput}
                            onChange={e => setNewChatInput(e.target.value)}
                            style={{ marginBottom: 0, flex: 1 }}
                        />
                        <button className="login-button" onClick={handleCreate} style={{ width: 'auto', marginTop: 0, padding: '0 20px' }}>
                            {newChatType === 'dm' ? 'Invia' : 'Crea'}
                        </button>
                    </div>
                </div>

                {/* Lista Inviati (Feedback Visivo) */}
                <div style={{ flex: 1, overflowY: 'auto' }}>
                    <h4 style={{ color: '#555', marginBottom: '15px', display: 'flex', alignItems: 'center', gap: '10px' }}>
                        Richieste Inviate
                        <span style={{ background: '#eee', padding: '2px 8px', borderRadius: '10px', fontSize: '0.8rem' }}>{sentList.length}</span>
                    </h4>

                    {sentList.length === 0 ? (
                        <div style={{ color: '#999', fontSize: '0.9rem', fontStyle: 'italic' }}>Nessuna richiesta in attesa.</div>
                    ) : (
                        <div className="sent-list">
                            {sentList.map(inv => (
                                <div key={inv.id} style={{
                                    background: '#fff', padding: '12px', borderRadius: '8px', marginBottom: '10px',
                                    borderLeft: inv.status === 'rejected' ? '4px solid #ff4757' : '4px solid #ffa502',
                                    boxShadow: '0 2px 5px rgba(0,0,0,0.03)'
                                }}>
                                    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '5px' }}>
                                        <span style={{ fontWeight: 'bold', color: '#333' }}>A: {inv.partner_username}</span>
                                        <span style={{
                                            fontSize: '0.7rem', textTransform: 'uppercase', fontWeight: 'bold',
                                            color: inv.status === 'rejected' ? '#ff4757' : '#ffa502'
                                        }}>
                                            {inv.status}
                                        </span>
                                    </div>
                                    <div style={{ fontSize: '0.85rem', color: '#666' }}>
                                        {inv.group_name}
                                    </div>
                                    <div style={{ fontSize: '0.75rem', color: '#aaa', marginTop: '5px' }}>{inv.sent_at}</div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </div>

            {/* COLONNA DESTRA: INVITI RICEVUTI */}
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
                <h3 style={{ marginTop: 0, color: '#333', borderBottom: '2px solid #f0f2f5', paddingBottom: '10px', marginBottom: '20px' }}>
                    Inviti Ricevuti
                    {receivedList.length > 0 && <span style={{ marginLeft: '10px', color: '#667eea' }}>({receivedList.length})</span>}
                </h3>

                {receivedList.length === 0 ? (
                    <div className="empty-state" style={{
                        background: '#fff', borderRadius: '12px', padding: '40px',
                        textAlign: 'center', color: '#888', border: '2px dashed #e0e0e0'
                    }}>
                        <div style={{ fontSize: '2rem', marginBottom: '10px' }}>📭</div>
                        Non hai nuovi inviti al momento.
                    </div>
                ) : (
                    <div className="received-list" style={{ overflowY: 'auto' }}>
                        {receivedList.map(inv => (
                            <div key={inv.id} style={{
                                background: 'white', padding: '20px', borderRadius: '12px', marginBottom: '15px',
                                boxShadow: '0 4px 15px rgba(0,0,0,0.05)', borderLeft: '5px solid #667eea'
                            }}>
                                <div style={{ marginBottom: '15px' }}>
                                    <div style={{ fontSize: '0.9rem', color: '#555', marginBottom: '5px' }}>
                                        <strong style={{ color: '#333', fontSize: '1rem' }}>{inv.partner_username}</strong> ti ha invitato a partecipare:
                                    </div>
                                    <div style={{ fontSize: '1.2rem', color: '#667eea', fontWeight: 'bold' }}>{inv.group_name}</div>
                                    <div style={{ fontSize: '0.75rem', color: '#999', marginTop: '8px' }}>Inviato il: {inv.sent_at}</div>
                                </div>
                                <div style={{ display: 'flex', gap: '10px' }}>
                                    <button
                                        onClick={() => respondToInvite(inv.id, false)}
                                        style={{ flex: 1, background: '#fff', color: '#ff4757', border: '1px solid #ff4757', padding: '10px', borderRadius: '6px', cursor: 'pointer', fontWeight: 'bold', transition: '0.2s' }}
                                    >
                                        Rifiuta
                                    </button>
                                    <button
                                        onClick={() => respondToInvite(inv.id, true)}
                                        style={{ flex: 1, background: '#667eea', color: 'white', border: 'none', padding: '10px', borderRadius: '6px', cursor: 'pointer', fontWeight: 'bold', boxShadow: '0 2px 5px rgba(102, 126, 234, 0.4)' }}
                                    >
                                        Accetta Invito
                                    </button>
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}

export default InvitesPage;