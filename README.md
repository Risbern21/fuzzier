# Fuzzier - A fully local semantic search powered file explorer
---
Fuzzier uses the ```openai/clip-vit-base-patch16``` multi-modal embeddings model for generating both text and image embedings.

The GUI is written in Rust (Iced) because:
- using electron for a file explorer is probably overkill.
- i dont like electron.
---
## Generating embeddings
"openai/clip-vit-base-patch16" is a pretty lightweight (~1.2GB) model so it can be run even with a cpu without much performance drop.
