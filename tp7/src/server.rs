use tokio::net::UdpSocket;
use std::collections::HashMap;
use std::net::SocketAddr;
use crate::dns_message::{DnsMessage, DnsAnswer};

pub struct DnsServer {
    socket: UdpSocket,
    records: HashMap<String, [u8; 4]>,
}

impl DnsServer {
    pub async fn new(bind_addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(bind_addr).await?;
        
        // Initialiser quelques enregistrements DNS de test
        let mut records = HashMap::new();
        records.insert("example.com".to_string(), [93, 184, 216, 34]);
        records.insert("google.com".to_string(), [142, 250, 191, 14]);
        records.insert("github.com".to_string(), [140, 82, 114, 4]);
        records.insert("localhost".to_string(), [127, 0, 0, 1]);
        
        println!("Serveur DNS démarré sur {}", bind_addr);
        println!("Enregistrements disponibles:");
        for (domain, ip) in &records {
            println!("  {} -> {}.{}.{}.{}", domain, ip[0], ip[1], ip[2], ip[3]);
        }
        
        Ok(Self {
            socket,
            records,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 512];
        
        loop {
            // Recevoir une requête
            let (size, client_addr) = self.socket.recv_from(&mut buffer).await?;
            
            // Traiter la requête en arrière-plan
            if let Err(e) = self.handle_query(&buffer[..size], client_addr).await {
                eprintln!("Erreur lors du traitement de la requête: {}", e);
            }
        }
    }

    async fn handle_query(&self, data: &[u8], client_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        // Parser la requête DNS
        let query = DnsMessage::from_bytes(data)?;
        
        println!("Requête reçue de {} (ID: {})", client_addr, query.header.id);
        
        if query.questions.is_empty() {
            return Err("Aucune question dans la requête".into());
        }
        
        let question = &query.questions[0];
        println!("  Question: {} (type: {})", question.name, question.qtype);
        
        // Chercher l'enregistrement
        let mut answers = Vec::new();
        
        if question.qtype == 1 { // A record
            if let Some(ip) = self.records.get(&question.name) {
                let answer = DnsAnswer::new(question.name.clone(), *ip);
                answers.push(answer);
                println!("  Réponse: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
            } else {
                println!("  Domaine non trouvé: {}", question.name);
            }
        }

        // Créer la réponse
        let response = DnsMessage::new_response(&query, answers);
        let response_bytes = response.to_bytes();

        // Envoyer la réponse
        self.socket.send_to(&response_bytes, client_addr).await?;
        println!("  Réponse envoyée à {}", client_addr);
        
        Ok(())
    }
}

pub async fn test_server() -> Result<(), Box<dyn std::error::Error>> {
    let bind_addr = "127.0.0.1:5353".parse()?;
    let server = DnsServer::new(bind_addr).await?;
    
    println!("=== Serveur DNS en écoute ===");
    println!("Utilisez Ctrl+C pour arrêter");
    
    server.run().await
}