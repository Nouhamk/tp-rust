use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use uuid::Uuid;

mod protocol;
use protocol::{MessageType, ProtocolMessage, ProtocolError, SessionState};

/// Structure représentant un utilisateur connecté
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub session_state: SessionState,
}

/// État partagé du serveur
#[derive(Debug)]
pub struct ServerState {
    /// Utilisateurs connectés (user_id -> User)
    pub users: RwLock<HashMap<String, User>>,
    /// Noms d'utilisateur pris (username -> user_id)
    pub usernames: RwLock<HashMap<String, String>>,
    /// Channel de diffusion pour les messages
    pub broadcast: broadcast::Sender<ProtocolMessage>,
}

impl ServerState {
    pub fn new() -> (Arc<Self>, broadcast::Receiver<ProtocolMessage>) {
        let (tx, rx) = broadcast::channel(1000);
        let state = Arc::new(Self {
            users: RwLock::new(HashMap::new()),
            usernames: RwLock::new(HashMap::new()),
            broadcast: tx,
        });
        (state, rx)
    }
    
    /// Enregistre un nouvel utilisateur
    pub async fn register_user(&self, username: String) -> Result<String, ProtocolError> {
        let mut usernames = self.usernames.write().await;
        let mut users = self.users.write().await;
        
        if usernames.contains_key(&username) {
            return Err(ProtocolError::UsernameExists(username));
        }
        
        let user_id = Uuid::new_v4().to_string();
        let user = User {
            id: user_id.clone(),
            username: username.clone(),
            session_state: SessionState::Authenticated(username.clone()),
        };
        
        usernames.insert(username.clone(), user_id.clone());
        users.insert(user_id.clone(), user);
        
        // Notifier les autres utilisateurs
        let join_msg = ProtocolMessage::new(MessageType::UserJoined { username });
        let _ = self.broadcast.send(join_msg);
        
        Ok(user_id)
    }
    
    /// Supprime un utilisateur
    pub async fn remove_user(&self, user_id: &str) {
        let mut users = self.users.write().await;
        let mut usernames = self.usernames.write().await;
        
        if let Some(user) = users.remove(user_id) {
            usernames.remove(&user.username);
            
            // Notifier les autres utilisateurs
            let leave_msg = ProtocolMessage::new(MessageType::UserLeft { 
                username: user.username 
            });
            let _ = self.broadcast.send(leave_msg);
        }
    }
    
    /// Obtient la liste des utilisateurs connectés
    pub async fn get_user_list(&self) -> Vec<String> {
        let users = self.users.read().await;
        users.values().map(|user| user.username.clone()).collect()
    }
    
    /// Diffuse un message à tous les utilisateurs connectés
    pub async fn broadcast_message(&self, from: String, content: String) {
        let message = ProtocolMessage::new(MessageType::MessageReceived {
            from,
            content,
            timestamp: chrono::Utc::now(),
        });
        
        let _ = self.broadcast.send(message);
    }
}

