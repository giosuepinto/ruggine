// src/main.jsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import { WebSocketProvider } from './context/WebSocketContext';
import Layout from './components/Layout';
import LoginPage from './pages/LoginPage';
import ChatPage from './pages/ChatPage';
import InvitesPage from './pages/InvitesPage'; // Assicurati di importarlo

// Definiamo le rotte
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