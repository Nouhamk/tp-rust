use std::io::{self, Write};
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::mpsc;

mod protocol;
use protocol::{MessageType, ProtocolMessage, ProtocolError};

/// État du client
#[derive(Debug, Clone)]
pub struct ClientState {
    pub username: Option<String>,
    pub user_id: Option<String>,
    pub connected: bool,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            username: None,
            user_id: None,
            connected: false,
        }
    }
    
    pub fn is_authenticated(&self) -> bool {
        self.username.is_some() && self.user_id.is_some()
    }
}

/// Client de chat
pub struct ChatClient {
    state: ClientState,
}

impl ChatClient {
    pub fn new() -> Self {
        Self {
            state: ClientState::new(),
        }
    }
    
    /// Se connecte au serveur
    pub async fn connect(&mut self, addr: &str) -> Result<(), ProtocolError> {
        let stream = TcpStream::connect(addr).await?;
        println!("Connecté au serveur {}", addr);
        
        let (reader, writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut writer = BufWriter::new(writer);
        
        self.state.connected = true;
        
        // Channel pour envoyer des messages depuis l'interface utilisateur
        let (tx, mut rx) = mpsc::channel::<ProtocolMessage>(100);
        
        // Tâche pour lire les messages du serveur
        let read_task = tokio::spawn(async move {
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        println!("Connexion fermée par le serveur");
                        break;
                    }
                    Ok(_) => {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            match ProtocolMessage::from_json(trimmed) {
                                Ok(msg) => {
                                    handle_server_message(msg).await;
                                }
                                Err(e) => {
                                    eprintln!("Erreur parsing message serveur: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Erreur lecture serveur: {}", e);
                        break;
                    }
                }
            }
        });
        
        // Tâche pour envoyer des messages au serveur
        let write_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(json) = msg.to_json() {
                    if let Err(e) = writer.write_all(format!("{}\n", json).as_bytes()).await {
                        eprintln!("Erreur envoi message: {}", e);
                        break;
                    }
                    if let Err(e) = writer.flush().await {
                        eprintln!("Erreur flush: {}", e);
                        break;
                    }
                }
            }
        });
        
        // Interface utilisateur dans le thread principal
        self.run_user_interface(tx).await?;
        
        // Attendre que les tâches se terminent
        read_task.abort();
        write_task.abort();
        
        Ok(())
    }
    
    /// Lance l'interface utilisateur
    async fn run_user_interface(&mut self, tx: mpsc::Sender<ProtocolMessage>) -> Result<(), ProtocolError> {
        println!("\n=== Client de Chat ===");
        println!("Commandes disponibles:");
        println!("  /register <nom>  - S'enregistrer avec un nom d'utilisateur");
        println!("  /users           - Lister les utilisateurs connectés");
        println!("  /quit            - Quitter le chat");
        println!("  <message>        - Envoyer un message (après enregistrement)");
        println!("================================\n");
        
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }
            
            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            
            if input.starts_with('/') {
                // Commande
                let parts: Vec<&str> = input.splitn(2, ' ').collect();
                let command = parts[0];
                
                match command {
                    "/register" => {
                        if parts.len() < 2 {
                            println!("Usage: /register <nom>");
                            continue;
                        }
                        let username = parts[1].to_string();
                        let msg = ProtocolMessage::new(MessageType::Register { username });
                        if tx.send(msg).await.is_err() {
                            break;
                        }
                    }
                    "/users" => {
                        let msg = ProtocolMessage::new(MessageType::ListUsers);
                        if tx.send(msg).await.is_err() {
                            break;
                        }
                    }
                    "/quit" => {
                        let msg = ProtocolMessage::new(MessageType::Disconnect);
                        let _ = tx.send(msg).await;
                        break;
                    }
                    _ => {
                        println!("Commande inconnue: {}", command);
                    }
                }
            } else {
                // Message normal
                if !self.state.is_authenticated() {
                    println!("Vous devez vous enregistrer d'abord avec /register <nom>");
                    continue;
                }
                
                let msg = ProtocolMessage::new(MessageType::SendMessage {
                    content: input.to_string()
                });
                if tx.send(msg).await.is_err() {
                    break;
                }
            }
        }
        
        Ok(())
    }
}

/// Traite les messages reçus du serveur
async fn handle_server_message(msg: ProtocolMessage) {
    match msg.message_type {
        MessageType::RegisterSuccess { user_id } => {
            println!("✓ Enregistrement réussi! ID: {}", user_id);
        }
        
        MessageType::RegisterError { reason } => {
            println!("✗ Erreur d'enregistrement: {}", reason);
        }
        
        MessageType::MessageReceived { from, content, timestamp } => {
            println!("[{}] {}: {}", 
                timestamp.format("%H:%M:%S"), 
                from, 
                content
            );
        }
        
        MessageType::UserList { users } => {
            println!("Utilisateurs connectés ({}):", users.len());
            for user in users {
                println!("  - {}", user);
            }
        }
        
        MessageType::UserJoined { username } => {
            println!("→ {} a rejoint le chat", username);
        }
        
        MessageType::UserLeft { username } => {
            println!("← {} a quitté le chat", username);
        }
        
        MessageType::Error { message } => {
            println!("✗ Erreur: {}", message);
        }
        
        MessageType::Pong => {
            println!("Pong reçu du serveur");
        }
        
        _ => {
            println!("Message non géré: {:?}", msg.message_type);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ChatClient::new();
    
    // Adresse du serveur
    let server_addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    
    println!("Tentative de connexion à {}...", server_addr);
    
    if let Err(e) = client.connect(&server_addr).await {
        eprintln!("Erreur de connexion: {}", e);
        std::process::exit(1);
    }
    
    println!("Au revoir!");
    Ok(())
}