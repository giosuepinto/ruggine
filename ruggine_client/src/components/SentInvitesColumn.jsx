// ============================================================================
// SentInvitesColumn.jsx - Colonna inviti inviati
// Mostra richieste inviate dall'utente.
// ============================================================================

// ================== Import componenti ==================
// (Nessun import necessario)

function SentInvitesColumn({ sentList }) {
    return (
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
                            <div key={inv.id} className="invite-card sent">
                                <div><strong>A:</strong> {inv.partner_username}</div>
                                <div><strong>Gruppo:</strong> {inv.group_name}</div>
                                <div><strong>Stato:</strong> {inv.status === 'pending' ? 'In attesa' : inv.status === 'rejected' ? 'Rifiutato' : inv.status}</div>
                                <div style={{ fontSize: '0.8rem', color: '#888' }}>{inv.sent_at}</div>
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}

export default SentInvitesColumn;
