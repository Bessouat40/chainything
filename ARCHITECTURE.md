# Chainything Architecture Roadmap

## DAG Execution (À implémenter)

### Algorithmes & Techniques
- **Topological Sort** (DFS-based) — parcours en profondeur pour ordonner l'exécution
- **Cycle Detection** — détecter A→B→A via composantes fortement connexes ou coloration (white/gray/black)
- **Memoization/Caching** — stocker les outputs par processor_id pour éviter recalculs
- **Lazy Evaluation** — n'exécuter que les processeurs accessibles depuis la sortie (DAG pruning)

### Concepts clés
- `ExecutionPlan: Vec<ProcessorId>` — ordre topologique établi une fois avant exécution
- `ExecutionCache: HashMap<ProcessorId, Arc<dyn Any>>` — état pendant l'exécution
- `ExecutionResult<T>` — either output T ou (ProcessorError, FailedProcessorId)

---

## Multi-Input Single-Output (Bonnes pratiques)

### Design Pattern
```
Processor {
  inputs: HashMap<InputLabel, Arc<dyn Any>>,   // inputs nommées
  output: Option<Arc<dyn Any>>,                 // 1 seule sortie
}
```

### Validation stricte
- ✅ **Type checking à runtime** — vérifier que chaque input a le bon type attendu
- ✅ **Required vs Optional inputs** — Processor déclare ses exigences
- ✅ **Early validation** — détecter les mismatches avant exécution (pas pendant)
- ✅ **Port matching** — checker que le output_type d'un processor match input_type de son dépendant

### API de composition
- `Pipeline::connect(from_id, output_port, to_id, input_port)` — explicite et verifiable
- Interdire la composition ad-hoc, forcer la déclaration upfront
- Builder pattern pour construire le DAG (type-safe si possible)

### Error Handling
- `ProcessorInputError` — input type mismatch
- `MissingInputError` — input requis mais absent
- `InvalidPipelineError` — cycle ou graph invalide
- Stack traces incluant le chemin d'exécution (quelle dépendance a échoué)

### Testing & Reproducibility
- Sérialiser le DAG (format JSON/YAML) pour reproduire les pipelines
- Deterministic execution order (même si plusieurs chemins possibles)
- Snapshot testing pour les outputs

---

## Code Quality Standards

### Types & Generics (Futur)
- Envisager `Processor<I: Input, O: Output>` pour type-safety (remplace `dyn Any` + downcast)
- Trait bounds: `I: Send + Sync + 'static`, `O: Send + Sync + 'static`

### Concurrency Readiness
- Tous les outputs en `Arc<T>` pour potentiel parallélisation future
- Pas d'état mutable global, chaque execution a sa propre cache
- Thread-safety via Arc + shared references (read-only semantics)

### Documentation
- Chaque Processor documente ses inputs, output type et side-effects
- Schema du DAG queryable at runtime (introspection)
