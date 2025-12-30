
// ================== Import base React ==================
import React from 'react';
import ReactDOM from 'react-dom/client';

// ================== Import router ==================
import { createBrowserRouter, RouterProvider } from 'react-router-dom';

// ================== Import context ==================
import { WebSocketProvider } from './context/WebSocketContext';

// ================== Import componenti ==================
import Layout from './components/Layout';

// ================== Import pagine ==================
import LoginPage from './pages/LoginPage';
import ChatPage from './pages/ChatPage';
import InvitesPage from './pages/InvitesPage';

// Definizione delle rotte
const router = createBrowserRouter([
    {
        element: <WebSocketProvider><Layout /></WebSocketProvider>, // Wrap globale
        children: [
            {
                path: '/',
                element: <LoginPage />,
            },
            {
                path: '/chat',
                element: <ChatPage />,
            },
            {
                path: '/invites',
                element: <InvitesPage />,
            },
        ]
    }
]);

ReactDOM.createRoot(document.getElementById('root')).render(
    <React.StrictMode>
        <RouterProvider router={router} />
    </React.StrictMode>
);