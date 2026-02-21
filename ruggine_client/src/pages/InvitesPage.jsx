
// ============================================================================
// InvitesPage.jsx - Gestione inviti inviati e ricevuti
// Mostra colonne inviti e filtra inviti attivi.
// ============================================================================

// ================== Import base React ==================
import React, { useState, useEffect } from 'react';

// ================== Import context ==================
import { useWebSocket } from '../context/WebSocketContext';

// ================== Import stile globale ==================
import '../App.css';

// ================== Import componenti ==================
import SentInvitesColumn from '../components/SentInvitesColumn';
import ReceivedInvitesColumn from '../components/ReceivedInvitesColumn';

// Componente pagina inviti
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
            <SentInvitesColumn sentList={sentList} />
            <ReceivedInvitesColumn receivedList={receivedList} respondToInvite={respondToInvite} />
        </div>
    );
}

export default InvitesPage;