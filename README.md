# Synthèse - Première séance Rust

## 1. Commandes Cargo essentielles

- **`cargo run`** : compile et exécute le projet
- **`cargo build`** : compile le projet en mode debug
- **`cargo check`** : vérifie le code sans compiler

## 2. Variables et Types de données

### Types numériques
- **`i32`** : entier signé 32 bits (-2,147,483,648 à 2,147,483,647) - **type par défaut**
- **`u32`** : entier non signé 32 bits (0 à 4,294,967,295)
- **`i64`** : entier signé 64 bits - très grand intervalle
- **`u8`** : entier non signé 8 bits (0 à 255)
- **`f32`** : flottant simple précision 32 bits
- **`f64`** : flottant double précision 64 bits - **type par défaut**

### Types supplémentaires
- **`bool`** : booléen (`true` ou `false`)
- **`char`** : caractère Unicode 4 octets (`'a'`, `'1'`, `'\n'`)
- **`String`** : chaîne mutable allouée dynamiquement
- **`&str`** : référence de chaîne immuable

### Déclaration de variables
```rust
let nom = "Kevin";              // &str (référence de chaîne)
let age: u32 = 30;             // Annotation de type explicite
let temperature: f32 = 32.5;    // Nombre flottant
```

### Mutabilité
```rust
let x = 5;          // Immuable par défaut
let mut y = 10;     // Mutable avec mot-clé 'mut'
y = 15;             // Modification possible
```

### Convention de nommage
- **Snake_case** obligatoire en Rust
- Pas de chiffres en début, pas d'espaces ni de tirets
- Préfixer par `_` pour éviter les warnings sur variables non utilisées

## 2. Fonctions

Les fonctions en Rust se déclarent avec le mot-clé `fn`. Elles peuvent prendre des paramètres typés et retourner une valeur. Le type de retour est spécifié après `->`. Une fonction sans `return` explicite retourne la dernière expression (sans point-virgule).

### Syntaxe de base
```rust
fn addition(n1: i32, n2: i32) -> i32 {
    n1 + n2  // Retour implicite (pas de point-virgule)
}

fn say_hello(nom: &str) {  // &str = référence de chaîne
    println!("Bonjour, {}", nom);
}
```

### Appel de fonction
```rust
let resultat = addition(12, 3);
say_hello("Loggi Hello");
```

## 3. Structures de contrôle

Les structures de contrôle permettent de diriger le flux d'exécution du programme selon des conditions ou des motifs.

### Conditions
Les conditions `if/else` évaluent une expression booléenne pour exécuter différents blocs de code.

```rust
let nombre = 16;
if nombre % 2 == 0 {
    println!("Pair");
} else {
    println!("Impair");
}
```

### Pattern Matching (équivalent du switch)
Le `match` permet de comparer une valeur contre plusieurs motifs. Contrairement au switch, tous les cas possibles doivent être couverts (exhaustivité).

```rust
let nombre = 5;
match nombre {
    1 => println!("Un"),
    2 => println!("Deux"),
    5 => println!("Cinq"),
    _ => println!("Autre nombre"),  // Cas par défaut obligatoire
}
```

## 4. Boucles

Les boucles permettent de répéter l'exécution d'un bloc de code. Rust propose trois types de boucles principales.

### Boucles détaillées

#### Boucle for avec intervalles
La boucle `for` itère sur une séquence ou un intervalle. Les intervalles peuvent être inclusifs (`..=`) ou exclusifs (`..`).

```rust
for i in 1..=10 {      // Inclusif (1 à 10)
    println!("i vaut {}", i);
}

for i in 1..5 {        // Exclusif (1 à 4)
    println!("i vaut {}", i);
}
```

#### Boucle while
La boucle `while` continue tant qu'une condition reste vraie.

```rust
let mut compteur = 0;
while compteur < 4 {
    println!("Compteur = {}", compteur);
    compteur += 1;
}
```

