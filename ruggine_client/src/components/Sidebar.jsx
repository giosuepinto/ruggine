// Sidebar.jsx

// ================== Import funzioni da pagine ==================
import { formatGroupName, getAvatarInitial } from "../pages/ChatPage";

// Componente Sidebar
function Sidebar({ groups, user, activeGroupId, setActiveGroupId, getLastMessagePreview, setIsModalOpen }) {
    return (
        <div className="sidebar">
            <div className="sidebar-header">
                <h3 style={{ margin: 0, color: '#333' }}>Le tue Chat</h3>
                <button className="add-chat-btn" onClick={() => setIsModalOpen(true)} title="Nuova Chat">+</button>
            </div>
            <div className="chat-list">
                {groups.map(group => {
                    const name = formatGroupName(group, user);
                    if (group.is_dm && group.members.length < 2) return null;
                    return (
                        <div
                            key={group.group_id}
                            className={`chat-item ${activeGroupId === group.group_id ? 'active' : ''}`}
                            onClick={() => setActiveGroupId(group.group_id)}
                        >
                            <div className="avatar" style={group.group_id === 1 ? {background: 'none', color: 'inherit', padding: 0, position: 'relative', overflow: 'visible'} : {}}>
                                {group.group_id === 1 ? (
                                    <img src="/ruggine_logo.png" alt="Ruggine Logo" style={{ width: 60, height: 60, objectFit: 'contain', borderRadius: '50%', display: 'block', margin: '0 auto' }} />
                                ) : (
                                    getAvatarInitial(name)
                                )}
                            </div>
                            <div className="chat-info">
                                <div className="chat-name">{name}</div>
                                <div className="chat-preview">{getLastMessagePreview(group.group_id)}</div>
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

export default Sidebar;
