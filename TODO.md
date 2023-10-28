# Parsing the document
+ [ ] Iterate over character by character and create tokens. We will have several kinds of tokens (called Symbols)
    + Words
    + Punctuations
    + Substitution tokens
        + Replace
        + Spread
+ [ ] Test the parser
    + [ ] Create a system that can convert the tokens to a string
        + Needs to add a intermediate state with information such as "CLING"
        + Then conver this intermediate state to string and compare with the original
