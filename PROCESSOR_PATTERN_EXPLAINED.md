# ProcessorBase + Processor : Explication Complète pour Débutants

## Le Problème : Pourquoi c'est compliqué

Imagine tu veux une boîte magique qui transforme des trucs :

```rust
trait Processor {
    fn process(&mut self, input: ???) -> Result<???, Error>;
}
```

**Le problème :** Chaque processeur prend des trucs DIFFÉRENTS en entrée et sortie !
- ImageReader : prend `String` (chemin) → sort `RawImage`
- Greyscale : prend `RawImage` → sort `RawImage`
- ImageMerger : prend `(RawImage, RawImage)` → sort `RawImage`

Si tu dis `input: String`, alors Greyscale casse. Si tu dis `input: dyn Any`, tu dois downcast partout (lourd).

---

## Solution 1 : Génériques Simples (Ça marche pour UN cas)

```rust
trait Processor<I, O> {
    fn process(&mut self, input: I) -> Result<O, Error>;
}

struct ImageReader;
impl Processor<String, RawImage> for ImageReader {
    fn process(&mut self, input: String) -> Result<RawImage, Error> {
        // ...
    }
}

struct Greyscale;
impl Processor<RawImage, RawImage> for Greyscale {
    fn process(&mut self, input: RawImage) -> Result<RawImage, Error> {
        // ...
    }
}
```

**Ça c'est parfait pour UN processeur !** Mais...

### Le piège : Comment les stocker ensemble ?

```rust
let mut processors = vec![];
processors.push(Box::new(ImageReader)); // Processor<String, RawImage>
processors.push(Box::new(Greyscale));   // Processor<RawImage, RawImage>
// ❌ ERREUR ! Types incompatibles, pas le même <I, O>
```

Pourquoi l'erreur ? Parce que `Box<dyn Processor<I, O>>` demande à Rust : "*quel* est l'I ? *quel* est l'O ?". Mais tu en as plusieurs !

**Analogie :** Tu ne peux pas mélanger des boîtes de différentes formes dans un carton : "boîte carrée" et "boîte ronde" sont des **types différents**.

---

## Solution 2 : ProcessorBase (le pattern qu'il faut)

L'idée : créer un trait **sans génériques** comme "interface commune" pour les stocker, puis un trait **générique** pour la sécurité des types.

### Étape 1 : Trait sans génériques (la "boîte générique")

```rust
pub trait ProcessorBase: Send + Sync {
    // Infos sur ses types
    fn input_type_id(&self) -> TypeId;
    fn output_type_id(&self) -> TypeId;
    
    // Exécution "effacée" (type-erased)
    fn process_erased(
        &mut self, 
        input: Option<Arc<dyn Any + Send + Sync>>
    ) -> Result<Arc<dyn Any + Send + Sync>, ProcessorError>;
}
```

**Que se passe-t-il :**
- Tous les Processors implémentent ça (peu importe leurs I/O)
- Donc tu peux les stocker dans la même vec/map : `Vec<Box<dyn ProcessorBase>>`
- ✅ Ça résout le problème de stockage !

### Étape 2 : Trait générique (sécurité des types DANS chaque impl)

```rust
pub trait Processor<I: Send + Sync + 'static, O: Send + Sync + 'static>: ProcessorBase {
    // Versioning "typée" = plus sûre
    fn process(&mut self, input: Option<Arc<I>>) -> Result<Arc<O>, ProcessorError>;
}
```

**Que se passe-t-il :**
- `I` et `O` sont connus au moment de coder le Processor
- Pas besoin de downcast dans ton code, c'est type-safe
- Mais on peut quand même les stocker ensemble via ProcessorBase

### Étape 3 : Le "bridge" (blanket implementation)

```rust
impl<I, O, P> ProcessorBase for P
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
    P: Processor<I, O>, // Si P implémente Processor<I, O>
{
    fn input_type_id(&self) -> TypeId {
        TypeId::of::<I>()
    }

    fn output_type_id(&self) -> TypeId {
        TypeId::of::<O>()
    }

    fn process_erased(
        &mut self, 
        input: Option<Arc<dyn Any + Send + Sync>>
    ) -> Result<Arc<dyn Any + Send + Sync>, ProcessorError> {
        // Convertir dyn Any → I (downcast)
        let typed_input = input.and_then(|i| i.downcast::<I>().ok());
        
        // Appeler la version typée
        self.process(typed_input)
            // Convertir O → dyn Any
            .map(|output| output as Arc<dyn Any + Send + Sync>)
    }
}
```

