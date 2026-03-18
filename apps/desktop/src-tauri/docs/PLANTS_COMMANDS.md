# Plants Commands - Documentação Técnica

Este documento explica o fluxo completo de cada command do módulo `plants`, desde a chamada do frontend até o retorno.

## Arquitetura Geral

```
┌─────────────┐     IPC      ┌──────────────┐     ┌───────────────┐     ┌─────────────┐
│  Frontend   │ ──────────►  │   Command    │ ──► │  PlantService │ ──► │ PlantStore  │
│  (invoke)   │              │  (plants.rs) │     │   (service)   │     │  (HashMap)  │
└─────────────┘              └──────────────┘     └───────────────┘     └─────────────┘
                                    │                                          │
                                    │         ┌──────────────┐                 │
                                    └────────►│   AppState   │◄────────────────┘
                                              │  (managed)   │
                                              └──────────────┘
```

## Componentes

| Arquivo | Responsabilidade |
|---------|------------------|
| `lib.rs` | Entry point, registra commands e injeta `AppState` |
| `state/app_state.rs` | Estado global, encapsula `PlantStore` |
| `state/plant_store.rs` | Armazenamento thread-safe em memória |
| `core/services/plant.rs` | Lógica de negócio (validação, criação) |
| `core/models/plant.rs` | Structs de dados (Plant, PlantResponse, etc.) |
| `core/error.rs` | Tipos de erro e conversão para DTO |
| `commands/plants.rs` | Handlers IPC expostos ao frontend |

---

## 1. `create_plant`

**Propósito:** Cria uma nova planta e persiste no runtime.

### Fluxo

```rust
#[tauri::command]
pub fn create_plant(
    state: State<'_, AppState>,
    request: CreatePlantRequest,
) -> Result<PlantResponse, ErrorDto>
```

#### Passo a passo:

1. **Frontend chama via IPC:**
   ```typescript
   await invoke('create_plant', { request: { name: "Planta 1", variables: [...] } })
   ```

2. **Tauri deserializa o JSON** para `CreatePlantRequest`:
   ```rust
   pub struct CreatePlantRequest {
       pub name: String,
       pub variables: Vec<CreatePlantVariableRequest>,
       pub driver_id: Option<String>,
       pub controller_ids: Option<Vec<String>>,
   }
   ```

3. **Command recebe `State<AppState>`** (injetado automaticamente pelo Tauri):
   ```rust
   // Tauri injeta o state registrado em .manage(AppState::new())
   state: State<'_, AppState>
   ```

4. **Chama `PlantService::create_and_store`:**
   ```rust
   let plant = PlantService::create_and_store(state.plants(), request)
       .map_err(ErrorDto::from)?;
   ```

5. **Service valida e constrói a planta:**
   ```rust
   pub fn create_and_store(store: &PlantStore, request: CreatePlantRequest) -> AppResult<Plant> {
       let plant = Self::build_plant(request)?;  // Valida e cria
       store.insert(plant.clone())?;              // Persiste no store
       Ok(plant)
   }
   ```

6. **Validações em `build_plant`:**
   - Nome não pode ser vazio
   - Deve ter pelo menos uma variável
   - Cada variável: nome não vazio, `pv_min < pv_max`, `setpoint` dentro do range

7. **Store persiste com lock de escrita:**
   ```rust
   pub fn insert(&self, plant: Plant) -> AppResult<()> {
       let mut plants = self.plants.write();  // Lock exclusivo (RwLock)
       if plants.contains_key(&plant.id) {
           return Err(AppError::InvalidArgument(...));
       }
       plants.insert(plant.id.clone(), plant);
       Ok(())
   }
   ```

8. **Retorna `PlantResponse`** (serializado para JSON):
   ```rust
   Ok(PlantResponse::from(&plant))
   ```

### Diagrama de Sequência

```
Frontend          Command           Service           Store
    │                │                 │                │
    │─invoke(...)──►│                 │                │
    │                │─create_and_store()──►│          │
    │                │                 │─build_plant()─┤
    │                │                 │   (valida)    │
    │                │                 │◄──────────────┤
    │                │                 │─insert()──────►│
    │                │                 │               │ plants.write()
    │                │                 │               │ HashMap::insert()
    │                │                 │◄──────────────┤
    │                │◄────Plant──────│                │
    │◄─PlantResponse─│                 │                │
```

---

## 2. `list_plants`

**Propósito:** Retorna todas as plantas em memória.

### Fluxo

```rust
#[tauri::command]
pub fn list_plants(state: State<'_, AppState>) -> Vec<PlantResponse>
```

1. **Chama `PlantService::list`:**
   ```rust
   PlantService::list(state.plants())
   ```

2. **Store retorna cópias (com lock de leitura):**
   ```rust
   pub fn list(&self) -> Vec<Plant> {
       let plants = self.plants.read();  // Lock compartilhado
       plants.values().cloned().collect()
   }
   ```

3. **Mapeia para `PlantResponse`:**
   ```rust
   .iter()
   .map(PlantResponse::from)
   .collect()
   ```

**Nota:** Lock de leitura permite múltiplas leituras simultâneas.

---

## 3. `get_plant`

**Propósito:** Busca uma planta específica por ID.