/// Gère une connexion client
pub async fn handle_client(
    stream: TcpStream,
    state: Arc<ServerState>,
) -> Result<(), ProtocolError> {
    let peer_addr = stream.peer_addr()?;
    println!("Nouvelle connexion de: {}", peer_addr);
    
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let writer = Arc::new(tokio::sync::Mutex::new(BufWriter::new(writer)));
    let mut broadcast_rx = state.broadcast.subscribe();
    
    let mut user_id: Option<String> = None;
    let mut session_state = SessionState::Connected;
    
    // Tâche pour recevoir les messages de diffusion
    let writer_clone = writer.clone();
    let broadcast_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if let Ok(json) = msg.to_json() {
                let mut writer_guard = writer_clone.lock().await;
                if let Err(e) = writer_guard.write_all(format!("{}\n", json).as_bytes()).await {
                    eprintln!("Erreur envoi broadcast: {}", e);
                    break;
                }
                if let Err(e) = writer_guard.flush().await {
                    eprintln!("Erreur flush broadcast: {}", e);
                    break;
                }
            }
        }
    });
    
    // Boucle principale de traitement des messages
    let mut line = String::new();
    loop {
        line.clear();
        
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // Connexion fermée
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                
                match ProtocolMessage::from_json(trimmed) {
                    Ok(msg) => {
                        let response = handle_message(
                            msg,
                            &mut session_state,
                            &mut user_id,
                            &state,
                        ).await;
                        
                        if let Some(response_msg) = response {
                            if let Ok(json) = response_msg.to_json() {
                                let mut writer_guard = writer.lock().await;
                                if let Err(e) = writer_guard.write_all(format!("{}\n", json).as_bytes()).await {
                                    eprintln!("Erreur envoi réponse: {}", e);
                                    break;
                                }
                                if let Err(e) = writer_guard.flush().await {
                                    eprintln!("Erreur flush réponse: {}", e);
                                    break;
                                }
                            }
                        }
                        
                        // Vérifier si le client s'est déconnecté
                        if session_state == SessionState::Disconnected {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Erreur parsing message: {}", e);
                        let error_msg = ProtocolMessage::error(format!("Message invalide: {}", e));
                        if let Ok(json) = error_msg.to_json() {
                            let mut writer_guard = writer.lock().await;
                            let _ = writer_guard.write_all(format!("{}\n", json).as_bytes()).await;
                            let _ = writer_guard.flush().await;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Erreur lecture: {}", e);
                break;
            }
        }
    }
    
    // Nettoyage à la déconnexion
    if let Some(id) = user_id {
        state.remove_user(&id).await;
    }
    
    broadcast_task.abort();
    println!("Client {} déconnecté", peer_addr);
    Ok(())
}

/// Traite un message reçu du client
async fn handle_message(
    msg: ProtocolMessage,
    session_state: &mut SessionState,
    user_id: &mut Option<String>,
    state: &Arc<ServerState>,
) -> Option<ProtocolMessage> {
    match msg.message_type {
        MessageType::Register { username } => {
            if *session_state != SessionState::Connected {
                return Some(ProtocolMessage::error("Déjà authentifié".to_string()));
            }
            
            match state.register_user(username.clone()).await {
                Ok(id) => {
                    *user_id = Some(id.clone());
                    *session_state = SessionState::Authenticated(username);
                    Some(ProtocolMessage::new(MessageType::RegisterSuccess { user_id: id }))
                }
                Err(ProtocolError::UsernameExists(_)) => {
                    Some(ProtocolMessage::new(MessageType::RegisterError {
                        reason: "Nom d'utilisateur déjà pris".to_string()
                    }))
                }
                Err(e) => {
                    Some(ProtocolMessage::error(e.to_string()))
                }
            }
        }
        
        MessageType::SendMessage { content } => {
            if let SessionState::Authenticated(username) = session_state {
                state.broadcast_message(username.clone(), content).await;
                None // Pas de réponse directe, le message sera diffusé
            } else {
                Some(ProtocolMessage::error("Non authentifié".to_string()))
            }
        }
        
        MessageType::ListUsers => {
            let users = state.get_user_list().await;
            Some(ProtocolMessage::new(MessageType::UserList { users }))
        }
        
        MessageType::Disconnect => {
            *session_state = SessionState::Disconnected;
            None
        }
        
        MessageType::Ping => {
            Some(ProtocolMessage::pong())
        }
        
        _ => {
            Some(ProtocolMessage::error("Type de message non supporté".to_string()))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Serveur de chat démarré sur 127.0.0.1:8080");
    
    let (state, _) = ServerState::new();
    
    loop {
        let (stream, _) = listener.accept().await?;
        let state_clone = state.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, state_clone).await {
                eprintln!("Erreur client: {}", e);
            }
        });
    }
}