use tokio::net::UdpSocket;
use std::net::SocketAddr;
use crate::dns_message::DnsMessage;

pub struct DnsClient {
    socket: UdpSocket,
    server_addr: SocketAddr,
}

impl DnsClient {
    pub async fn new(server_addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        
        Ok(Self {
            socket,
            server_addr,
        })
    }

    pub async fn resolve(&self, domain: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Créer une requête DNS
        let query_id = rand::random::<u16>();
        let query = DnsMessage::new_query(query_id, domain.to_string());
        let query_bytes = query.to_bytes();

        // Envoyer la requête
        self.socket.send_to(&query_bytes, &self.server_addr).await?;
        println!("Requête envoyée pour: {}", domain);

        // Recevoir la réponse
        let mut buffer = [0u8; 512];
        let (size, _) = self.socket.recv_from(&mut buffer).await?;
        
        // Parser la réponse
        let response = DnsMessage::from_bytes(&buffer[..size])?;
        
        if response.header.id != query_id {
            return Err("ID de réponse invalide".into());
        }

        if response.answers.is_empty() {
            return Err("Aucune réponse trouvée".into());
        }

        // Extraire l'adresse IP de la première réponse
        let answer = &response.answers[0];
        if answer.rdata.len() == 4 {
            let ip = format!("{}.{}.{}.{}", 
                answer.rdata[0], answer.rdata[1], 
                answer.rdata[2], answer.rdata[3]);
            Ok(ip)
        } else {
            Err("Format de réponse invalide".into())
        }
    }
}

// Fonction utilitaire pour les tests
pub async fn test_client() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr = "127.0.0.1:5353".parse()?;
    let client = DnsClient::new(server_addr).await?;
    
    println!("=== Test du client DNS ===");
    
    let domains = vec!["example.com", "google.com", "github.com"];
    
    for domain in domains {
        match client.resolve(domain).await {
            Ok(ip) => println!("{} -> {}", domain, ip),
            Err(e) => println!("Erreur pour {}: {}", domain, e),
        }
    }
    
    Ok(())
}