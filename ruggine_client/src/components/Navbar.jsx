// ============================================================================
// Navbar.jsx - Barra di navigazione principale
// Mostra link di navigazione e info utente.
// ============================================================================

// ================== Import componenti ==================
// (Nessun import esplicito, componente dichiarato inline)

// Componente barra di navigazione
function Navbar({ user, isConnected, logout, invitations, navigate, location }) {
    const receivedCount = invitations?.received?.length || 0;
    return (
        <div className="navbar">
            <div className="nav-brand">
                <span className={`status-dot ${isConnected ? 'status-online' : 'status-offline'}`}></span>
                Ruggine Chat
            </div>
            <div style={{ display: 'flex', gap: '30px' }}>
                <button
                    onClick={() => navigate('/chat')}
                    className="nav-link"
                    style={{
                        background: 'none', border: 'none', color: 'white', cursor: 'pointer',
                        fontWeight: location.pathname === '/chat' ? 'bold' : 'normal',
                        fontSize: '1rem', opacity: location.pathname === '/chat' ? 1 : 0.8
                    }}
                >
                    Le Mie Chat
                </button>
                <button
                    onClick={() => navigate('/invites')}
                    className="nav-link"
                    style={{
                        background: 'none', border: 'none', color: 'white', cursor: 'pointer',
                        fontWeight: location.pathname === '/invites' ? 'bold' : 'normal',
                        fontSize: '1rem', position: 'relative', opacity: location.pathname === '/invites' ? 1 : 0.8
                    }}
                >
                    Inviti
                    {receivedCount > 0 && (
                        <span style={{
                            position: 'absolute', top: '-8px', right: '-12px',
                            background: '#ff4757', color: 'white', borderRadius: '50%',
                            minWidth: '18px', height: '18px', padding: '0 4px', fontSize: '0.7rem',
                            display: 'flex', justifyContent: 'center', alignItems: 'center', fontWeight: 'bold'
                        }}>
                            {receivedCount}
                        </span>
                    )}
                </button>
            </div>
            <div className="nav-user-info">
                <span>Ciao, <strong>{user}</strong></span>
                <button onClick={logout} className="logout-btn">Logout</button>
            </div>
        </div>
    );
}

export default Navbar;
