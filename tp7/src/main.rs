mod dns_message;
mod client;
mod server;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage:");
        println!("  {} server          - Démarrer le serveur DNS", args[0]);
        println!("  {} client <domain> - Résoudre un domaine", args[0]);
        println!("  {} test            - Tester client et serveur", args[0]);
        return Ok(());
    }

    match args[1].as_str() {
        "server" => {
            server::test_server().await?;
        },
        "client" => {
            if args.len() < 3 {
                println!("Usage: {} client <domain>", args[0]);
                return Ok(());
            }
            
            let domain = &args[2];
            let server_addr = "127.0.0.1:5353".parse()?;
            let client = client::DnsClient::new(server_addr).await?;
            
            match client.resolve(domain).await {
                Ok(ip) => println!("{} -> {}", domain, ip),
                Err(e) => println!("Erreur: {}", e),
            }
        },
        "test" => {
            println!("Mode test - démarrage du serveur en arrière-plan...");
            
            // Démarrer le serveur en arrière-plan
            tokio::spawn(async {
                if let Err(e) = server::test_server().await {
                    eprintln!("Erreur serveur: {}", e);
                }
            });
            
            // Attendre que le serveur démarre
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Tester le client
            client::test_client().await?;
        },
        _ => {
            println!("Commande inconnue: {}", args[1]);
        }
    }
    
    Ok(())
}