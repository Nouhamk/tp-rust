struct CompteBancaire {
    nom: String,
    solde: f64, // Changé en f64 pour être cohérent
}

impl CompteBancaire {
    // Constructeur pour créer un nouveau compte
    fn nouveau(nom: String, solde: f64) -> CompteBancaire {
        CompteBancaire { nom, solde }
    }

    fn afficher(&self) {
        println!("Compte de {} : {} €", self.nom, self.solde);
    }

    fn deposer(&mut self, montant: f64) {
        // Empêcher le dépôt d'un montant négatif
        if montant <= 0.0 {
            println!("Erreur : Impossible de déposer un montant négatif ou nul !");
            return;
        }
        
        self.solde += montant;
        println!("+{} € déposés", montant);
    }

    fn retirer(&mut self, montant: f64) {
        if self.solde >= montant {
            self.solde -= montant;
            println!("-{} € retirés", montant);
        } else {
            println!("Solde insuffisant");
        }
    }

    // Nouvelle méthode : renommer qui renvoie un nouveau compte avec le nom changé
    fn renommer(&self, nouveau_nom: String) -> CompteBancaire {
        CompteBancaire {
            nom: nouveau_nom,
            solde: self.solde, // On garde le même solde
        }
    }

    fn fermer(self) {
        println!("Le compte de {} est fermé, dernier solde : {} €", self.nom, self.solde);
    }
}

fn main() {
    // Création de plusieurs comptes dans un Vec
    let mut comptes = vec![
        CompteBancaire::nouveau(String::from("Nouredine"), 3000.0),
        CompteBancaire::nouveau(String::from("Nouhaila"), 1500.0),
        CompteBancaire::nouveau(String::from("Lina"), 2200.0),
    ];

    println!("État initial des comptes");
    // Utilisation d'iter() et enumerate() pour afficher tous les comptes
    for (index, compte) in comptes.iter().enumerate() {
        print!("Compte #{}: ", index + 1);
        compte.afficher();
    }

    println!("\nTest des opérations");
    
    // Test sur le premier compte (index 0)
    comptes[0].deposer(130.0);
    comptes[0].retirer(20.0);
    
    // Test du dépôt négatif (sera refusé)
    comptes[0].deposer(-50.0);
    
    println!("\nÉtat après opérations");
    for (index, compte) in comptes.iter().enumerate() {
        print!("Compte #{}: ", index + 1);
        compte.afficher();
    }

    println!("\nTest de la méthode renommer");
    // Créer un nouveau compte avec un nom différent
    let compte_renomme = comptes[0].renommer(String::from("Nouredine Moukaddime"));
    print!("Compte original: ");
    comptes[0].afficher();
    print!("Compte renommé: ");
    compte_renomme.afficher();

    println!("\nRecherche d'un compte par nom");
    let nom_recherche = "Nouhaila";
    
    // Utilisation d'iter() et enumerate() pour chercher un compte
    for (index, compte) in comptes.iter().enumerate() {
        if compte.nom == nom_recherche {
            println!("Compte trouvé à l'index {}: {}", index, compte.nom);
            compte.afficher();
            break;
        }
    }

    println!("\nFermeture d'un compte");
    // On retire un compte du Vec pour le fermer
    if let Some(compte_a_fermer) = comptes.pop() {
        compte_a_fermer.fermer();
    }

    println!("\nComptes restants");
    for (index, compte) in comptes.iter().enumerate() {
        print!("Compte #{}: ", index + 1);
        compte.afficher();
    }
}