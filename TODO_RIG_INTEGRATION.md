# TODO: Rig Rust AI Framework Integration

## Task: Integrate Rig into the Rust Flash Loan Application

### Information Gathered:
- `backend/Cargo.toml` exists with tokio, dotenvy already
- Main entry: `backend/main.rs`
- Learning module: `backend/learning/mod.rs`
- `.gitignore` already excludes `.env` files for security

### Plan:

#### Step 1: Update Cargo.toml with dependencies
- Add `rig = "0.1"` (latest stable from crates.io)
- Add `anyhow = "1"`
- Already has tokio and dotenvy

#### Step 2: Create AI module files
```
backend/ai/
├── mod.rs      # Module exports
├── groq.rs    # Groq provider implementation
├── openrouter.rs # OpenRouter provider
└── manager.rs # AI provider dispatcher
```

#### Step 3: Implement providers
- Groq: `groq_prompt(system_prompt, user_prompt) -> Result<String>`
- OpenRouter: `openrouter_prompt(system_prompt, user_prompt) -> Result<String>`
- Manager: `ask_ai(provider, system_prompt, user_prompt) -> Result<String>`

#### Step 4: Test compilation
- Run `cargo check` on backend

### Deliverables:
- [ ] Updated `backend/Cargo.toml`
- [ ] `backend/ai/mod.rs`
- [ ] `backend/ai/groq.rs`  
- [ ] `backend/ai/openrouter.rs`
- [ ] `backend/ai/manager.rs`
- [ ] Compilation verification

### Dependencies:
- rig (latest stable)
- tokio (already in Cargo.toml)
- dotenvy (already in Cargo.toml)
- anyhow (need to add)

### Security Considerations:
- API keys loaded from `.env` (already in .gitignore)
- No hardcoded secrets
- No `unwrap()` or `expect()` in production code
