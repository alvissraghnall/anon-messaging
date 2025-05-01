pub struct PublicKey(pub String); //B64 encoded SPKKI
pub struct PrivateKey(pub String);
pub struct Signature(pub String);

pub struct KeyPair {
    private_key: PrivateKey,
    public_key: PublicKey,
}