#### Boucle loop (infinie avec break)
La boucle `loop` est infinie par défaut et nécessite un `break` explicite pour s'arrêter.

```rust
let mut compteur = 0;
loop {
    println!("Compteur: {}", compteur);
    compteur += 1;
    if compteur == 3 {
        break;  // Sortie de boucle
    }
}
```

## 5. Collections et itération

Les collections permettent de stocker plusieurs éléments. Rust distingue les tableaux (taille fixe) des vecteurs (taille dynamique).

### Tableaux (taille fixe)
Les tableaux ont une taille définie à la compilation et ne peuvent pas changer de taille.

```rust
let voitures = ["jeep", "renault", "bmw"];
let tab: [i32; 4] = [11, 23, 19, 19];  // Type et taille explicites
let _tab2: [i32; 4] = [1, 2, 3, 4];    // _ pour éviter warning non-utilisé

// Parcours simple
for voiture in voitures {
    println!("Voiture : {}", voiture);
}

// Parcours avec référence
for &elt in &tab {
    println!("Élément : {}", elt);
}

// Parcours par index
for i in 0..tab.len() {
    println!("tab[{}] = {}", i, tab[i]);
}
```

### Vecteurs (taille dynamique)
Les vecteurs peuvent grandir ou rétrécir pendant l'exécution du programme.

```rust
let noms = vec![String::from("Kevin"), String::from("Nouredine")];
```

### Enumerate (index + valeur)
La méthode `enumerate()` permet d'obtenir à la fois l'index et la valeur lors de l'itération.

```rust
for (i, voiture) in voitures.iter().enumerate() {
    println!("Index {} : {}", i, voiture);
}
```

**Méthodes importantes :**
- **`iter()`** : crée un itérateur sur la collection sans la consommer
- **`enumerate()`** : transforme l'itérateur en séquence (index, valeur)

## 6. Structures (struct)

Les structures permettent de regrouper des données liées sous un même type personnalisé. Elles sont similaires aux classes dans d'autres langages mais sans héritage.

### Définition et utilisation
```rust
struct Salarie {
    nom: String,
    ville: String,
    age: u32
}

// Création d'instance
let kevin = Salarie {
    nom: String::from("Kevin"),
    ville: String::from("Lyon"),
    age: 25
};

// Accès aux attributs
println!("Nom: {}, Ville: {}, Age: {}", kevin.nom, kevin.ville, kevin.age);
```

## 7. Implémentation de méthodes (impl)

Le bloc `impl` permet d'associer des fonctions (méthodes) à une structure. Ces méthodes peuvent accéder et manipuler les données de l'instance.

### Types d'emprunts dans les méthodes

**`&self`** - Emprunt immuable : lecture seule des données
**`&mut self`** - Emprunt mutable : lecture et modification autorisées  
**`self`** - Prise de possession : consomme l'objet (inaccessible après)

```rust
impl CompteBancaire {
    fn afficher(&self) {              // Lecture seule
        println!("Compte de {} : {} €", self.nom, self.solde);
    }
    
    fn deposer(&mut self, montant: f64) {  // Modification autorisée
        self.solde += montant;
    }
    
    fn fermer(self) {                 // Prend possession
        println!("Compte fermé");
        // self est consommé, objet inaccessible après cette méthode
    }
}
```

## 9. Exemple concret : Menu interactif

```rust
use std::io;

let options = ["Afficher solde", "Retrait", "Liste comptes", "Quitter"];

println!("Menu:");
for (i, option) in options.iter().enumerate() {
    println!("{}. {}", i + 1, option); 
}

println!("Votre choix:");
let mut choix = String::new();
io::stdin().read_line(&mut choix).expect("Erreur de lecture");

let choix: usize = match choix.trim().parse() {
    Ok(num) => num,
    Err(_) => {
        println!("Numéro invalide");
        return;
    }
};

if choix == 0 || choix > options.len() {
    println!("Choix invalide");
} else {
    println!("Sélectionné : {}", options[choix - 1]);
}
```
