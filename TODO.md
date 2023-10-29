# Tokenizing the document
+ [x] Tokenize the document. We have 2 kinds of tokens Words and Punctuations.
+ [x] Test tokinizer. We will do this by doing a round trip of String -> Tokens -> String
# Parsing the document
+ [x] Iterate over Tokens and create symbols. We will have several kinds of Symbols
    + Words
        + Punctuations: will be treated as a word as well (since no difference after parsing)
    + Substitution tokens
        + Replace
        + Spread
+ [ ] Test the parser
    + [ ] Create a system that can convert the tokens to a string
        + Needs to add a intermediate state with information such as "CLING"
        + Then conver this intermediate state to string and compare with the original
