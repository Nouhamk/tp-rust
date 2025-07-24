use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use chrono::Utc;
use std::fs;

// Structure pour gérer le fichier de log partagé
#[derive(Clone)]
struct LogServer {
    log_file: Arc<Mutex<tokio::fs::File>>,
}

impl LogServer {
    async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Créer le dossier logs s'il n'existe pas
        fs::create_dir_all("logs")?;
        
        // Ouvrir le fichier en mode append
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/server.log")
            .await?;
            
        Ok(LogServer {
            log_file: Arc::new(Mutex::new(file)),
        })
    }
    
    async fn write_log(&self, message: &str, client_id: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
        let log_entry = format!("[{timestamp}] Client {client_id}: {message}\n");
        
        // Verrouiller le fichier pour écriture thread-safe
        let mut file = self.log_file.lock().await;
        file.write_all(log_entry.as_bytes()).await?;
        file.flush().await?;
        
        println!("📝 [{}] Client {}: {}", timestamp, client_id, message.trim());
        Ok(())
    }
}

async fn handle_client(
    stream: TcpStream, 
    log_server: LogServer, 
    client_id: usize
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let peer_addr = stream.peer_addr()?;
    println!("🔗 Client {} connecté depuis {}", client_id, peer_addr);
    
    let (reader, mut writer) = stream.into_split();
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();
    
    // Envoyer un message de bienvenue
    writer.write_all(format!("Bienvenue! Vous êtes le client {}. Tapez vos messages:\n", client_id).as_bytes()).await?;
    
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        
        // Commande spéciale pour déconnecter
        if line.trim().eq_ignore_ascii_case("quit") || line.trim().eq_ignore_ascii_case("exit") {
            writer.write_all(b"Au revoir!\n").await?;
            break;
        }
        
        // Écrire dans le fichier log
        if let Err(e) = log_server.write_log(&line, client_id).await {
            eprintln!("❌ Erreur lors de l'écriture du log: {}", e);
            writer.write_all(b"Erreur serveur lors de l'enregistrement\n").await?;
        } else {
            writer.write_all(b"Message enregistre avec succes\n").await?;
        }
    }
    
    println!("❌ Client {} déconnecté", client_id);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Démarrage du serveur de journalisation...");
    
    // Initialiser le serveur de log
    let log_server = LogServer::new().await?;
    println!("📁 Fichier de log initialisé: logs/server.log");
    
    // Écouter sur le port 8080
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("👂 Serveur en écoute sur 127.0.0.1:8080");
    println!("💡 Les clients peuvent se connecter avec: telnet 127.0.0.1 8080");
    println!("💡 Ou avec: nc 127.0.0.1 8080");
    println!("📋 Tapez 'quit' ou 'exit' pour vous déconnecter");
    println!("{}", "=".repeat(50));
    
    let mut client_counter = 0;
    let mut tasks = Vec::new();
    
    loop {
        // Accepter une nouvelle connexion
        match listener.accept().await {
            Ok((stream, _)) => {
                client_counter += 1;
                let client_id = client_counter;
                let log_server_clone = log_server.clone();
                
                // Spawner une tâche pour chaque client
                let task = tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, log_server_clone, client_id).await {
                        eprintln!("❌ Erreur avec client {}: {}", client_id, e);
                    }
                });
                
                tasks.push(task);
                
                // Nettoyer les tâches terminées pour éviter l'accumulation
                tasks.retain(|task| !task.is_finished());
            }
            Err(e) => {
                eprintln!("❌ Erreur lors de l'acceptation de connexion: {}", e);
            }
        }
    }
}