# `winter` module

Brief explainer on the folders here:

- `lexer/` - turns raw text into lexemes (tokens)
- `parser/` - turns lexemes into AST nodes
- `compiler/` - turns AST nodes into Nix AST nodes
- `nix/` - turns nix AST into text
- `handler/` - joins entire flow together in `SourceHandler`
- `cli/` - has CLI for interacting with handler
