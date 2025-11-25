// src/pages/InvitesPage.jsx
import React, { useState, useEffect } from 'react';
import { useWebSocket } from '../context/WebSocketContext';
import '../App.css';

function InvitesPage() {
    const { invitations, respondToInvite } = useWebSocket();
    // Stato locale per forzare il refresh del timer ogni 10 secondi
    // così gli inviti scaduti spariscono dalla UI dinamicamente
    const [, setTick] = useState(0);

    useEffect(() => {
        const timer = setInterval(() => setTick(t => t + 1), 5000); // Check ogni 5s
        return () => clearInterval(timer);
    }, []);

    // Helper per filtrare inviti "scaduti" (Rifiutati da > 2 min)
    const filterActiveInvites = (invites) => {
        if (!invites) return [];
        const now = new Date();

        return invites.filter(inv => {
            // Se è pending, mostralo sempre
            if (inv.status === 'pending') return true;

            // Se è rejected, controlla il tempo
            // Nota: Assumiamo che sent_at sia UTC o gestito localmente.
            // SQLite da stringhe tipo "2023-11-24 10:00:00".
            // Aggiungiamo "Z" per trattarlo come UTC se il server usa UTC,
            // oppure lasciamo così se il browser capisce.
            // Per sicurezza parsiamo la data.
            const inviteTime = new Date(inv.sent_at.replace(" ", "T") + "Z"); // Hack per formato SQLite UTC
            const diffMinutes = (now - inviteTime) / 1000 / 60;

            // Mostra solo se sono passati meno di 2 minuti
            return diffMinutes < 2;
        });
    };

    const receivedList = invitations.received || [];
    // Applichiamo il filtro solo alla lista inviati (dove vediamo i rifiutati)
    const sentList = filterActiveInvites(invitations.sent);

    return (
        <div className="invites-container" style={{ padding: '2rem', maxWidth: '1000px', margin: '0 auto', display: 'flex', gap: '40px', height: 'calc(100vh - 80px)' }}>

            {/* COLONNA SINISTRA: INVIATI (SENT) */}
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
                <h3 style={{ marginTop: 0, color: '#555', borderBottom: '2px solid #f0f2f5', paddingBottom: '10px' }}>
                    Richieste Inviate
                    {sentList.length > 0 && <span style={{ marginLeft: '10px', background: '#eee', padding: '2px 8px', borderRadius: '10px', fontSize: '0.8rem', color: '#333' }}>{sentList.length}</span>}
                </h3>

                <div style={{ flex: 1, overflowY: 'auto' }}>
                    {sentList.length === 0 ? (
                        <div style={{ color: '#999', fontSize: '0.9rem', fontStyle: 'italic', padding: '20px', background: '#fff', borderRadius: '8px', textAlign: 'center' }}>
                            Nessuna richiesta in attesa.
                        </div>
                    ) : (
                        <div className="sent-list">
                            {sentList.map(inv => (
                                <div key={inv.id} style={{
                                    background: '#fff', padding: '15px', borderRadius: '8px', marginBottom: '10px',
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
                                        Gruppo: <strong>{inv.group_name}</strong>
                                    </div>
                                    <div style={{ fontSize: '0.75rem', color: '#aaa', marginTop: '5px' }}>
                                        {inv.status === 'rejected' ? 'Rifiutato recentemente' : `Inviato: ${inv.sent_at}`}
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </div>

            {/* COLONNA DESTRA: RICEVUTI (RECEIVED) */}
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
                <h3 style={{ marginTop: 0, color: '#333', borderBottom: '2px solid #f0f2f5', paddingBottom: '10px', marginBottom: '20px' }}>
                    Inviti Ricevuti
                    {receivedList.length > 0 && <span style={{ marginLeft: '10px', color: '#667eea', fontWeight: 'bold' }}>({receivedList.length})</span>}
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
                                    <div style={{ fontSize: '0.75rem', color: '#999', marginTop: '8px' }}>Ricevuto: {inv.sent_at}</div>
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