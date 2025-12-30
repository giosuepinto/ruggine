
// ============================================================================
// Layout.jsx - Layout principale dell'applicazione
// Contiene la navbar e il contenuto delle pagine.
// ============================================================================
// ================== Import base React ==================
import React from 'react';

// ================== Import router ==================
import { Outlet, useNavigate, useLocation } from 'react-router-dom';

// ================== Import context ==================
import { useWebSocket } from '../context/WebSocketContext';

// ================== Import stile globale ==================
import '../App.css';

// ================== Import componenti ==================
import Navbar from './Navbar';

// Componente layout principale
const Layout = () => {
    const { user, isConnected, logout, invitations } = useWebSocket();
    const navigate = useNavigate();
    const location = useLocation();
    if (location.pathname === '/') return <Outlet />;
    return (
        <div className="app-container">
            <Navbar
                user={user}
                isConnected={isConnected}
                logout={logout}
                invitations={invitations}
                navigate={navigate}
                location={location}
            />
            <div style={{ flex: 1, overflow: 'hidden' }}>
                <Outlet />
            </div>
        </div>
    );
};

export default Layout;