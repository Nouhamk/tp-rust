use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Codes d'opération pour le protocole de chat
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum MessageType {
    // Messages du client vers le serveur
    Register { username: String },
    SendMessage { content: String },
    ListUsers,
    Disconnect,
    
    // Messages du serveur vers le client
    RegisterSuccess { user_id: String },
    RegisterError { reason: String },
    MessageReceived { 
        from: String, 
        content: String, 
        timestamp: DateTime<Utc> 
    },
    UserList { users: Vec<String> },
    UserJoined { username: String },
    UserLeft { username: String },
    Error { message: String },
    
    // Messages bidirectionnels
    Ping,
    Pong,
}

/// Structure principale du protocole
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    /// Identifiant unique du message
    pub id: String,
    /// Type et contenu du message
    pub message_type: MessageType,
    /// Timestamp de création
    pub timestamp: DateTime<Utc>,
}

impl ProtocolMessage {
    /// Crée un nouveau message de protocole
    pub fn new(message_type: MessageType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type,
            timestamp: Utc::now(),
        }
    }
    
    /// Sérialise le message en JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    /// Désérialise un message depuis JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
    
    /// Crée un message d'erreur
    #[allow(dead_code)]
    pub fn error(message: String) -> Self {
        Self::new(MessageType::Error { message })
    }
    
    /// Crée un message de ping
    #[allow(dead_code)]
    pub fn ping() -> Self {
        Self::new(MessageType::Ping)
    }
    
    /// Crée un message de pong
    #[allow(dead_code)]
    pub fn pong() -> Self {
        Self::new(MessageType::Pong)
    }
}

/// États possibles d'une session client
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum SessionState {
    /// Client connecté mais non authentifié
    Connected,
    /// Client authentifié avec un nom d'utilisateur
    Authenticated(String),
    /// Client déconnecté
    Disconnected,
}

/// Gestion des erreurs du protocole
#[derive(Debug)]
pub enum ProtocolError {
    SerializationError(serde_json::Error),
    NetworkError(std::io::Error),
    UsernameExists(String),
    NotAuthenticated,
    InvalidMessage(String),
    SessionClosed,
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::SerializationError(e) => write!(f, "Erreur de sérialisation: {}", e),
            ProtocolError::NetworkError(e) => write!(f, "Erreur réseau: {}", e),
            ProtocolError::UsernameExists(username) => write!(f, "Nom d'utilisateur déjà pris: {}", username),
            ProtocolError::NotAuthenticated => write!(f, "Utilisateur non authentifié"),
            ProtocolError::InvalidMessage(msg) => write!(f, "Message invalide: {}", msg),
            ProtocolError::SessionClosed => write!(f, "Session fermée"),
        }
    }
}

impl std::error::Error for ProtocolError {}

impl From<serde_json::Error> for ProtocolError {
    fn from(err: serde_json::Error) -> Self {
        ProtocolError::SerializationError(err)
    }
}

impl From<std::io::Error> for ProtocolError {
    fn from(err: std::io::Error) -> Self {
        ProtocolError::NetworkError(err)
    }
}