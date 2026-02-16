// ============================================================================
// ReceivedInvitesColumn.jsx - Colonna inviti ricevuti
// Mostra inviti ricevuti e azioni accetta/rifiuta.
// ============================================================================

// ================== Import componenti ==================
// (Nessun import necessario)

function ReceivedInvitesColumn({ receivedList, respondToInvite }) {
    return (
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
            <h3 style={{ marginTop: 0, color: '#555', borderBottom: '2px solid #f0f2f5', paddingBottom: '10px' }}>
                Inviti Ricevuti
                {receivedList.length > 0 && <span style={{ marginLeft: '10px', background: '#eee', padding: '2px 8px', borderRadius: '10px', fontSize: '0.8rem', color: '#333' }}>{receivedList.length}</span>}
            </h3>
            <div style={{ flex: 1, overflowY: 'auto' }}>
                {receivedList.length === 0 ? (
                    <div style={{ color: '#999', fontSize: '0.9rem', fontStyle: 'italic', padding: '20px', background: '#fff', borderRadius: '8px', textAlign: 'center' }}>
                        Nessun invito ricevuto.
                    </div>
                ) : (
                    <div className="received-list">
                        {receivedList.map(inv => (
                            <div key={inv.id} className="invite-card received">
                                <div><strong>Da:</strong> {inv.partner_username}</div>
                                <div><strong>Gruppo:</strong> {inv.group_name}</div>
                                <div><strong>Stato:</strong> {inv.status === 'pending' ? 'In attesa' : inv.status === 'accepted' ? 'Accettato' : 'Rifiutato'}</div>
                                <div style={{ fontSize: '0.8rem', color: '#888' }}>{inv.sent_at}</div>
                                {inv.status === 'pending' && (
                                    <div style={{ marginTop: '10px', display: 'flex', gap: '10px' }}>
                                        <button className="mini-btn accept" onClick={() => respondToInvite(inv.id, true)}>Accetta</button>
                                        <button className="mini-btn reject" onClick={() => respondToInvite(inv.id, false)}>Rifiuta</button>
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}

export default ReceivedInvitesColumn;
