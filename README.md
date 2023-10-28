# PTemplator

TODO: describe what we are trying to achieve 
TODO: describe the template

## Design
+ We will have several stages (similar to a complier)
    1. Tokenizer
    2. Parser
    3. Substitutor
    4. Text Generator
### Tokenizer
+ Input : `String` (NOTE: our tokens are not multiline)
+ Output : `[Token]` where Token is either a word (a continuous sequence of non punctuation characters) or a punctuation

### Parser
+ Input : `[Token]`
+ Output : `[Symbol]`  where Symbol is either a word, punctuation or Variable (We will have different kinds of Variables)

### Substitutor
+ Input : `[Symbol]` 
+ Output : `[Token]`

### Text Generator
+ Input : `[Token]`
+ Output : `String`

+ Each stage will take the ownership of the output from the previous stage
+ Each stages output must have position information
