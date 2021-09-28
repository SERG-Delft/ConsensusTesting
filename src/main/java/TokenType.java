public enum TokenType {
    None(1), // unused
    NodePublic(28),
    NodePrivate(32),
    AccountID(0),
    AccountPublic(35),
    AccountSecret(34),
    FamilyGenerator(41),  // unused
    FamilySeed(33);

    public int value;

    TokenType(int value) {
        this.value = value;
    }
}
