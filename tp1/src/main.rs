use std::io;

fn main() {
    println!("Gestion de Compte Bancaire");
    
    // Variables pour le compte
    let nom_titulaire = "Nouhaila Moukadd";
    let mut solde:f32 = 1000.0;
    
    // Menu principal avec boucle
    loop {
        // Affichage du menu
        afficher_menu();
        
        // Lire le choix
        println!("Veuillez saisir un numéro de votre choix:");
        let mut choix = String::new();
        io::stdin().read_line(&mut choix).expect("Erreur de lecture");
        
        // Convertir le choix en nombre
        let choix:usize = match choix.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Veuillez saisir un numéro valide");
                continue; 
            }
        };
        
        // Traiter le choix
        if choix == 1 {
            // Afficher solde
            afficher_solde(nom_titulaire, solde);
        } else if choix == 2 {
            // Retrait
            solde = effectuer_retrait(solde);
        } else if choix == 3 {
            // Liste des comptes
            lister_comptes(nom_titulaire, solde);
        } else if choix == 4 {
            // Quitter
            println!("Au revoir !");
            break;
        } else {
            println!("Choix hors système ! Limite système (1-4)");
        }
    }
}

// Fonction pour afficher le menu
fn afficher_menu() {
    let options = ["Afficher solde", "Retrait", "Liste comptes", "Quitter"];
    println!("\nMENU");
    for (i, option) in options.iter().enumerate() {
        println!("{}. {}", i+1, option);
    }
}

// Fonction pour afficher le solde
fn afficher_solde(nom: &str, solde: f32) {
    println!("Compte de {} : {:.2}€", nom, solde);
}

// Fonction pour effectuer un retrait
fn effectuer_retrait(solde_actuel: f32) -> f32 {
    println!("Entrez le montant à retirer:");
    let mut montant_input = String::new();
    io::stdin().read_line(&mut montant_input).expect("Erreur de lecture");
    
    let montant:f32 = match montant_input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Montant invalide !");
            return solde_actuel;
        }
    };
    
    if montant <= 0.0 {
        println!("Le montant doit être positif !");
        solde_actuel
    } else if montant > solde_actuel {
        println!("Solde insuffisant ! Solde actuel: {:.2}€", solde_actuel);
        solde_actuel
    } else {
        let nouveau_solde = solde_actuel - montant;
        println!("Retrait de {:.2}€ effectué. Nouveau solde: {:.2}€", montant, nouveau_solde);
        nouveau_solde
    }
}

// Fonction pour lister les comptes
fn lister_comptes(nom: &str, solde: f32) {
    println!("\nLISTE DES COMPTES");
    println!("1. Compte de {} - Solde: {:.2}€", nom, solde);
}