### Fluxo

```rust
#[tauri::command]
pub fn get_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto>
```

1. **Chama `PlantService::get`:**
   ```rust
   let plant = PlantService::get(state.plants(), &id).map_err(ErrorDto::from)?;
   ```

2. **Store busca com lock de leitura:**
   ```rust
   pub fn get(&self, id: &str) -> AppResult<Plant> {
       let plants = self.plants.read();
       plants
           .get(id)
           .cloned()
           .ok_or_else(|| AppError::NotFound(format!("Planta '{}' não encontrada", id)))
   }
   ```

3. **Se não encontrar, retorna erro:**
   ```json
   { "code": "NOT_FOUND", "message": "Planta 'xyz' não encontrada" }
   ```

---

## 4. `remove_plant`

**Propósito:** Remove uma planta do runtime.

### Fluxo

```rust
#[tauri::command]
pub fn remove_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto>
```

1. **Chama `PlantService::remove`:**
   ```rust
   let plant = PlantService::remove(state.plants(), &id).map_err(ErrorDto::from)?;
   ```

2. **Store remove com lock de escrita:**
   ```rust
   pub fn remove(&self, id: &str) -> AppResult<Plant> {
       let mut plants = self.plants.write();  // Lock exclusivo
       plants
           .remove(id)
           .ok_or_else(|| AppError::NotFound(...))
   }
   ```

3. **Retorna a planta removida** (permite undo no frontend se necessário).

---

## 5. `connect_plant`

**Propósito:** Marca a planta como conectada.

### Fluxo

```rust
#[tauri::command]
pub fn connect_plant(state: State<'_, AppState>, id: String) -> Result<PlantResponse, ErrorDto>
```

1. **Chama `PlantService::connect`:**
   ```rust
   let plant = PlantService::connect(state.plants(), &id).map_err(ErrorDto::from)?;
   ```

2. **Service usa `store.update` com closure:**
   ```rust
   pub fn connect(store: &PlantStore, id: &str) -> AppResult<Plant> {
       store.update(id, |plant| {
           plant.connected = true;
       })
   }
   ```

3. **Store aplica atualização atomicamente:**
   ```rust
   pub fn update<F>(&self, id: &str, updater: F) -> AppResult<Plant>
   where
       F: FnOnce(&mut Plant),
   {
       let mut plants = self.plants.write();
       let plant = plants.get_mut(id).ok_or_else(|| ...)?;
       updater(plant);  // Aplica a closure
       Ok(plant.clone())
   }
   ```

---

## 6. `disconnect_plant`

**Propósito:** Marca a planta como desconectada.

### Fluxo

```rust
pub fn disconnect(store: &PlantStore, id: &str) -> AppResult<Plant> {
    store.update(id, |plant| {
        plant.connected = false;
        plant.paused = false;  // Também reseta pausa
    })
}
```

---

## 7. `pause_plant`

**Propósito:** Pausa a execução da planta.

### Fluxo

```rust
pub fn pause(store: &PlantStore, id: &str) -> AppResult<Plant> {
    store.update(id, |plant| {
        plant.paused = true;
    })
}
```

---

## 8. `resume_plant`

**Propósito:** Resume a execução da planta.

### Fluxo

```rust
pub fn resume(store: &PlantStore, id: &str) -> AppResult<Plant> {
    store.update(id, |plant| {
        plant.paused = false;
    })
}
```

---

## Tratamento de Erros

Todos os commands seguem o padrão:

```rust
.map_err(ErrorDto::from)?
```

### Conversão `AppError` → `ErrorDto`

```rust
impl From<AppError> for ErrorDto {
    fn from(err: AppError) -> Self {
        let (code, message) = match err {
            AppError::InvalidArgument(msg) => ("INVALID_ARGUMENT", msg),
            AppError::NotFound(msg) => ("NOT_FOUND", msg),
            AppError::IoError(msg) => ("IO_ERROR", msg),
            AppError::InternalError => ("INTERNAL_ERROR", "An internal error occurred".into()),
        };
        Self { code: code.to_string(), message }
    }
}
```

### No Frontend

```typescript
try {
    const plant = await invoke('create_plant', { request });
} catch (error) {
    // error = { code: "INVALID_ARGUMENT", message: "Nome da planta é obrigatório" }
}
```

---

## Thread Safety

O `PlantStore` usa `parking_lot::RwLock`:

| Operação | Tipo de Lock | Bloqueio |
|----------|--------------|----------|
| `list`, `get`, `exists` | Read | Compartilhado (múltiplas leituras) |
| `insert`, `remove`, `update`, `clear` | Write | Exclusivo (uma escrita) |

**Por que `parking_lot`?**
- Mais performático que `std::sync::RwLock`
- Não requer `unwrap()` (não retorna `PoisonError`)
- API mais ergonômica

---

## Inicialização

Em `lib.rs`:

```rust
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())  // Registra estado global
        .invoke_handler(tauri::generate_handler![
            create_plant,
            list_plants,
            // ...
        ])
        .run(...)
}
```

O `AppState::new()` cria:
```rust
Self {
    plant_store: Arc::new(PlantStore::new()),  // HashMap vazio
}
```

O `Arc` permite compartilhar o store entre threads de forma segura.