**Qu'est-ce qui se passe :**
- Rust génère automatiquement cette impl pour TOUS les Processors
- Le downcast/upcast (conversion Any) se fait UNE FOIS
- Chaque Processor concrèt ne voit jamais `dyn Any`, seulement ses types I/O

---

## Exemple Complet : Ça marche !

```rust
// ImageReader : String → RawImage
struct ImageReader;
impl Processor<String, RawImage> for ImageReader {
    fn process(&mut self, path: Option<Arc<String>>) -> Result<Arc<RawImage>, Error> {
        let path = path.ok_or(ProcessorError::MissingInput)?;
        let img = image::open(path.as_ref())?;
        Ok(Arc::new(RawImage { /* ... */ }))
    }
}

// Greyscale : RawImage → RawImage
struct Greyscale;
impl Processor<RawImage, RawImage> for Greyscale {
    fn process(&mut self, img: Option<Arc<RawImage>>) -> Result<Arc<RawImage>, Error> {
        let img = img.ok_or(ProcessorError::MissingInput)?;
        // ... greyscale logic ...
        Ok(Arc::new(greyscaled))
    }
}

// Dans Pipeline : stockage hétérogène
pub struct Pipeline {
    processors: HashMap<String, Box<dyn ProcessorBase>>,
}

// Utilisation
let mut pipeline = Pipeline::new();
pipeline.add_processor("reader", Box::new(ImageReader));
pipeline.add_processor("grey", Box::new(Greyscale));

// Exécution : appelle process_erased sous le capot
pipeline.execute()?;
```

---

## C'est une bonne pratique ? ✅ OUI, mais...

### ✅ Avantages
| Avantage | Pourquoi |
|----------|---------|
| **Type-safe** | Chaque Processor code ses types réels, pas `dyn Any` partout |
| **Extensible** | Ajoute des Processors sans changer Pipeline |
| **Hétérogène** | Peux mixer String→Image et Image→Image dans la même collection |
| **Performant** | Downcasts centralisés, une seule place où ça peut échouer |
| **Maintenance** | Changer les types d'un Processor ne casse pas les autres |

### ⚠️ Inconvénients
| Inconvénient | Quand ça pose problème |
|--------------|----------------------|
| **Complexe** | Faut comprendre les génériques + trait objects + TypeId |
| **Verbeux** | 2 traits au lieu d'1, code généré |
| **Runtime type checks** | `TypeId::of()` c'est cool mais pas foolproof (doxycycline mal) |
| **Compilation lente** | Génériques = monomorphization = binaire plus gros |

### 📊 Comparaison avec les alternatives

#### Alternative 1 : Tout en `dyn Any` (simple, lourd)
```rust
trait Processor {
    fn process(&mut self, input: Option<Arc<dyn Any>>) -> Result<Arc<dyn Any>, Error>;
}
```
- ✅ Simple
- ❌ Downcast DANS chaque processor (répétition)
- ❌ Pas de vérification des types jusqu'à l'exécution

#### Alternative 2 : Enum de tous les types (inextensible)
```rust
enum ProcessorInput {
    String(Arc<String>),
    RawImage(Arc<RawImage>),
}
```
- ✅ Type-safe
- ❌ Casse si t'ajoutes un type (gérer l'enum partout)
- ❌ Peu extensible

#### Alternative 3 : ProcessorBase + Processor (NOTRE PATTERN) ⭐
- ✅ Type-safe + hétérogène + extensible
- ⚠️ Plus complexe mais c'est un investissement une fois

---

## Quand l'utiliser ?

✅ **Utilise ProcessorBase + Processor si :**
- Tu as plusieurs types de Processors avec des I/O différents
- C'est une librairie (extensibilité importante)
- La performance/sécurité des types dépasse en importance la simplicité

❌ **Reste simple (`dyn Any`) si :**
- Tous tes Processors ont les mêmes I/O
- C'est un projet petit/personnel
- Tu préfères 200 lignes simples à 1000 lignes génériques

---

## Résumé Mental

**ProcessorBase** = "tous les Processors portent une badge commun" (stockage)
**Processor<I, O>** = "chaque Processor sait ses vraies formes" (sécurité)
**Blanket impl** = "la traduction automatique" (magie Rust)

C'est un pattern **solide et recommandé** pour les architectures de plugins/pipelines. Vaut le coup d'apprendre !
